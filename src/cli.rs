use crate::{
    error::{ArgError, R},
    E,
};

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub enum OutputMode {
    Quiet,
    Normal,
    Verbose,
}

pub struct Config {
    pub file_path: Option<String>,
    pub output_mode: OutputMode,
}

impl Config {
    fn new() -> Self {
        Config {
            file_path: None,
            output_mode: OutputMode::Normal,
        }
    }
}

pub fn read_args(args: impl Iterator<Item = String>) -> R<Option<Config>> {
    let mut config = Config::new();
    let mut read_options = true;

    for arg in args {
        if arg.is_empty() {
            continue;
        }
        if !(read_options && arg.starts_with('-')) {
            config.file_path.get_or_insert(arg);
            continue;
        }
        match arg.as_str().trim() {
            "-v" | "--verbose" => config.output_mode = OutputMode::Verbose,
            "-q" | "--quiet" => config.output_mode = OutputMode::Quiet,
            "-h" | "--help" => return Ok(None),
            "--" => read_options = false,
            _ => E!(ArgError::Unknown(arg)),
        }
    }

    Ok(Some(config))
}
