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
    id: CpuidFeature,
    count: usize,
}

impl FSimple {
    pub fn result(self) -> (CpuidFeature, usize) {
        (self.id, self.count)
    }
}

impl Feature for FSimple {
    fn new(id: CpuidFeature) -> Self {
        Self { id, count: 0 }
    }

    fn add(&mut self, _: Instruction) {
        self.count += 1;
    }

    fn found(&self) -> bool {
        self.count > 0
    }
}

pub struct FDetail {
    id: CpuidFeature,
    mnemonics: Vec<usize>,
}

impl FDetail {
    pub fn result(self) -> (CpuidFeature, Vec<usize>) {
        (self.id, self.mnemonics)
    }
}

impl Feature for FDetail {
    fn new(id: CpuidFeature) -> Self {
        Self {
            id,
            mnemonics: Vec::new(),
        }
    }

    fn add(&mut self, instruction: Instruction) {
        let mnemonic = instruction.mnemonic() as usize;
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
        Self {
            bitness,
            features: CpuidFeature::values().map(T::new).collect(),
        }
    }

    fn add(&mut self, instruction: Instruction) {
        if instruction.is_invalid() {
            return;
        }

        for id in instruction.cpuid_features() {
            if let Some(feature) = self.features.get_mut(*id as usize) {
                feature.add(instruction);
            }
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
        let has_cpuid = self
            .features
            .get(CpuidFeature::CPUID as usize)
            .map(|i| i.found())
            .unwrap_or_default();

        (self.features, has_cpuid)
    }
}
