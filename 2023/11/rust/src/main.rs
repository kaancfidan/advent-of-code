mod galaxy;

use crate::galaxy::Galaxy;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::path::Path;
use std::{env, io};

#[derive(Debug)]
enum InputError {
    MissingPath,
    IO(io::Error),
    ParseFailed(galaxy::ParseError),
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

    let galaxy = Galaxy::from_stream(file).map_err(InputError::ParseFailed)?;

    // part 1
    let part1_dist = galaxy.calculate_total_distance(2);
    println!(
        "Part 1 - sum of all distances between stars is {}",
        part1_dist
    );

    // part 2
    let part2_dist = galaxy.calculate_total_distance(1_000_000);
    println!(
        "Part 2 - sum of all distances between stars is {}",
        part2_dist
    );

    Ok(())
}
