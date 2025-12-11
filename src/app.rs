use crate::binary::{Binary, Segment};
use crate::cli::{Mode, Output};
use crate::decoder::{Counter, TaskCount, TaskDetail};
use crate::io::File;
use crate::print;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::io::Result as IoResult;

const DEFAULT_MODE: Mode = Mode::Detect;
const DEFAULT_OUTPUT: Output = Output::Normal;

#[derive(Debug)]
pub enum AppError {
    WrongTarget,
    WrongArch,
    NoText,
}

impl Error for AppError {}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        use AppError::*;
        match self {
            WrongTarget => write!(f, "Should target a file"),
            WrongArch => write!(f, "Unsupported architecture"),
            NoText => write!(f, "No 'text' sections found in the file"),
        }
    }
}

macro_rules! or {
    ($o: expr, $e: expr $(,)?) => {{
        use AppError::*;
        ($o).ok_or($e)?
    }};
}

macro_rules! test {
    ($b: expr, $e: expr $(,)?) => {
        or!((!$b).then_some(()), $e)
    };
}

fn run_detect(file: File, bitness: u32, segments: &[Segment], output: Output) -> IoResult<()> {
    let features = file.decode::<TaskCount>(bitness, segments)?;

    if output > Output::Quiet {
        print::cpuid(&features);
        print!("Features: ");
    }

    print::features(&features)
}

fn run_stats(file: File, bitness: u32, segments: &[Segment], output: Output) -> IoResult<()> {
    let mut stats = file.decode::<TaskCount>(bitness, segments)?;
    Counter::sort(&mut stats);

    if output > Output::Quiet {
        println!();
        print::stats_note();
    }

    print::stats(&stats)
}

fn run_details(file: File, bitness: u32, segments: &[Segment], output: Output) -> IoResult<()> {
    let (mut features, mut registers) = file.decode::<TaskDetail>(bitness, segments)?;
    Counter::sort(&mut features);
    Counter::sort(&mut registers);

    if output > Output::Quiet {
        println!();
        print::header("Instructions");
        print::stats_note();
    }

    print::details(&features)?;

    if output > Output::Quiet {
        print::header("Registers");
    }

    print::stats(&registers)
}

pub fn run() -> Result<bool, Box<dyn Error>> {
    let Some(config) = crate::cli::read_args(std::env::args().skip(1))? else {
        return Ok(true);
    };

    let Some(file_path) = config.file_path() else {
        return Ok(true);
    };

    let mode = config.mode().unwrap_or(DEFAULT_MODE);
    let output = config.output().unwrap_or(DEFAULT_OUTPUT);

    if output > Output::Normal {
        print::file_path(file_path);
    }

    let file = File::open(file_path)?;
    test!(file.is_dir()?, WrongTarget);

    let binary = Binary::parse(file.fs_file())?;

    if output > Output::Quiet {
        print::binary(&binary);
    }

    let bitness = or!(binary.bitness(), WrongArch);
    let segments = binary.segments();
    test!(segments.is_empty(), NoText);

    if output > Output::Normal {
        println!("Text sections:");
        segments.iter().for_each(print::segment);
    }

    use Mode::*;
    match mode {
        Detect => run_detect(file, bitness, segments, output),
        Stats => run_stats(file, bitness, segments, output),
        Details => run_details(file, bitness, segments, output),
    }?;

    Ok(false)
}
