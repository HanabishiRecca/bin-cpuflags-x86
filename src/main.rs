use std::{env, fs::File, process::ExitCode};

mod binary;
mod cli;
mod decoder;
mod error;

use crate::{
    cli::{Config, OutputMode},
    error::{AppError, R},
};

#[macro_export]
macro_rules! check {
    ($b: expr, $e: expr $(,)?) => {
        ($b).then_some(()).ok_or($e)?
    };
}

fn run_for(path: &str, details: bool, output_mode: OutputMode) -> R<()> {
    if output_mode > OutputMode::Normal {
        println!("Reading '{path}'...");
    }

    let mut file = File::open(path)?;
    check!(
        !file.metadata()?.file_type().is_dir(),
        AppError::WrongTarget
    );

    let (sections, bitness) = binary::parse(&file, output_mode)?;
    check!(!sections.is_empty(), AppError::NoText);
    decoder::run(&mut file, &sections, bitness, details, output_mode)?;
    Ok(())
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
        }) => run_for(&path, details, output_mode)?,
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
