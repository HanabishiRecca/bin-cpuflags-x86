mod strings;

use crate::types::Arr;
use iced_x86::{CpuidFeature, Decoder, DecoderOptions, Instruction};
use std::{
    fmt,
    fs::File,
    io::{BufRead, BufReader, Result, Seek, SeekFrom},
};

const OPTIONS: u32 = DecoderOptions::NO_INVALID_CHECK;

pub trait Feature {
    fn new(id: CpuidFeature) -> Self;
    fn add(&mut self, instruction: Instruction);
    fn found(&self) -> bool;
}

#[derive(PartialEq)]
pub struct Id(usize);

impl Id {
    pub fn name(&self) -> &'static str {
        strings::FEATURE[self.0]
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.name().fmt(f)
    }
}

pub struct FSimple {
    id: Id,
    count: u64,
}

impl FSimple {
    pub fn result(self) -> (Id, u64) {
        (self.id, self.count)
    }
}

impl Feature for FSimple {
    fn new(id: CpuidFeature) -> Self {
        Self { id: Id(id as usize), count: 0 }
    }

    fn add(&mut self, _: Instruction) {
        self.count += 1;
    }

    fn found(&self) -> bool {
        self.count > 0
    }
}

#[derive(PartialEq)]
pub struct Mnemonic(usize);

impl Mnemonic {
    pub fn name(&self) -> &'static str {
        strings::MNEMONIC[self.0]
    }
}

impl fmt::Display for Mnemonic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.name().fmt(f)
    }
}

pub struct FDetail {
    id: Id,
    mnemonics: Vec<Mnemonic>,
}

impl FDetail {
    pub fn result(self) -> (Id, Vec<Mnemonic>) {
        (self.id, self.mnemonics)
    }
}

impl Feature for FDetail {
    fn new(id: CpuidFeature) -> Self {
        Self { id: Id(id as usize), mnemonics: Vec::new() }
    }

    fn add(&mut self, instruction: Instruction) {
        let mnemonic = Mnemonic(instruction.mnemonic() as usize);
        if !self.mnemonics.contains(&mnemonic) {
            self.mnemonics.push(mnemonic);
        }
    }

    fn found(&self) -> bool {
        !self.mnemonics.is_empty()
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

    pub fn result(self) -> (Arr<T>, bool) {
        let has_cpuid = self.features[CpuidFeature::CPUID as usize].found();
        (self.features, has_cpuid)
    }
}
