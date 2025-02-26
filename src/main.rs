mod binary;
mod cli;
mod decoder;
mod types;

use crate::{cli::OutputMode, types::Arr};
use std::{
    env,
    fs::File,
    io::{Read, Seek, SeekFrom},
    process::ExitCode,
};

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

    let mut file = File::open(file_path)?;
    test!(file.metadata()?.file_type().is_dir(), WrongTarget);

    let binary = binary::parse(&file)?;

    if output_mode > OutputMode::Quiet {
        println!("Format: {:?}", binary.format());
        println!("Architecture: {:?}", binary.arch());
    }

    let sections = binary.sections();
    test!(sections.is_empty(), NoText);

    if output_mode > OutputMode::Normal {
        println!("Text sections: ");

        for section in sections {
            println!(
                "    {} => 0x{:x}, {} bytes",
                section.name().unwrap_or_default(),
                section.address(),
                section.size()
            );
        }
    }

    let bitness = or!(binary.bitness(), WrongArch);
    let mut info = decoder::new_infos();

    for section in sections {
        let (offset, size) = section.range();
        let mut data = Arr::from(vec![0; size as usize]);
        file.seek(SeekFrom::Start(offset))?;
        file.read_exact(&mut data)?;
        decoder::decode(&data, bitness, &mut info, show_details);
    }

    if output_mode > OutputMode::Quiet {
        println!("Features: ");
    }

    for feature in decoder::features() {
        let Some(i) = info.get(feature as usize) else {
            continue;
        };

        if !i.found() {
            continue;
        }

        print!("{feature:?} ");

        if !show_details {
            continue;
        }

        print!(": ");

        for d in i.details() {
            print!("{d:?} ");
        }

        println!();
    }

    if output_mode > OutputMode::Quiet {
        if !show_details {
            println!();
        }

        if decoder::has_cpuid(&info) {
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
