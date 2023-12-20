mod path;

use crate::path::{City, Crucible, ParseError, Position};
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
            InputError::ParseError(e) => write!(f, "Could not parse city: {e}"),
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

    let city = City::from_stream(file).map_err(InputError::ParseError)?;

    let path = city.navigate(
        Position { x: 0, y: 0 },
        Position {
            x: city.width - 1,
            y: city.height - 1,
        },
        Crucible::Ultra,
    );

    let total_loss: u32 = path.iter().map(|b| b.heat_loss).sum();

    print!("Total heat loss: {total_loss}");

    Ok(())
}
