mod binary;
mod cli;
mod decoder;
mod types;

use crate::{
    binary::Segment,
    cli::OutputMode,
    decoder::{Feature, Task},
};
use std::{env, fs::File, process::ExitCode};

const DEFAULT_SHOW_DETAILS: bool = false;
const DEFAULT_OUTPUT_MODE: OutputMode = OutputMode::Normal;

macro_rules! default {
    ($option: expr, $default: expr) => {
        match $option {
            Some(value) => value,
            _ => $default,
        }
    };
}

#[derive(Debug)]
enum Error {
    WrongTarget,
    WrongArch,
    NoText,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use Error::*;
        match self {
            WrongTarget => write!(f, "Should target a file"),
            WrongArch => write!(f, "Unsupported architecture"),
            NoText => write!(f, "No 'text' sections found in the file"),
        }
    }
}

macro_rules! or {
    ($o: expr, $e: expr $(,)?) => {{
        use Error::*;
        ($o).ok_or($e)?
    }};
}

macro_rules! test {
    ($b: expr, $e: expr $(,)?) => {
        or!((!$b).then_some(()), $e)
    };
}

fn print_help() {
    let bin = env::current_exe().ok();
    println!(
        include_str!("help.in"),
        PKG = env!("CARGO_PKG_NAME"),
        VER = env!("CARGO_PKG_VERSION"),
        BIN_NAME = default!(
            (|| bin.as_ref()?.file_name()?.to_str())(),
            env!("CARGO_BIN_NAME")
        ),
    );
}

fn print_section(segment: &Segment) {
    println!(
        "    {} => 0x{:x}, {} bytes",
        segment.name().unwrap_or_default(),
        segment.offset(),
        segment.size(),
    );
}

fn print_feature(feature: &Feature, details: bool) {
    if !feature.found() {
        return;
    }

    print!("{:?} ", feature.id());

    if !details {
        return;
    }

    print!(": ");

    for code in feature.details() {
        print!("{:?} ", code.mnemonic());
    }

    println!();
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let Some(config) = cli::read_args(env::args().skip(1))? else {
        print_help();
        return Ok(());
    };
    let Some(file_path) = config.file_path() else {
        print_help();
        return Ok(());
    };
    let show_details = default!(config.show_details(), DEFAULT_SHOW_DETAILS);
    let output_mode = default!(config.output_mode(), DEFAULT_OUTPUT_MODE);

    if output_mode > OutputMode::Normal {
        println!("Reading '{file_path}'...");
    }

    let file = File::open(file_path)?;
    test!(file.metadata()?.file_type().is_dir(), WrongTarget);
    let binary = binary::parse(&file)?;

    if output_mode > OutputMode::Quiet {
        println!("Format: {:?}", binary.format());
        println!("Architecture: {:?}", binary.architecture());
    }

    let sections = binary.segments();
    test!(sections.is_empty(), NoText);

    if output_mode > OutputMode::Normal {
        println!("Text sections: ");
        sections.iter().for_each(print_section);
    }

    let mut task = Task::new(&file, or!(binary.bitness(), WrongArch), show_details);

    for segment in sections {
        task.read(segment.offset(), segment.size())?;
    }

    if output_mode > OutputMode::Quiet {
        println!("Features: ");
    }

    for feature in task.features() {
        print_feature(feature, show_details);
    }

    if output_mode > OutputMode::Quiet {
        if !show_details {
            println!();
        }

        if task.cpuid() {
            println!("Warning: CPUID usage detected. Features could switch in runtime.")
        }
    }

    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
        _ => ExitCode::SUCCESS,
    }
}
