use std::{error::Error, fmt};

pub type R<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub enum AppError {
    WrongArch,
    NoText,
}

impl Error for AppError {}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use AppError::*;
        match self {
            WrongArch => write!(f, "Unsupported architecture"),
            NoText => write!(f, "No 'text' sections found in the file"),
        }
    }
}
