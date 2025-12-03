mod binary;
mod cli;
mod decoder;
mod types;

use crate::{
    binary::{Binary, Segment},
    cli::{DecoderMode, OutputMode},
    decoder::{FDetail, FSimple, Feature, Mnemonic, Task},
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

fn print_simple(features: Arr<FSimple>, output_mode: OutputMode) -> io::Result<()> {
    let mut stdout = io::stdout().lock();

    if output_mode > OutputMode::Quiet {
        write!(stdout, "Features: ")?;
    }

    for feature in features {
        write!(stdout, "{} ", feature.name())?;
    }

    writeln!(stdout)?;

    Ok(())
}

fn print_stat(mut features: Arr<FSimple>, output_mode: OutputMode) -> io::Result<()> {
    let mut stdout = io::stdout().lock();

    if output_mode > OutputMode::Quiet {
        writeln!(stdout, "----------")?;
    }

    features.sort_unstable_by_key(|feature| Reverse(feature.count()));

    let nlen = features.iter().map(|f| f.name().len()).max().unwrap_or(0);
    let total: u64 = features.iter().map(FSimple::count).sum();
    writeln!(stdout, "{:nlen$} {total}", "=")?;

    for feature in features {
        let ratio = (feature.count() as f64 / total as f64) * 100.0;
        writeln!(stdout, "{:nlen$} {} ({ratio:.2}%)", feature.name(), feature.count())?;
    }

    Ok(())
}

fn print_detail(features: Arr<FDetail>, output_mode: OutputMode) -> io::Result<()> {
    let mut stdout = io::stdout().lock();

    if output_mode > OutputMode::Quiet {
        writeln!(stdout, "----------")?;
    }

    for feature in features {
        write!(stdout, "{}: ", feature.name())?;

        let mut mnemonics = feature.into_mnemonics();
        mnemonics.sort_unstable_by_key(Mnemonic::name);

        for mnemonic in mnemonics {
            write!(stdout, "{} ", mnemonic.name())?;
        }

        writeln!(stdout)?;
    }

    Ok(())
}

fn decode<T: Feature>(
    mut file: File, binary: Binary, output_mode: OutputMode,
    print: fn(Arr<T>, OutputMode) -> io::Result<()>,
) -> Result<()> {
    let mut task = Task::<T>::new(or!(binary.bitness(), WrongArch));

    for segment in binary.segments() {
        task.read(&mut file, segment.offset(), segment.size())?;
    }

    if output_mode > OutputMode::Quiet && task.has_cpuid() {
        println!("Warning: CPUID usage detected. Features could switch in runtime.");
    }

    print(task.into_features(), output_mode)?;
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

    let file = File::open(file_path)?;
    test!(file.metadata()?.file_type().is_dir(), WrongTarget);

    let binary = parse(&file, output_mode)?;

    use DecoderMode::*;
    match decoder_mode {
        Simple => decode(file, binary, output_mode, print_simple)?,
        Stat => decode(file, binary, output_mode, print_stat)?,
        Detail => decode(file, binary, output_mode, print_detail)?,
    };

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
