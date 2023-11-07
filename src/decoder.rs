use iced_x86::{CpuidFeature, Decoder, DecoderOptions, Mnemonic};
use std::{
    collections::HashSet,
    fs::File,
    io::{Read, Seek, SeekFrom},
};

use crate::{cli::OutputMode, error::R};

/// Should be bigger or equal to `IcedConstants::CPUID_FEATURE_ENUM_COUNT`.
/// The crate does not export it unfortunatelty.
const CF_COUNT: usize = 256;

type Detail = HashSet<Mnemonic>;

fn decode(data: &[u8], bitness: u32, found: &mut [bool], details: Option<&mut [Detail]>) {
    let decoder = Decoder::new(bitness, data, DecoderOptions::NO_INVALID_CHECK);

    macro_rules! body {
        ($($d: expr)?) => {
            for instruction in decoder {
                for &feature in instruction.cpuid_features() {
                    let index = feature as usize;
                    if let Some(flag) = found.get_mut(index) {
                        *flag = true;
                        $(if let Some(d) = $d.get_mut(index) {
                            d.insert(instruction.mnemonic());
                        })?
                    }
                }
            }
        };
    }

    match details {
        Some(d) => body!(d),
        _ => body!(),
    }
}

fn read_file(
    file: &mut File,
    sections: &[(u64, u64)],
    bitness: u32,
    details: bool,
) -> R<([bool; CF_COUNT], Option<Vec<Detail>>)> {
    let mut found = [false; CF_COUNT];
    let mut details = details.then(|| vec![HashSet::new(); CF_COUNT]);
    let mut buffer = vec![0; sections.iter().map(|(_, size)| *size).max().unwrap_or(0) as usize];

    for &(offset, size) in sections {
        let data = &mut buffer[..size as usize];
        file.seek(SeekFrom::Start(offset))?;
        file.read_exact(data)?;
        decode(data, bitness, &mut found, details.as_deref_mut());
    }

    Ok((found, details))
}

fn print_features(found: &[bool], details: Option<&[Detail]>, output_mode: OutputMode) {
    if output_mode > OutputMode::Quiet {
        print!("Features: ");

        if details.is_some() {
            println!();
        }
    }

    macro_rules! body {
        ($($d: expr)?) => {{
            for feature in CpuidFeature::values() {
                let index = feature as usize;
                if let Some(true) = found.get(index) {
                    print!("{feature:?} ");
                    $(if let Some(d) = $d.get(index) {
                        print!(": ");
                        for m in d {
                            print!("{m:?} ");
                        }
                        println!();
                    })?
                }
            }
        }};
    }

    match details {
        Some(d) => body!(d),
        _ => body!(),
    }

    if output_mode > OutputMode::Quiet {
        if details.is_none() {
            println!();
        }

        if let Some(true) = found.get(CpuidFeature::CPUID as usize) {
            println!("Warning: CPUID usage detected. The program can switch instruction sets in runtime.")
        }
    }
}

pub fn run(
    file: &mut File,
    sections: &[(u64, u64)],
    bitness: u32,
    details: bool,
    output_mode: OutputMode,
) -> R<()> {
    let (found, details) = read_file(file, sections, bitness, details)?;
    print_features(&found, details.as_deref(), output_mode);
    Ok(())
}
