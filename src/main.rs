mod error;

use crate::error::{AppError, R};
use iced_x86::{CpuidFeature, Decoder, DecoderOptions};
use object::{Architecture, File, Object, ObjectSection, ReadCache};
use std::{env, fs, process::ExitCode};

fn read_bin(path: &str) -> R<()> {
    let data = ReadCache::new(fs::File::open(path)?);
    let file = File::parse(&data)?;
    println!("Format: {:?}", file.format());

    let architecture = file.architecture();
    println!("Architecture: {architecture:?}");

    matches!(
        architecture,
        Architecture::X86_64 | Architecture::X86_64_X32 | Architecture::I386
    )
    .then_some(())
    .ok_or(AppError::WrongArch)?;

    let mut decoder = Decoder::with_ip(
        match architecture {
            Architecture::X86_64 => 64,
            _ => 32,
        },
        file.section_by_name(".text")
            .ok_or(AppError::NoText)?
            .data()?,
        0,
        DecoderOptions::NO_INVALID_CHECK,
    );

    let mut found = vec![false; CpuidFeature::values().max().unwrap_or_default() as usize];

    for instruction in decoder.iter() {
        for &feature in instruction.cpuid_features() {
            if let Some(flag) = found.get_mut(feature as usize) {
                *flag = true;
            }
        }
    }

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
