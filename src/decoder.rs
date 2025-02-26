use crate::types::Arr;
use iced_x86::{CpuidFeature, Decoder, DecoderOptions, Mnemonic};
use std::{
    collections::HashSet,
    fs::File,
    io::{Read, Result, Seek, SeekFrom},
};

pub struct FeatureInfo {
    feature: CpuidFeature,
    found: bool,
    details: HashSet<Mnemonic>,
}

impl FeatureInfo {
    pub fn new(feature: CpuidFeature) -> Self {
        Self {
            feature,
            found: false,
            details: HashSet::new(),
        }
    }

    pub fn feature(&self) -> CpuidFeature {
        self.feature
    }

    pub fn found(&self) -> bool {
        self.found
    }

    pub fn details(&self) -> impl Iterator<Item = &Mnemonic> {
        self.details.iter()
    }
}

pub struct Task<'a> {
    file: &'a File,
    bitness: u32,
    features: Arr<FeatureInfo>,
    details: bool,
}

impl<'a> Task<'a> {
    pub fn new(file: &'a File, bitness: u32, details: bool) -> Self {
        Self {
            file,
            bitness,
            features: CpuidFeature::values().map(FeatureInfo::new).collect(),
            details,
        }
    }

    pub fn read(&mut self, (offset, size): (u64, u64)) -> Result<()> {
        let mut data = Arr::from(vec![0; size as usize]);
        self.file.seek(SeekFrom::Start(offset))?;
        self.file.read_exact(&mut data)?;
        let decoder = Decoder::new(self.bitness, &data, DecoderOptions::NO_INVALID_CHECK);

        macro_rules! body {
            ($($d: expr)?) => {
                for instruction in decoder {
                    for &feature in instruction.cpuid_features() {
                        let Some(info) = self.features.get_mut(feature as usize) else {
                            continue;
                        };
                        info.found = true;
                        $(info.details.insert(instruction.mnemonic());$d)?
                    }
                }
            };
        }

        match self.details {
            true => body!({}),
            _ => body!(),
        }

        Ok(())
    }

    pub fn features(&self) -> &[FeatureInfo] {
        &self.features
    }

    pub fn cpuid(&self) -> bool {
        self.features
            .get(CpuidFeature::CPUID as usize)
            .map(|i| i.found())
            .unwrap_or_default()
    }
}
