mod mirror;

use crate::mirror::{ParseError, Valley};
use either::{Left, Right};
use std::fmt::{Display, Formatter};
use std::io;
use std::path::Path;
use std::{env, fs};

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
            InputError::ParseError(e) => write!(f, "Could not parse valley: {e}"),
        }
    }
}

fn main() -> Result<(), InputError> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        return Err(InputError::MissingPath());
    }

    let path = Path::new(&args[1]);
    let content = fs::read_to_string(path).map_err(InputError::IO)?;

    let valleys: Vec<_> = content
        .split("\r\n\r\n")
        .map(|s| {
            let lines = s.trim().split("\r\n").collect::<Vec<_>>();
            Valley::try_from(&lines[..]).map_err(InputError::ParseError)
        })
        .collect::<Result<Vec<_>, _>>()?;

    let sum: usize = valleys
        .iter()
        .map(|v| v.mirror_pos(None))
        .map(|p| match p {
            Some(Left(h)) => 100 * h,
            Some(Right(v)) => v,
            _ => 0,
        })
        .sum();

    println!("Part 1 sum: {sum}");

    let smudged_sum: usize = valleys
        .iter()
        .map(|v| v.smudged_mirror_pos())
        .map(|p| match p {
            Some(Left(h)) => 100 * h,
            Some(Right(v)) => v,
            _ => 0,
        })
        .sum();

    println!("Part 2 sum: {smudged_sum}");

    Ok(())
}
