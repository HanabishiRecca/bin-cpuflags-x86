#[cfg(test)]
mod tests;

use crate::types::Str;

#[derive(Clone, Copy, PartialEq)]
#[cfg_attr(test, derive(Debug))]
pub enum DecoderMode {
    Simple,
    Stat,
    Detail,
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
#[cfg_attr(test, derive(Debug))]
pub enum OutputMode {
    Quiet,
    Normal,
    Verbose,
}

#[derive(Default)]
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct Config {
    file_path: Option<Str>,
    decoder_mode: Option<DecoderMode>,
    output_mode: Option<OutputMode>,
}

impl Config {
    pub fn file_path(&self) -> Option<&str> {
        self.file_path.as_deref()
    }

    pub fn decoder_mode(&self) -> Option<DecoderMode> {
        self.decoder_mode
    }

    pub fn output_mode(&self) -> Option<OutputMode> {
        self.output_mode
    }
}

#[derive(Debug)]
pub enum Error {
    Unknown(Str),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use Error::*;
        match self {
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
        From::from($s)
    };
}

pub fn read_args(args: impl Iterator<Item = impl AsRef<str>>) -> Result<Option<Config>> {
    let mut config = Config::default();
    let mut escape = false;

    for arg in args {
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
        match arg {
            "-s" | "--stats" => {
                config.decoder_mode = Some(DecoderMode::Stat);
            }
            "-d" | "--details" => {
                config.decoder_mode = Some(DecoderMode::Detail);
            }
            "-v" | "--verbose" => {
                config.output_mode = Some(OutputMode::Verbose);
            }
            "-q" | "--quiet" => {
                config.output_mode = Some(OutputMode::Quiet);
            }
            "--" => {
                escape = true;
            }
            "-h" | "--help" => return Ok(None),
            _ => E!(Unknown(F!(arg))),
        }
    }

    Ok(Some(config))
}
