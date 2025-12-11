mod strings;

use crate::types::Arr;
use iced_x86::{CpuidFeature, Decoder as Iced, DecoderOptions, Instruction};
use std::cmp::Reverse;

/// Keep in sync with `IcedConstants::CPUID_FEATURE_ENUM_COUNT`!
const FEATURE_COUNT: usize = 178;
/// Keep in sync with `IcedConstants::MNEMONIC_ENUM_COUNT`!
const MNEMONIC_ENUM_COUNT: usize = 1894;
/// Keep in sync with `IcedConstants::REGISTER_ENUM_COUNT`!
const REGISTER_ENUM_COUNT: usize = 256;

const OPTIONS: u32 = DecoderOptions::NO_INVALID_CHECK;

pub trait Counter: Sized {
    fn name(&self) -> &'static str;
    fn count(&self) -> u64;

    fn sort(counters: &mut [Self]) {
        counters.sort_unstable_by_key(|counter| Reverse(counter.count()));
    }
}

trait DataMapper<T>: Sized {
    fn filter(_: &(usize, T)) -> bool;
    fn map(_: (usize, T)) -> Self;

    fn map_data(input: Arr<T>) -> Arr<Self> {
        input.into_iter().enumerate().filter(Self::filter).map(Self::map).collect()
    }
}

pub struct FeatureCounter {
    id: usize,
    count: u64,
}

impl FeatureCounter {
    pub fn is_cpuid(&self) -> bool {
        self.id == CpuidFeature::CPUID as usize
    }
}

impl Counter for FeatureCounter {
    fn name(&self) -> &'static str {
        strings::FEATURE[self.id]
    }

    fn count(&self) -> u64 {
        self.count
    }
}

impl DataMapper<u64> for FeatureCounter {
    fn filter((_, count): &(usize, u64)) -> bool {
        *count > 0
    }

    fn map((id, count): (usize, u64)) -> Self {
        Self { id, count }
    }
}

pub struct MnemonicCounter {
    id: usize,
    count: u64,
}

impl Counter for MnemonicCounter {
    fn name(&self) -> &'static str {
        strings::MNEMONIC[self.id]
    }

    fn count(&self) -> u64 {
        self.count
    }
}

impl DataMapper<u64> for MnemonicCounter {
    fn filter((_, count): &(usize, u64)) -> bool {
        *count > 0
    }

    fn map((id, count): (usize, u64)) -> Self {
        Self { id, count }
    }
}

struct Feature {
    count: u64,
    mnemonics: Arr<u64>,
}

impl Feature {
    fn new(_: usize) -> Self {
        Self { count: 0, mnemonics: Arr::from(vec![0; MNEMONIC_ENUM_COUNT]) }
    }

    fn add(&mut self, mnemonic: usize) {
        self.count += 1;
        self.mnemonics[mnemonic] += 1;
    }

    fn count(&self) -> u64 {
        self.count
    }

    fn into_mnemonics(self) -> Arr<MnemonicCounter> {
        MnemonicCounter::map_data(self.mnemonics)
    }
}

pub struct DetailCounter {
    id: usize,
    count: u64,
    mnemonics: Arr<MnemonicCounter>,
}

impl DetailCounter {
    pub fn mnemonics(&self) -> &[MnemonicCounter] {
        &self.mnemonics
    }
}

impl Counter for DetailCounter {
    fn name(&self) -> &'static str {
        strings::FEATURE[self.id]
    }

    fn count(&self) -> u64 {
        self.count
    }

    fn sort(details: &mut [Self]) {
        details.sort_unstable_by_key(|detail| Reverse(detail.count()));

        for detail in details {
            MnemonicCounter::sort(&mut detail.mnemonics);
        }
    }
}

impl DataMapper<Feature> for DetailCounter {
    fn filter((_, feature): &(usize, Feature)) -> bool {
        feature.count() > 0
    }

    fn map((id, feature): (usize, Feature)) -> Self {
        Self { id, count: feature.count(), mnemonics: feature.into_mnemonics() }
    }
}

pub struct RegisterCounter {
    id: usize,
    count: u64,
}

impl Counter for RegisterCounter {
    fn name(&self) -> &'static str {
        strings::REGISTER[self.id]
    }

    fn count(&self) -> u64 {
        self.count
    }
}

impl DataMapper<u64> for RegisterCounter {
    fn filter((_, count): &(usize, u64)) -> bool {
        *count > 0
    }

    fn map((id, count): (usize, u64)) -> Self {
        Self { id, count }
    }
}

pub trait Task {
    type Result;
    fn new() -> Self;
    fn add(&mut self, instruction: Instruction);
    fn into_result(self) -> Self::Result;
}

pub struct TaskCount {
    features: Arr<u64>,
}

impl Task for TaskCount {
    type Result = Arr<FeatureCounter>;

    fn new() -> Self {
        let features = Arr::from(vec![0; FEATURE_COUNT]);
        Self { features }
    }

    fn add(&mut self, instruction: Instruction) {
        if instruction.is_invalid() {
            return;
        }

        for id in instruction.cpuid_features() {
            self.features[*id as usize] += 1;
        }
    }

    fn into_result(self) -> Self::Result {
        FeatureCounter::map_data(self.features)
    }
}

pub struct TaskDetail {
    features: Arr<Feature>,
    registers: Arr<u64>,
}

impl Task for TaskDetail {
    type Result = (Arr<DetailCounter>, Arr<RegisterCounter>);

    fn new() -> Self {
        let features = (0..FEATURE_COUNT).map(Feature::new).collect();
        let registers = Arr::from(vec![0; REGISTER_ENUM_COUNT]);
        Self { features, registers }
    }

    fn add(&mut self, instruction: Instruction) {
        if instruction.is_invalid() {
            return;
        }

        let mnemonic = instruction.mnemonic() as usize;

        for id in instruction.cpuid_features() {
            self.features[*id as usize].add(mnemonic);
        }

        for op in 0..4 {
            let register = instruction.op_register(op) as usize;

            if register > 0 {
                self.registers[register] += 1;
            }
        }
    }

    fn into_result(self) -> Self::Result {
        let features = DetailCounter::map_data(self.features);
        let registers = RegisterCounter::map_data(self.registers);
        (features, registers)
    }
}

pub struct Decoder<T: Task> {
    bitness: u32,
    task: T,
}

impl<T: Task> Decoder<T> {
    pub fn new(bitness: u32) -> Self {
        Self { bitness, task: T::new() }
    }

    pub fn read(&mut self, data: &[u8]) {
        let decoder = Iced::new(self.bitness, data, OPTIONS);

        for instruction in decoder {
            self.task.add(instruction);
        }
    }

    pub fn into_result(self) -> T::Result {
        self.task.into_result()
    }
}
