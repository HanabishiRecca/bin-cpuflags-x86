mod binary;
mod cli;
mod decoder;
mod types;

use crate::{
    binary::Segment,
    cli::{Mode, Output},
    decoder::{Counter, Decoder, DetailCounter, FeatureCounter, Task, TaskCount, TaskDetail},
};
use std::{
    env, error, fmt,
    fs::File,
    io::{self, StdoutLock, Write},
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

fn print_segment(segment: &Segment) {
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

fn decode<T: Task>(mut file: File, bitness: u32, segments: &[Segment]) -> io::Result<T> {
    let mut decoder = Decoder::new(bitness);

    for segment in segments {
        decoder.read(&mut file, segment.offset(), segment.size())?;
    }

    Ok(decoder.into_task())
}

fn print_features(features: &[FeatureCounter]) -> io::Result<()> {
    let mut stdout = io::stdout().lock();

    for feature in features {
        write!(stdout, "{} ", feature.name())?;
    }

    writeln!(stdout)
}

fn print_counters(
    counters: &[impl Counter], stdout: &mut StdoutLock, total: u64, tab: usize,
) -> io::Result<()> {
    let nlen = counters.iter().map(Counter::name).map(str::len).max().unwrap_or(0);

    for counter in counters {
        let count = counter.count();
        let ratio = (count as f64 / total as f64) * 100.0;
        writeln!(stdout, "{:tab$}{:nlen$} {count} ({ratio:.2}%)", "", counter.name())?;
    }

    writeln!(stdout)
}

fn print_stats(counters: &[impl Counter]) -> io::Result<()> {
    let total: u64 = counters.iter().map(Counter::count).sum();
    let stdout = &mut io::stdout().lock();
    writeln!(stdout, "= {total}")?;
    print_counters(counters, stdout, total, 0)
}

fn print_details(details: &[DetailCounter]) -> io::Result<()> {
    let total: u64 = details.iter().map(DetailCounter::count).sum();
    let stdout = &mut io::stdout().lock();
    writeln!(stdout, "= {total}")?;
    writeln!(stdout)?;

    for detail in details {
        let count = detail.count();
        let ratio = (count as f64 / total as f64) * 100.0;
        writeln!(stdout, "{} {count} ({ratio:.2}%)", detail.name())?;
        print_counters(detail.mnemonics(), stdout, total, 4)?;
    }

    Ok(())
}

fn run_detect(file: File, bitness: u32, segments: &[Segment], output: Output) -> io::Result<()> {
    let features = decode::<TaskCount>(file, bitness, segments)?.into_result();

    if output > Output::Quiet {
        detect_cpuid(&features);
        print!("Features: ");
    }

    print_features(&features)
}

fn run_stats(file: File, bitness: u32, segments: &[Segment], output: Output) -> io::Result<()> {
    let mut stats = decode::<TaskCount>(file, bitness, segments)?.into_result();
    Counter::sort(&mut stats);

    if output > Output::Quiet {
        println!("----------");
        print_stats_note();
    }

    print_stats(&stats)
}

fn run_details(file: File, bitness: u32, segments: &[Segment], output: Output) -> io::Result<()> {
    let (mut features, mut registers) =
        decode::<TaskDetail>(file, bitness, segments)?.into_result();

    DetailCounter::sort(&mut features);
    Counter::sort(&mut registers);

    if output > Output::Quiet {
        println!();
        print_header("Instructions");
        print_stats_note();
    }

    print_details(&features)?;

    if output > Output::Quiet {
        print_header("Registers");
    }

    print_stats(&registers)
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
        segments.iter().for_each(print_segment);
    }

    use Mode::*;
    match mode {
        Detect => run_detect(file, bitness, segments, output),
        Stats => run_stats(file, bitness, segments, output),
        Details => run_details(file, bitness, segments, output),
    }?;

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
