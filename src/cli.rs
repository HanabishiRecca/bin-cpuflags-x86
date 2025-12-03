#[cfg(test)]
mod tests;

use crate::types::Str;
use std::{error, fmt};

#[derive(Clone, Copy, PartialEq)]
#[cfg_attr(test, derive(Debug))]
pub enum Mode {
    Detect,
    Stats,
    Details,
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
#[cfg_attr(test, derive(Debug))]
pub enum Output {
    Quiet,
    Normal,
    Verbose,
}

#[derive(Default)]
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct Config {
    file_path: Option<Str>,
    mode: Option<Mode>,
    output: Option<Output>,
}

impl Config {
    pub fn file_path(&self) -> Option<&str> {
        self.file_path.as_deref()
    }

    pub fn mode(&self) -> Option<Mode> {
        self.mode
    }

    pub fn output(&self) -> Option<Output> {
        self.output
    }
}

#[derive(Debug)]
pub enum Error {
    NoValue(Str),
    InvalidValue(Str, Str),
    Unknown(Str),
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match self {
            NoValue(arg) => write!(f, "Option '{arg}' requires a value"),
            InvalidValue(arg, value) => write!(f, "Invalid value '{value}' for option '{arg}'"),
            Unknown(arg) => write!(f, "Unknown option '{arg}'"),
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

macro_rules! E {
    ($e: expr) => {{
        use Error::*;
        return Err($e);
    }};
}

macro_rules! F {
    ($s: expr) => {
        From::from($s.as_ref())
    };
}

pub fn read_args(mut args: impl Iterator<Item = impl AsRef<str>>) -> Result<Option<Config>> {
    let mut config = Config::default();
    let mut escape = false;

    while let Some(arg) = args.next() {
        let arg = arg.as_ref();

        if escape {
            config.file_path = Some(F!(arg));
            break;
        }

        if arg.is_empty() {
            continue;
        }

        if !arg.starts_with('-') {
            config.file_path = Some(F!(arg));
            continue;
        }

        macro_rules! next {
            () => {
                match args.next() {
                    Some(value) => value,
                    _ => E!(NoValue(F!(arg))),
                }
            };
        }

        match arg {
            "--mode" => {
                let value = next!();
                use Mode::*;
                config.mode = Some(match value.as_ref() {
                    "detect" => Detect,
                    "stats" => Stats,
                    "details" => Details,
                    _ => E!(InvalidValue(F!(arg), F!(value))),
                });
            }
            "-s" | "--stats" => {
                config.mode = Some(Mode::Stats);
            }
            "-d" | "--details" => {
                config.mode = Some(Mode::Details);
            }

            "--output" => {
                let value = next!();
                use Output::*;
                config.output = Some(match value.as_ref() {
                    "quiet" => Quiet,
                    "normal" => Normal,
                    "verbose" => Verbose,
                    _ => E!(InvalidValue(F!(arg), F!(value))),
                });
            }
            "-v" | "--verbose" => {
                config.output = Some(Output::Verbose);
            }
            "-q" | "--quiet" => {
                config.output = Some(Output::Quiet);
            }

            "-h" | "--help" => return Ok(None),

            "--" => {
                escape = true;
            }

            _ => E!(Unknown(F!(arg))),
        }
    }

    Ok(Some(config))
}
