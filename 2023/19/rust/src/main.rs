mod workflow;

use crate::workflow::{Elves, ParameterRange, ParseError, WorkflowError};
use std::collections::HashMap;
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
    WorkflowError(WorkflowError),
}

impl Display for InputError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            InputError::MissingPath() => write!(f, "No input file path provided"),
            InputError::IO(e) => write!(f, "Could not open file: {e}"),
            InputError::ParseError(e) => write!(f, "Could not parse plan: {e}"),
            InputError::WorkflowError(e) => write!(f, "Could not follow workflow: {e}"),
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

    let elves = &Elves::from_instructions(file).map_err(InputError::ParseError)?;

    let accepted = elves.check_parts().map_err(InputError::WorkflowError)?;

    let total_rating: u64 = accepted.iter().map(|p| p.rating()).sum();

    println!("Total rating is {total_rating}");

    let mut r = ParameterRange {
        ranges: HashMap::new(),
    };
    r.ranges.insert('x', 1..4001);
    r.ranges.insert('m', 1..4001);
    r.ranges.insert('a', 1..4001);
    r.ranges.insert('s', 1..4001);

    let valid_ranges = elves.find_valid_ranges(r).unwrap();

    let combinations: u64 = valid_ranges.iter().map(|r| r.combinations()).sum();

    println!("Number of valid combinations is {combinations}");

    Ok(())
}
