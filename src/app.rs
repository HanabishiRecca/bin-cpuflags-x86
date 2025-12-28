use crate::binary::{Binary, Segment};
use crate::cli::{self, Mode, Output};
use crate::decoder::{Item, Task, TaskCount, TaskDetail};
use crate::print;
use crate::reader::Reader;
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
    reader: Reader,
    bitness: u32,
    segments: Arr<Segment>,
}

impl App {
    fn new(reader: Reader, bitness: u32, segments: Arr<Segment>) -> Self {
        Self { reader, bitness, segments }
    }

    fn exec<T: Task>(&self, task: T) -> IoResult<T::Result> {
        self.reader.read(task, self.bitness, &self.segments)
    }

    fn detect(&self) -> IoResult<()> {
        let features = self.exec(TaskCount::new())?;
        print::features(&features);
        Ok(())
    }

    fn stats(&self) -> IoResult<()> {
        let mut stats = self.exec(TaskCount::new())?;
        Item::sort_list(&mut stats);
        print::stats(&stats);
        Ok(())
    }

    fn details(&self) -> IoResult<()> {
        let (mut details, mut registers) = self.exec(TaskDetail::new())?;
        Item::sort_list(&mut details);
        Item::sort_list(&mut registers);
        print::details(&details);
        print::registers(&registers);
        Ok(())
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

macro_rules! ok {
    ($o: expr $(,)?) => {
        match $o {
            Some(value) => value,
            _ => return Ok(true),
        }
    };
}

macro_rules! err {
    ($o: expr, $e: expr $(,)?) => {{
        use AppError::*;
        ($o).ok_or($e)?
    }};
}

pub fn run() -> Result<bool, Box<dyn Error>> {
    let config = ok!(cli::read_args(env::args().skip(1))?);
    print::set_output(config.output().unwrap_or(DEFAULT_OUTPUT));

    let file_path = ok!(config.file_path());
    print::file_path(file_path);

    let reader = err!(Reader::open(file_path)?, WrongTarget);
    let binary = Binary::parse(reader.file())?;
    print::binary(&binary);

    let bitness = err!(binary.bitness(), WrongArch);
    let segments = err!(binary.into_segments(), NoText);
    print::segments(&segments);

    let mode = config.mode().unwrap_or(DEFAULT_MODE);
    App::new(reader, bitness, segments).run(mode)?;

    Ok(false)
}
