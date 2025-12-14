use crate::binary::{Binary, Segment};
use crate::cli::{self, Mode, Output};
use crate::decoder::{Item, Task, TaskCount, TaskDetail};
use crate::io::File;
use crate::print;
use crate::types::Arr;
use std::env;
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

struct App {
    file: File,
    bitness: u32,
    segments: Arr<Segment>,
    output: Output,
}

impl App {
    fn new(file: File, bitness: u32, segments: Arr<Segment>, output: Output) -> Self {
        Self { file, bitness, segments, output }
    }

    fn exec<T: Task>(&self, task: T) -> IoResult<T::Result> {
        self.file.decode(task, self.bitness, &self.segments)
    }

    #[inline(never)]
    fn detect(&self) -> IoResult<()> {
        let features = self.exec(TaskCount::new())?;

        if self.output > Output::Quiet {
            print::cpuid(&features);
            print!("Features: ");
        }

        print::features(&features)
    }

    #[inline(never)]
    fn stats(&self) -> IoResult<()> {
        let mut stats = self.exec(TaskCount::new())?;
        Item::sort_list(&mut stats);

        if self.output > Output::Quiet {
            println!();
            print::stats_note();
        }

        print::stats(&stats)
    }

    #[inline(never)]
    fn details(&self) -> IoResult<()> {
        let (mut details, mut registers) = self.exec(TaskDetail::new())?;
        Item::sort_list(&mut details);
        Item::sort_list(&mut registers);

        if self.output > Output::Quiet {
            println!();
            print::header("Instructions");
            print::stats_note();
        }

        print::details(&details)?;

        if self.output > Output::Quiet {
            print::header("Registers");
        }

        print::stats(&registers)
    }

    fn run(&self, mode: Mode) -> IoResult<()> {
        use Mode::*;
        match mode {
            Detect => self.detect(),
            Stats => self.stats(),
            Details => self.details(),
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

pub fn run() -> Result<bool, Box<dyn Error>> {
    let Some(config) = cli::read_args(env::args().skip(1))? else {
        return Ok(true);
    };

    let Some(file_path) = config.file_path() else {
        return Ok(true);
    };

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
    let segments = binary.into_segments();
    test!(segments.is_empty(), NoText);

    if output > Output::Normal {
        println!("Text sections:");
        segments.iter().for_each(print::segment);
    }

    let mode = config.mode().unwrap_or(DEFAULT_MODE);
    App::new(file, bitness, segments, output).run(mode)?;

    Ok(false)
}
