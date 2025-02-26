use iced_x86::{CpuidFeature, Decoder, DecoderOptions, Mnemonic};
use std::{array, collections::HashSet};

/// Should be bigger or equal to `IcedConstants::CPUID_FEATURE_ENUM_COUNT`.
/// The crate does not export it unfortunatelty.
const CF_COUNT: usize = 256;

#[derive(Default)]
pub struct Info {
    found: bool,
    details: HashSet<Mnemonic>,
}

impl Info {
    pub fn found(&self) -> bool {
        self.found
    }

    pub fn details(&self) -> impl Iterator<Item = &Mnemonic> {
        self.details.iter()
    }
}

pub fn new_infos() -> [Info; CF_COUNT] {
    array::from_fn(|_| Info::default())
}

pub fn decode(data: &[u8], bitness: u32, infos: &mut [Info; CF_COUNT], need_details: bool) {
    let decoder = Decoder::new(bitness, data, DecoderOptions::NO_INVALID_CHECK);

    macro_rules! body {
        ($($d: expr)?) => {
            for instruction in decoder {
                for &feature in instruction.cpuid_features() {
                    let Some(info) = infos.get_mut(feature as usize) else {
                        continue;
                    };
                    info.found = true;
                    $(info.details.insert(instruction.mnemonic());$d)?
                }
            }
        };
    }

    match need_details {
        true => body!({}),
        _ => body!(),
    }
}

pub fn features() -> impl Iterator<Item = CpuidFeature> {
    CpuidFeature::values()
}

pub fn has_cpuid(features: &[Info; CF_COUNT]) -> bool {
    features
        .get(CpuidFeature::CPUID as usize)
        .map(|i| i.found())
        .unwrap_or_default()
}
