mod dig;

use crate::dig::{Elves, ParseError, Plan};
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
    PlanDecodeError(String),
}

impl Display for InputError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            InputError::MissingPath() => write!(f, "No input file path provided"),
            InputError::ParseError(e) => write!(f, "Could not parse plan: {e}"),
            InputError::IO(e) => write!(f, "Could not open file: {e}"),
            InputError::PlanDecodeError(s) => write!(f, "Could not decode plan: {s}"),
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

    let mut plan = Plan::from_stream(file).map_err(InputError::ParseError)?;

    let (mut pool, seed) = Elves::dig_sides(&plan);

    Elves::dig_out_interior(&mut pool, seed);

    println!("Initial volume: {}", pool.volume());

    Elves::decode_plan(&mut plan).map_err(InputError::PlanDecodeError)?;

    let decoded_volume = Elves::calculate_volume(&plan);

    println!("Decoded volume: {}", decoded_volume);

    Ok(())
}
