use crate::part2::{Map, MapParseError};
use std::env;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io;
use std::path::Path;

mod common;
mod part1;
mod part2;

#[derive(Debug)]
enum InputError {
    MissingPath,
    IO(io::Error),
    ParseFailed(MapParseError),
    AssumptionFailed,
}

impl From<io::Error> for InputError {
    fn from(e: io::Error) -> Self {
        InputError::IO(e)
    }
}

impl Display for InputError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            InputError::MissingPath => write!(f, "No input file path provided"),
            InputError::IO(e) => write!(f, "Could not read input file: {}", e),
            InputError::ParseFailed(e) => write!(f, "Could not parse input: {}", e),
            InputError::AssumptionFailed => {
                write!(f, "Input is not well-aligned with form assumptions")
            }
        }
    }
}

fn main() -> Result<(), InputError> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        return Err(InputError::MissingPath);
    }

    let path = Path::new(&args[1]);
    let file = File::open(path)?;

    let map = Map::parse_from_stream(file).map_err(InputError::ParseFailed)?;

    let count = map
        .calculate_steps()
        .map_err(|_| InputError::AssumptionFailed)?;

    println!("Total number of steps is: {}", count);

    Ok(())
}
