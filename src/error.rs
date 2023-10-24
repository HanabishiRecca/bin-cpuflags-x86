use std::{error::Error, fmt};

pub type R<T> = Result<T, Box<dyn Error>>;

#[macro_export]
macro_rules! E {
    ($e: expr) => {
        Err($e)?
    };
}

#[derive(Debug)]
pub enum AppError {
    WrongTarget,
    WrongArch,
    NoText,
}

impl Error for AppError {}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use AppError::*;
        match self {
            WrongTarget => write!(f, "Should target a file"),
            WrongArch => write!(f, "Unsupported architecture"),
            NoText => write!(f, "No 'text' sections found in the file"),
        }
    }
}

#[derive(Debug)]
pub enum ArgError {
    Unknown(String),
}

impl Error for ArgError {}

impl fmt::Display for ArgError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ArgError::*;
        match self {
            Unknown(arg) => write!(f, "unknown option '{arg}'"),
        }
    }
}
