mod strings;

use crate::types::Arr;
use iced_x86::{CpuidFeature, Decoder as Iced, DecoderOptions, Instruction, Register};
use std::{
    fs::File,
    io::{BufRead, BufReader, Result, Seek, SeekFrom},
};

/// Keep in sync with `IcedConstants::CPUID_FEATURE_ENUM_COUNT`!
const FEATURE_COUNT: usize = 178;
/// Keep in sync with `IcedConstants::MNEMONIC_ENUM_COUNT`!
const MNEMONIC_ENUM_COUNT: usize = 1894;
/// Keep in sync with `IcedConstants::REGISTER_ENUM_COUNT`!
const REGISTER_ENUM_COUNT: usize = 256;

const OPTIONS: u32 = DecoderOptions::NO_INVALID_CHECK;

pub struct Record {
    name: &'static str,
    count: u64,
}

impl Record {
    fn filter((_, count): &(usize, u64)) -> bool {
        *count > 0
    }

    fn map_from(input: Arr<u64>, names: &[&'static str]) -> Arr<Self> {
        let map = |(id, count): (usize, u64)| Self { name: names[id], count };
        input.into_iter().enumerate().filter(Record::filter).map(map).collect()
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn count(&self) -> u64 {
        self.count
    }
}

pub trait Task {
    type Result;
    fn new() -> Self;
    fn add(&mut self, instruction: Instruction);
    fn into_result(self) -> Self::Result;
}

pub struct TaskDetect {
    features: Arr<u64>,
}

impl TaskDetect {
    pub fn has_cpuid(&self) -> bool {
        self.features[CpuidFeature::CPUID as usize] > 0
    }
}

impl Task for TaskDetect {
    type Result = Arr<Record>;

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
        Record::map_from(self.features, &strings::FEATURE)
    }
}

pub struct TaskCount {
    features: Arr<u64>,
}

impl Task for TaskCount {
    type Result = Arr<Record>;

    fn new() -> Self {
        let features = Arr::from(vec![0; FEATURE_COUNT]);
        Self { features }
    }

    fn add(&mut self, instruction: Instruction) {
        if instruction.is_invalid() {
            return;
        }

        self.features[instruction.cpuid_features()[0] as usize] += 1;
    }

    fn into_result(self) -> Self::Result {
        Record::map_from(self.features, &strings::FEATURE)
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

    fn into_mnemonics(self) -> Arr<Record> {
        Record::map_from(self.mnemonics, &strings::MNEMONIC)
    }
}

pub struct RecordF {
    name: &'static str,
    count: u64,
    mnemonics: Arr<Record>,
}

impl RecordF {
    fn map((id, feature): (usize, Feature)) -> Option<Self> {
        if feature.count == 0 {
            return None;
        }

        Some(Self {
            name: strings::FEATURE[id],
            count: feature.count,
            mnemonics: feature.into_mnemonics(),
        })
    }

    fn map_from(input: Arr<Feature>) -> Arr<Self> {
        input.into_iter().enumerate().filter_map(Self::map).collect()
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn count(&self) -> u64 {
        self.count
    }

    pub fn into_mnemonics(self) -> Arr<Record> {
        self.mnemonics
    }
}

pub struct TaskDetail {
    features: Arr<Feature>,
    registers: Arr<u64>,
}

impl Task for TaskDetail {
    type Result = (Arr<RecordF>, Arr<Record>);

    fn new() -> Self {
        let features = (0..FEATURE_COUNT).map(Feature::new).collect();
        let registers = Arr::from(vec![0; REGISTER_ENUM_COUNT]);
        Self { features, registers }
    }

    fn add(&mut self, instruction: Instruction) {
        if instruction.is_invalid() {
            return;
        }

        let feature = &mut self.features[instruction.cpuid_features()[0] as usize];
        feature.add(instruction.mnemonic() as usize);

        for op in 0..4 {
            let register = instruction.op_register(op);

            if register != Register::None {
                self.registers[register as usize] += 1;
            }
        }
    }

    fn into_result(self) -> Self::Result {
        let features = RecordF::map_from(self.features);
        let registers = Record::map_from(self.registers, &strings::REGISTER);
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

    pub fn read(&mut self, file: &mut File, offset: u64, size: u64) -> Result<()> {
        file.seek(SeekFrom::Start(offset))?;

        let mut reader = BufReader::with_capacity(size as usize, file);
        let decoder = Iced::new(self.bitness, reader.fill_buf()?, OPTIONS);

        for instruction in decoder {
            self.task.add(instruction);
        }

        Ok(())
    }

    pub fn into_task(self) -> T {
        self.task
    }
}
