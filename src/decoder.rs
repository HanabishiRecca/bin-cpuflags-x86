mod strings;

use crate::types::Arr;
use iced_x86::{CpuidFeature, Decoder, DecoderOptions, Instruction};
use std::{
    fs::File,
    io::{BufRead, BufReader, Result, Seek, SeekFrom},
};

const OPTIONS: u32 = DecoderOptions::NO_INVALID_CHECK;

pub trait Feature {
    fn new(id: CpuidFeature) -> Self;
    fn add(&mut self, instruction: Instruction);
    fn found(&self) -> bool;
}

pub struct FSimple {
    id: usize,
    count: u64,
}

impl FSimple {
    pub fn name(&self) -> &'static str {
        strings::FEATURE[self.id]
    }

    pub fn count(&self) -> u64 {
        self.count
    }
}

impl Feature for FSimple {
    fn new(id: CpuidFeature) -> Self {
        Self { id: id as usize, count: 0 }
    }

    fn add(&mut self, _: Instruction) {
        self.count += 1;
    }

    fn found(&self) -> bool {
        self.count > 0
    }
}

pub struct Mnemonic {
    id: usize,
    count: u64,
}

impl Mnemonic {
    fn new(id: usize) -> Self {
        Self { id, count: 1 }
    }

    fn id(&self) -> usize {
        self.id
    }

    fn inc(&mut self) {
        self.count += 1;
    }

    pub fn name(&self) -> &'static str {
        strings::MNEMONIC[self.id]
    }

    pub fn count(&self) -> u64 {
        self.count
    }
}

pub struct FDetail {
    id: usize,
    count: u64,
    mnemonics: Vec<Mnemonic>,
}

impl FDetail {
    pub fn name(&self) -> &'static str {
        strings::FEATURE[self.id]
    }

    pub fn count(&self) -> u64 {
        self.count
    }

    pub fn into_mnemonics(self) -> Vec<Mnemonic> {
        self.mnemonics
    }
}

impl Feature for FDetail {
    fn new(id: CpuidFeature) -> Self {
        Self { id: id as usize, count: 0, mnemonics: Vec::new() }
    }

    fn add(&mut self, instruction: Instruction) {
        self.count += 1;
        let id = instruction.mnemonic() as usize;

        match self.mnemonics.iter_mut().find(|mnemonic| mnemonic.id() == id) {
            Some(mnemonic) => mnemonic.inc(),
            _ => self.mnemonics.push(Mnemonic::new(id)),
        }
    }

    fn found(&self) -> bool {
        self.count > 0
    }
}

pub struct Task<T: Feature> {
    bitness: u32,
    features: Arr<T>,
}

impl<T: Feature> Task<T> {
    pub fn new(bitness: u32) -> Self {
        let features = CpuidFeature::values().map(T::new).collect();
        Self { bitness, features }
    }

    fn add(&mut self, instruction: Instruction) {
        if instruction.is_invalid() {
            return;
        }

        for id in instruction.cpuid_features() {
            self.features[*id as usize].add(instruction);
        }
    }

    pub fn read(&mut self, file: &mut File, offset: u64, size: u64) -> Result<()> {
        file.seek(SeekFrom::Start(offset))?;

        let mut reader = BufReader::with_capacity(size as usize, file);
        let decoder = Decoder::new(self.bitness, reader.fill_buf()?, OPTIONS);

        for instruction in decoder {
            self.add(instruction);
        }

        Ok(())
    }

    pub fn has_cpuid(&self) -> bool {
        self.features[CpuidFeature::CPUID as usize].found()
    }

    pub fn into_features(self) -> Arr<T> {
        self.features.into_iter().filter(T::found).collect()
    }
}
