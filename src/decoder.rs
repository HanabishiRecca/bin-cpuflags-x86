use crate::types::Arr;
use iced_x86::{Code, CpuidFeature, Decoder, DecoderOptions};
use std::{
    collections::HashSet,
    fs::File,
    io::{BufRead, BufReader, Result, Seek, SeekFrom},
};

const OPTIONS: u32 = DecoderOptions::NO_INVALID_CHECK;

pub struct Feature {
    id: CpuidFeature,
    found: bool,
    details: HashSet<Code>,
}

impl Feature {
    pub fn new(id: CpuidFeature) -> Self {
        Self {
            id,
            found: false,
            details: HashSet::new(),
        }
    }

    pub fn id(&self) -> CpuidFeature {
        self.id
    }

    pub fn found(&self) -> bool {
        self.found
    }

    pub fn details(&self) -> impl Iterator<Item = &Code> {
        self.details.iter()
    }

    fn mark(&mut self) {
        self.found = true;
    }

    fn add(&mut self, code: Code) {
        self.details.insert(code);
    }
}

pub struct Task<'a> {
    file: &'a File,
    bitness: u32,
    details: bool,
    features: Arr<Feature>,
}

impl<'a> Task<'a> {
    pub fn new(file: &'a File, bitness: u32, details: bool) -> Self {
        Self {
            file,
            bitness,
            details,
            features: CpuidFeature::values().map(Feature::new).collect(),
        }
    }

    pub fn read(&mut self, offset: u64, size: u64) -> Result<()> {
        self.file.seek(SeekFrom::Start(offset))?;
        let mut reader = BufReader::with_capacity(size as usize, self.file);
        let decoder = Decoder::new(self.bitness, reader.fill_buf()?, OPTIONS);

        macro_rules! body {
            ($($d: expr)?) => {{
                for instruction in decoder {
                    for id in instruction.cpuid_features() {
                        let Some(feature) = self.features.get_mut(*id as usize) else {
                            continue;
                        };
                        feature.mark();
                        $(feature.add(instruction.code());$d)?
                    }
                }
                Ok(())
            }};
        }

        if self.details { body!({}) } else { body!() }
    }

    pub fn features(&self) -> &[Feature] {
        &self.features
    }

    pub fn cpuid(&self) -> bool {
        self.features
            .get(CpuidFeature::CPUID as usize)
            .map(|i| i.found())
            .unwrap_or_default()
    }
}
