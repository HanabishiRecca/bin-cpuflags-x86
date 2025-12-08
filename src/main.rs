mod binary;
mod cli;
mod decoder;
mod types;

use crate::{
    binary::{Binary, Segment},
    cli::{Mode, Output},
    decoder::{Decoder, Record, RecordF, Task, TaskCount, TaskDetail, TaskDetect},
    types::Arr,
};
use std::{
    cmp::Reverse,
    env, error, fmt,
    fs::File,
    io::{self, Write},
    process::ExitCode,
    result,
};

const DEFAULT_MODE: Mode = Mode::Detect;
const DEFAULT_OUTPUT: Output = Output::Normal;

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
        BIN_NAME = default!((|| bin.as_ref()?.file_name()?.to_str())(), env!("CARGO_BIN_NAME")),
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

fn print_header(text: &str) {
    let len = text.len() + 8;
    println!("{:-<len$}", "");
    println!("{text:^len$}");
    println!("{:-<len$}", "");
}

fn parse(file: &File, output: Output) -> Result<Binary> {
    let binary = binary::parse(file)?;

    if output > Output::Quiet {
        println!("Format: {:?}", binary.format());
        println!("Architecture: {:?}", binary.architecture());
    }

    let sections = binary.segments();
    test!(sections.is_empty(), NoText);

    if output > Output::Normal {
        println!("Text sections: ");
        sections.iter().for_each(print_section);
    }

    Ok(binary)
}

fn decode<T: Task>(mut file: File, binary: Binary) -> Result<T> {
    let mut decoder = Decoder::new(or!(binary.bitness(), WrongArch));

    for segment in binary.segments() {
        decoder.read(&mut file, segment.offset(), segment.size())?;
    }

    Ok(decoder.into_task())
}

fn print_detect(features: Arr<Record>) -> io::Result<()> {
    let mut stdout = io::stdout().lock();

    for feature in features {
        write!(stdout, "{} ", feature.name())?;
    }

    writeln!(stdout)
}

fn print_records(mut records: Arr<Record>) -> io::Result<()> {
    records.sort_unstable_by_key(|feature| Reverse(feature.count()));

    let nlen = records.iter().map(Record::name).map(str::len).max().unwrap_or(0);
    let total: u64 = records.iter().map(Record::count).sum();

    let mut stdout = io::stdout().lock();
    writeln!(stdout, "{:nlen$} {total}", "=")?;

    for register in records {
        let ratio = (register.count() as f64 / total as f64) * 100.0;
        writeln!(stdout, "{:nlen$} {} ({ratio:.2}%)", register.name(), register.count())?;
    }

    writeln!(stdout)
}

fn print_features(mut features: Arr<RecordF>) -> io::Result<()> {
    features.sort_unstable_by_key(|feature| Reverse(feature.count()));

    let total: u64 = features.iter().map(RecordF::count).sum();

    let mut stdout = io::stdout().lock();
    writeln!(stdout, "= {total}")?;
    writeln!(stdout)?;

    for feature in features {
        let ratio = (feature.count() as f64 / total as f64) * 100.0;
        writeln!(stdout, "{} {} ({ratio:.2}%)", feature.name(), feature.count())?;

        let mut mnemonics = feature.into_mnemonics();
        mnemonics.sort_unstable_by_key(|mnemonic| Reverse(mnemonic.count()));

        let nlen = mnemonics.iter().map(Record::name).map(str::len).max().unwrap_or(0);

        for mnemonic in mnemonics {
            let ratio = (mnemonic.count() as f64 / total as f64) * 100.0;
            writeln!(stdout, "    {:nlen$} {} ({ratio:.2}%)", mnemonic.name(), mnemonic.count())?;
        }

        writeln!(stdout)?;
    }

    Ok(())
}

fn run() -> Result<bool> {
    let Some(config) = cli::read_args(env::args().skip(1))? else {
        return Ok(true);
    };

    let Some(file_path) = config.file_path() else {
        return Ok(true);
    };

    let mode = default!(config.mode(), DEFAULT_MODE);
    let output = default!(config.output(), DEFAULT_OUTPUT);

    if output > Output::Normal {
        println!("Reading '{file_path}'...");
    }

    let file = File::open(file_path)?;
    test!(file.metadata()?.file_type().is_dir(), WrongTarget);

    let binary = parse(&file, output)?;

    use Mode::*;
    match mode {
        Detect => {
            let result = decode::<TaskDetect>(file, binary)?;

            if output > Output::Quiet {
                if result.has_cpuid() {
                    println!("Warning: CPUID usage detected, features could switch in runtime");
                }

                print!("Features: ");
            }

            print_detect(result.into_result())?;
        }
        Stats => {
            let result = decode::<TaskCount>(file, binary)?;

            if output > Output::Quiet {
                println!("----------");
            }

            print_records(result.into_result())?;
        }
        Details => {
            let (features, registers) = decode::<TaskDetail>(file, binary)?.into_result();

            if output > Output::Quiet {
                println!();
                print_header("Instructions");
            }

            print_features(features)?;

            if output > Output::Quiet {
                print_header("Registers");
            }

            print_records(registers)?;
        }
    };

    Ok(false)
}

fn main() -> ExitCode {
    match run() {
        Ok(help) => {
            help.then(print_help);
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}
