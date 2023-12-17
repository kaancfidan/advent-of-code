mod tilt;

use crate::tilt::{ParseError, Platform};
use std::env;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io;
use std::path::Path;

#[derive(Debug)]
enum InputError {
    MissingPath(),
    IO(io::Error),
    ParseError(ParseError),
}

impl From<io::Error> for InputError {
    fn from(e: io::Error) -> Self {
        InputError::IO(e)
    }
}

impl Display for InputError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            InputError::MissingPath() => write!(f, "No input file path provided"),
            InputError::IO(e) => write!(f, "Could not read input file: {e}"),
            InputError::ParseError(e) => write!(f, "Could not parse platform: {e}"),
        }
    }
}

fn main() -> Result<(), InputError> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        return Err(InputError::MissingPath());
    }

    let path = Path::new(&args[1]);
    let file = File::open(path)?;

    let platform = Platform::from_stream(file).map_err(InputError::ParseError)?;

    let cycled = platform.cycled_many(1_000_000_000);

    let total_weight = cycled.total_weight();

    println!("Total weight of the platform after billion cycles: {total_weight}");

    Ok(())
}
