mod series;

use crate::series::{ParseError, Series};
use std::env;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{self, BufRead};
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
            InputError::IO(e) => write!(f, "Could not read input file: {}", e),
            InputError::ParseError(e) => write!(f, "Could not parse input file: {}", e),
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
    let reader = io::BufReader::new(file);

    let mut series: Vec<Series> = reader
        .lines()
        .map(|l| {
            let line = l.map_err(InputError::IO)?;
            line.parse::<Series>().map_err(InputError::ParseError)
        })
        .collect::<Result<Vec<_>, _>>()?;

    // part 1
    for s in series.iter_mut() {
        s.extrapolate_forward()
    }

    let sum: i64 = series.iter().map(|s| s.levels[0].last().unwrap()).sum();

    println!("Part 1 result is: {}", sum);

    // part 2
    for s in series.iter_mut() {
        s.extrapolate_backwards()
    }

    let sum: i64 = series.iter().map(|s| s.levels[0].first().unwrap()).sum();

    println!("Part 2 result is: {}", sum);

    Ok(())
}
