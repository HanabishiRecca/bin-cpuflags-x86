mod binary;
mod cli;
mod decoder;
mod types;

use crate::{
    binary::Segment,
    cli::{Mode, Output},
    decoder::{Counter, Decoder, DetailCounter, FeatureCounter, Task, TaskCount, TaskDetail},
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

fn detect_cpuid(features: &[FeatureCounter]) {
    if features.iter().any(FeatureCounter::is_cpuid) {
        println!("Warning: CPUID usage detected, features could switch in runtime.");
    }
}

fn print_stats_note() {
    println!("Note: instructions that belong to multiple feature sets make counters overlap.");
}

fn decode<T: Task>(mut file: File, bitness: u32, segments: &[Segment]) -> Result<T> {
    let mut decoder = Decoder::new(bitness);

    for segment in segments {
        decoder.read(&mut file, segment.offset(), segment.size())?;
    }

    Ok(decoder.into_task())
}

fn print_features(features: Arr<FeatureCounter>) -> io::Result<()> {
    let mut stdout = io::stdout().lock();

    for feature in features {
        write!(stdout, "{} ", feature.name())?;
    }

    writeln!(stdout)
}

fn print_counters(mut counters: Arr<impl Counter>) -> io::Result<()> {
    counters.sort_unstable_by_key(|counter| Reverse(counter.count()));

    let nlen = counters.iter().map(Counter::name).map(str::len).max().unwrap_or(0);
    let total: u64 = counters.iter().map(Counter::count).sum();

    let mut stdout = io::stdout().lock();
    writeln!(stdout, "{:nlen$} {total}", "=")?;

    for counter in counters {
        let count = counter.count();
        let ratio = (count as f64 / total as f64) * 100.0;
        writeln!(stdout, "{:nlen$} {count} ({ratio:.2}%)", counter.name())?;
    }

    writeln!(stdout)
}

fn print_details(mut details: Arr<DetailCounter>) -> io::Result<()> {
    details.sort_unstable_by_key(|detail| Reverse(detail.count()));

    let total: u64 = details.iter().map(DetailCounter::count).sum();

    let mut stdout = io::stdout().lock();
    writeln!(stdout, "= {total}")?;
    writeln!(stdout)?;

    for detail in details {
        let count = detail.count();
        let ratio = (count as f64 / total as f64) * 100.0;
        writeln!(stdout, "{} {count} ({ratio:.2}%)", detail.name())?;

        let mut mnemonics = detail.into_mnemonics();
        mnemonics.sort_unstable_by_key(|mnemonic| Reverse(mnemonic.count()));

        let nlen = mnemonics.iter().map(Counter::name).map(str::len).max().unwrap_or(0);

        for mnemonic in mnemonics {
            let count = mnemonic.count();
            let ratio = (count as f64 / total as f64) * 100.0;
            writeln!(stdout, "    {:nlen$} {count} ({ratio:.2}%)", mnemonic.name())?;
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

    let binary = binary::parse(&file)?;

    if output > Output::Quiet {
        println!("Format: {:?}", binary.format());
        println!("Architecture: {:?}", binary.architecture());
    }

    let bitness = or!(binary.bitness(), WrongArch);
    let segments = binary.segments();
    test!(segments.is_empty(), NoText);

    if output > Output::Normal {
        println!("Text sections:");
        segments.iter().for_each(print_section);
    }

    use Mode::*;
    match mode {
        Detect => {
            let features = decode::<TaskCount>(file, bitness, segments)?.into_result();

            if output > Output::Quiet {
                detect_cpuid(&features);
                print!("Features: ");
            }

            print_features(features)?;
        }
        Stats => {
            let stats = decode::<TaskCount>(file, bitness, segments)?.into_result();

            if output > Output::Quiet {
                println!("----------");
                print_stats_note();
            }

            print_counters(stats)?;
        }
        Details => {
            let (features, registers) =
                decode::<TaskDetail>(file, bitness, segments)?.into_result();

            if output > Output::Quiet {
                println!();
                print_header("Instructions");
                print_stats_note();
            }

            print_details(features)?;

            if output > Output::Quiet {
                print_header("Registers");
            }

            print_counters(registers)?;
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
