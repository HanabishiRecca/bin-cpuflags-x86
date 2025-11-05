use crate::types::Arr;
use iced_x86::{Code, CpuidFeature, Decoder, DecoderOptions, Instruction};
use std::{
    collections::HashSet,
    fmt,
    fs::File,
    io::{BufRead, BufReader, Result, Seek, SeekFrom},
};

const OPTIONS: u32 = DecoderOptions::NO_INVALID_CHECK;

pub trait Feature {
    fn new(id: CpuidFeature) -> Self;
    fn add(&mut self, code: Code);
    fn found(&self) -> bool;
    fn need_endln() -> bool;
}

pub struct FSimple {
    id: CpuidFeature,
    found: bool,
}

impl Feature for FSimple {
    fn new(id: CpuidFeature) -> Self {
        Self { id, found: false }
    }

    fn add(&mut self, _: Code) {
        self.found = true;
    }

    fn found(&self) -> bool {
        self.found
    }

    fn need_endln() -> bool {
        true
    }
}

impl fmt::Display for FSimple {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} ", self.id)
    }
}

pub struct FDetail {
    id: CpuidFeature,
    details: HashSet<Code>,
}

impl Feature for FDetail {
    fn new(id: CpuidFeature) -> Self {
        Self {
            id,
            details: HashSet::new(),
        }
    }

    fn add(&mut self, code: Code) {
        self.details.insert(code);
    }

    fn found(&self) -> bool {
        !self.details.is_empty()
    }

    fn need_endln() -> bool {
        false
    }
}

impl fmt::Display for FDetail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} : ", self.id)?;

        for code in &self.details {
            write!(f, "{:?} ", code.mnemonic())?;
        }

        writeln!(f)
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
                feature.add(instruction.code());
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

    pub fn features(&self) -> &[T] {
        &self.features
    }

    pub fn has_cpuid(&self) -> bool {
        self.features
            .get(CpuidFeature::CPUID as usize)
            .map(|i| i.found())
            .unwrap_or_default()
    }
}
