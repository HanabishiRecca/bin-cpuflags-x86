mod error;

use crate::error::{AppError, R};
use iced_x86::{CpuidFeature, Decoder, DecoderOptions};
use object::{Architecture, File as ObjFile, Object, ObjectSection, ReadCache, SectionKind};
use std::{env, fs::File, process::ExitCode};

fn read_section(data: &[u8], bitness: u32, found: &mut [bool]) {
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

fn read_bin(path: &str) -> R<()> {
    let fh = File::open(path)?;
    check!(!fh.metadata()?.file_type().is_dir(), AppError::WrongTarget);

    let data = ReadCache::new(fh);
    let file = ObjFile::parse(&data)?;
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

    let mut found = vec![false; CpuidFeature::values().max().unwrap_or_default() as usize];
    let mut count = 0;

    for section in file.sections() {
        if section.kind() != SectionKind::Text {
            continue;
        }
        read_section(section.data()?, bitness, &mut found);
        count += 1;
    }

    check!(count > 0, AppError::NoText);
    println!("Detected features:");

    for feature in CpuidFeature::values() {
        if let Some(true) = found.get(feature as usize) {
            print!("{feature:?} ");
        }
    }

    println!();
    Ok(())
}

fn print_help() {
    let bin = env::current_exe().ok();
    println!(
        include_str!("help.in"),
        BIN_NAME = (|| bin.as_ref()?.file_name()?.to_str())().unwrap_or(env!("CARGO_BIN_NAME"))
    );
}

fn run_app() -> R<()> {
    if let Some(path) = env::args().nth(1) {
        return read_bin(&path);
    }

    print_help();
    Ok(())
}

fn main() -> ExitCode {
    run_app().err().map_or(ExitCode::SUCCESS, |e| {
        println!("Error: {e}");
        ExitCode::FAILURE
    })
}
