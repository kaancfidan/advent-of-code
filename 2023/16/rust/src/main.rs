mod light;

use crate::light::{Direction, ParseError, Room};
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

impl Display for InputError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            InputError::MissingPath() => write!(f, "No input file path provided"),
            InputError::ParseError(e) => write!(f, "Could not parse room: {e}"),
            InputError::IO(e) => write!(f, "Could not open file: {e}"),
        }
    }
}

fn main() -> Result<(), InputError> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        return Err(InputError::MissingPath());
    }

    let path = Path::new(&args[1]);
    let file = File::open(path).map_err(InputError::IO)?;

    let room = Room::from_stream(file).map_err(InputError::ParseError)?;

    let energized = room.energized((0, 0), Direction::East);

    println!("Energized tile count: {energized}");

    let max_energized = room.max_energized();

    println!("Max energized tile count: {max_energized}");

    Ok(())
}
