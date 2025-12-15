mod app;
mod binary;
mod cli;
mod decoder;
mod print;
mod reader;
mod types;

use std::process::ExitCode;

fn main() -> ExitCode {
    match app::run() {
        Ok(help) => {
            help.then(print::help);
            ExitCode::SUCCESS
        }
        Err(error) => {
            print::error(error.as_ref());
            ExitCode::FAILURE
        }
    }
}
