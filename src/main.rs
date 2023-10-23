mod error;

use crate::error::{AppError, R};
use iced_x86::{CpuidFeature, Decoder, DecoderOptions};
use object::{
    Architecture, File as ObjFile, Object, ObjectSection, ReadCache, ReadRef, SectionKind,
};
use std::{
    env,
    fs::File,
    io::{Read, Seek, SeekFrom},
    process::ExitCode,
};

/// Should be bigger or equal to `IcedConstants::CPUID_FEATURE_ENUM_COUNT`.
/// The crate does not export it unfortunatelty.
const CF_COUNT: usize = 256;

fn decode(data: &[u8], bitness: u32, found: &mut [bool]) {
    let decoder = Decoder::new(bitness, data, DecoderOptions::NO_INVALID_CHECK);

    for instruction in decoder {
        for &feature in instruction.cpuid_features() {
            if let Some(flag) = found.get_mut(feature as usize) {
                *flag = true;
            }
        }
    }
}

macro_rules! check {
    ($b: expr, $e: expr $(,)?) => {
        ($b).then_some(()).ok_or($e)?
    };
}

fn read_header<'a>(data: impl ReadRef<'a>) -> R<(u32, Vec<(u64, u64)>)> {
    let file = ObjFile::parse(data)?;
    println!("Format: {:?}", file.format());

    let architecture = file.architecture();
    println!("Architecture: {architecture:?}");

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

    let sections = file
        .sections()
        .filter_map(|s| match s.kind() {
            SectionKind::Text => s.file_range(),
            _ => None,
        })
        .collect();

    Ok((bitness, sections))
}

fn read_file(path: &str) -> R<[bool; CF_COUNT]> {
    let mut fh = File::open(path)?;
    check!(!fh.metadata()?.file_type().is_dir(), AppError::WrongTarget);

    let (bitness, sections) = { read_header(&ReadCache::new(&fh))? };
    check!(!sections.is_empty(), AppError::NoText);

    let mut found = [false; CF_COUNT];
    let mut buffer = vec![0; sections.iter().map(|(_, size)| *size).max().unwrap_or(0) as usize];

    for (offset, size) in sections {
        let data = &mut buffer[..size as usize];
        fh.seek(SeekFrom::Start(offset))?;
        fh.read_exact(data)?;
        decode(data, bitness, &mut found);
    }

    Ok(found)
}

fn print_features(found: &[bool]) {
    print!("Features: ");

    for feature in CpuidFeature::values() {
        if let Some(true) = found.get(feature as usize) {
            print!("{feature:?} ");
        }
    }

    println!();
}

fn print_help() {
    let bin = env::current_exe().ok();
    println!(
        include_str!("help.in"),
        BIN_NAME = (|| bin.as_ref()?.file_name()?.to_str())().unwrap_or(env!("CARGO_BIN_NAME"))
    );
}

fn run_app() -> R<()> {
    match env::args().nth(1) {
        Some(path) => print_features(&read_file(&path)?),
        _ => print_help(),
    }

    Ok(())
}

fn main() -> ExitCode {
    run_app().err().map_or(ExitCode::SUCCESS, |e| {
        println!("Error: {e}");
        ExitCode::FAILURE
    })
}
