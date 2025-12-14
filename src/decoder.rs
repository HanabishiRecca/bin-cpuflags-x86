mod strings;

use crate::types::Arr;
use iced_x86::{CpuidFeature, Decoder as Iced, DecoderOptions, Instruction};
use std::{cmp::Reverse, marker::PhantomData};

/// Keep in sync with `IcedConstants::CPUID_FEATURE_ENUM_COUNT`!
const FEATURE_COUNT: usize = 178;
/// Keep in sync with `IcedConstants::MNEMONIC_ENUM_COUNT`!
const MNEMONIC_ENUM_COUNT: usize = 1894;
/// Keep in sync with `IcedConstants::REGISTER_ENUM_COUNT`!
const REGISTER_ENUM_COUNT: usize = 256;

const OPTIONS: u32 = DecoderOptions::NO_INVALID_CHECK;

pub trait Item: Sized {
    fn name(&self) -> &'static str;
    fn count(&self) -> u64;
    fn sort(&mut self) {}

    fn sort_list(items: &mut [Self]) {
        items.sort_unstable_by_key(|counter| Reverse(counter.count()));
        items.iter_mut().for_each(Self::sort);
    }
}

trait Map<T>: Item {
    fn filter(_: &(usize, T)) -> bool;
    fn map(_: (usize, T)) -> Self;

    fn map_items(items: Arr<T>) -> Arr<Self> {
        items.into_iter().enumerate().filter(Self::filter).map(Self::map).collect()
    }
}

pub trait Name {
    fn name(id: usize) -> &'static str;
}

pub struct Feature;

impl Name for Feature {
    fn name(id: usize) -> &'static str {
        strings::FEATURE[id]
    }
}

pub struct Mnemonic;

impl Name for Mnemonic {
    fn name(id: usize) -> &'static str {
        strings::MNEMONIC[id]
    }
}

pub struct Register;

impl Name for Register {
    fn name(id: usize) -> &'static str {
        strings::REGISTER[id]
    }
}

pub struct Count<T: Name> {
    id: usize,
    count: u64,
    _name: PhantomData<T>,
}

impl Count<Feature> {
    pub fn is_cpuid(&self) -> bool {
        self.id == CpuidFeature::CPUID as usize
    }
}

impl<T: Name> Item for Count<T> {
    fn name(&self) -> &'static str {
        T::name(self.id)
    }

    fn count(&self) -> u64 {
        self.count
    }
}

impl<T: Name> Map<u64> for Count<T> {
    fn filter((_, count): &(usize, u64)) -> bool {
        *count > 0
    }

    fn map((id, count): (usize, u64)) -> Self {
        Self { id, count, _name: PhantomData }
    }
}

struct DetailCounter {
    count: u64,
    mnemonics: Arr<u64>,
}

impl DetailCounter {
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

    fn into_mnemonics(self) -> Arr<u64> {
        self.mnemonics
    }
}

pub struct Detail {
    id: usize,
    count: u64,
    mnemonics: Arr<Count<Mnemonic>>,
}

impl Detail {
    pub fn mnemonics(&self) -> &[Count<Mnemonic>] {
        &self.mnemonics
    }
}

impl Item for Detail {
    fn name(&self) -> &'static str {
        strings::FEATURE[self.id]
    }

    fn count(&self) -> u64 {
        self.count
    }

    fn sort(&mut self) {
        Item::sort_list(&mut self.mnemonics);
    }
}

impl Map<DetailCounter> for Detail {
    fn filter((_, feature): &(usize, DetailCounter)) -> bool {
        feature.count() > 0
    }

    fn map((id, feature): (usize, DetailCounter)) -> Self {
        let count = feature.count();
        let mnemonics = Count::map_items(feature.into_mnemonics());
        Self { id, count, mnemonics }
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
    type Result = Arr<Count<Feature>>;

    fn new() -> Self {
        let features = Arr::from(vec![0; FEATURE_COUNT]);
        Self { features }
    }

    fn add(&mut self, instruction: Instruction) {
        if instruction.is_invalid() {
            return;
        }

        for feature in instruction.cpuid_features() {
            self.features[*feature as usize] += 1;
        }
    }

    fn into_result(self) -> Self::Result {
        Count::map_items(self.features)
    }
}

pub struct TaskDetail {
    features: Arr<DetailCounter>,
    registers: Arr<u64>,
}

impl Task for TaskDetail {
    type Result = (Arr<Detail>, Arr<Count<Register>>);

    fn new() -> Self {
        let features = (0..FEATURE_COUNT).map(DetailCounter::new).collect();
        let registers = Arr::from(vec![0; REGISTER_ENUM_COUNT]);
        Self { features, registers }
    }

    fn add(&mut self, instruction: Instruction) {
        if instruction.is_invalid() {
            return;
        }

        let mnemonic = instruction.mnemonic() as usize;

        for feature in instruction.cpuid_features() {
            self.features[*feature as usize].add(mnemonic);
        }

        for op in 0..4 {
            let id = instruction.op_register(op) as usize;

            if id > 0 {
                self.registers[id] += 1;
            }
        }
    }

    fn into_result(self) -> Self::Result {
        let features = Detail::map_items(self.features);
        let registers = Count::map_items(self.registers);
        (features, registers)
    }
}

pub struct Decoder<T: Task> {
    bitness: u32,
    task: T,
}

impl<T: Task> Decoder<T> {
    pub fn new(bitness: u32, task: T) -> Self {
        Self { bitness, task }
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
