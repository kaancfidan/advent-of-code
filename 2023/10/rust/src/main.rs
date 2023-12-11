mod map;

use crate::map::Map;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::path::Path;
use std::{env, io};

#[derive(Debug)]
enum InputError {
    MissingPath,
    IO(io::Error),
    ParseFailed(map::ParseError),
    NoLoopFound,
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
            InputError::ParseFailed(e) => write!(f, "Could not parse input file: {}", e),
            InputError::NoLoopFound => write!(f, "No loop found in map"),
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

    let map = Map::from_stream(file).map_err(InputError::ParseFailed)?;
    let l = map.find_loop_iteration().ok_or(InputError::NoLoopFound)?;

    println!(
        "Found loop with length {}, furthest point is @ {}",
        l.len(),
        l.len() / 2
    );

    let nests = map.find_nests(l);

    println!("Found {} potential nest positions", nests.len());

    Ok(())
}
