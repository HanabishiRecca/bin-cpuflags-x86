use cli::Config;
use iced_x86::{CpuidFeature, Decoder, DecoderOptions, Mnemonic};
use object::{
    Architecture, File as ObjFile, Object, ObjectSection, ReadCache, ReadRef, SectionKind,
};
use std::{
    collections::HashSet,
    env,
    fs::File,
    io::{Read, Seek, SeekFrom},
    process::ExitCode,
};

mod cli;
mod error;

use crate::{
    cli::OutputMode,
    error::{AppError, R},
};

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

macro_rules! check {
    ($b: expr, $e: expr $(,)?) => {
        ($b).then_some(()).ok_or($e)?
    };
}

fn read_header<'a>(data: impl ReadRef<'a>, output_mode: OutputMode) -> R<(u32, Vec<(u64, u64)>)> {
    let file = ObjFile::parse(data)?;
    let architecture = file.architecture();

    if output_mode > OutputMode::Quiet {
        println!("Format: {:?}", file.format());
        println!("Architecture: {architecture:?}");
    }

    check!(
        matches!(
            architecture,
            Architecture::X86_64 | Architecture::X86_64_X32 | Architecture::I386
        ),
        AppError::WrongArch,
    );

    let bitness = match architecture {
        Architecture::X86_64 => 64,
        _ => 32,
    };

    if output_mode > OutputMode::Normal {
        println!("Text sections: ");
    }

    let sections = file
        .sections()
        .filter_map(|s| match s.kind() {
            SectionKind::Text => {
                if output_mode > OutputMode::Normal {
                    println!(
                        "    {} => 0x{:x}, {} bytes",
                        s.name().unwrap_or_default(),
                        s.address(),
                        s.size()
                    );
                }
                s.file_range()
            }
            _ => None,
        })
        .collect();

    Ok((bitness, sections))
}

fn read_file(
    path: &str,
    details: bool,
    output_mode: OutputMode,
) -> R<([bool; CF_COUNT], Option<Vec<Detail>>)> {
    if output_mode > OutputMode::Normal {
        println!("Reading '{path}'...");
    }

    let mut fh = File::open(path)?;
    check!(!fh.metadata()?.file_type().is_dir(), AppError::WrongTarget);

    let (bitness, sections) = { read_header(&ReadCache::new(&fh), output_mode)? };
    check!(!sections.is_empty(), AppError::NoText);

    let mut found = [false; CF_COUNT];
    let mut details = details.then(|| vec![HashSet::new(); CF_COUNT]);
    let mut buffer = vec![0; sections.iter().map(|(_, size)| *size).max().unwrap_or(0) as usize];

    for (offset, size) in sections {
        let data = &mut buffer[..size as usize];
        fh.seek(SeekFrom::Start(offset))?;
        fh.read_exact(data)?;
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

    if details.is_none() && output_mode > OutputMode::Quiet {
        println!();
    }
}

fn print_help() {
    let bin = env::current_exe().ok();
    println!(
        include_str!("help.in"),
        PKG = env!("CARGO_PKG_NAME"),
        VER = env!("CARGO_PKG_VERSION"),
        BIN = (|| bin.as_ref()?.file_name()?.to_str())().unwrap_or(env!("CARGO_BIN_NAME")),
    );
}

fn run_app() -> R<()> {
    let config = cli::read_args(env::args().skip(1))?;

    match config {
        Some(Config {
            file_path: Some(path),
            details,
            output_mode,
        }) => {
            let (found, details) = read_file(&path, details, output_mode)?;
            print_features(&found, details.as_deref(), output_mode);
        }
        _ => print_help(),
    }

    Ok(())
}

fn main() -> ExitCode {
    run_app().err().map_or(ExitCode::SUCCESS, |e| {
        eprintln!("Error: {e}");
        ExitCode::FAILURE
    })
}
