mod binary;
mod cli;
mod decoder;
mod types;

use crate::{
    binary::{Binary, Segment},
    cli::{DecoderMode, OutputMode},
    decoder::{FDetail, FSimple, Feature, Task},
};
use std::{env, error, fmt, fs::File, process::ExitCode, result};

const DEFAULT_DECODER_MODE: DecoderMode = DecoderMode::Simple;
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

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match self {
            WrongTarget => write!(f, "Should target a file"),
            WrongArch => write!(f, "Unsupported architecture"),
            NoText => write!(f, "No 'text' sections found in the file"),
        }
    }
}

type Result<T> = result::Result<T, Box<dyn error::Error>>;

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

fn parse(file: &File, output_mode: OutputMode) -> Result<Binary> {
    let binary = binary::parse(file)?;

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

    Ok(binary)
}

fn decode<T: Feature + fmt::Display>(
    file: &mut File,
    binary: &Binary,
    output_mode: OutputMode,
) -> Result<()> {
    let mut task = Task::<T>::new(or!(binary.bitness(), WrongArch));

    for segment in binary.segments() {
        task.read(file, segment.offset(), segment.size())?;
    }

    if output_mode > OutputMode::Quiet {
        println!("Features: ");
    }

    for feature in task.features() {
        if feature.found() {
            print!("{feature}");
        }
    }

    if output_mode > OutputMode::Quiet {
        if T::need_endln() {
            println!();
        }

        if task.has_cpuid() {
            println!("Warning: CPUID usage detected. Features could switch in runtime.");
        }
    }

    Ok(())
}

fn run() -> Result<()> {
    let Some(config) = cli::read_args(env::args().skip(1))? else {
        print_help();
        return Ok(());
    };
    let Some(file_path) = config.file_path() else {
        print_help();
        return Ok(());
    };
    let decoder_mode = default!(config.decoder_mode(), DEFAULT_DECODER_MODE);
    let output_mode = default!(config.output_mode(), DEFAULT_OUTPUT_MODE);

    if output_mode > OutputMode::Normal {
        println!("Reading '{file_path}'...");
    }

    let mut file = File::open(file_path)?;
    test!(file.metadata()?.file_type().is_dir(), WrongTarget);
    let binary = parse(&file, output_mode)?;

    use DecoderMode::*;
    match decoder_mode {
        Simple => decode::<FSimple>(&mut file, &binary, output_mode),
        Detail => decode::<FDetail>(&mut file, &binary, output_mode),
    }
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
