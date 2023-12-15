mod inventory;

use crate::inventory::Record;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::{env, io};

#[derive(Debug)]
enum InputError {
    MissingPath,
    IO(io::Error),
    ParseFailed(inventory::ParseError),
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

    let reader = BufReader::new(file);

    let mut memo = HashMap::new();

    let records: Vec<_> = reader
        .lines()
        .map(|l| {
            let line = l.map_err(InputError::IO)?;
            let record = line.parse::<Record>().map_err(InputError::ParseFailed)?;
            Ok(record)
        })
        .collect::<Result<Vec<_>, InputError>>()?;

    let sum_valid: u64 = records
        .iter()
        .map(|r| r.valid_configuration_count(&mut memo))
        .sum();

    println!("Number of valid configurations: {}", sum_valid);

    let sum_unfolded: u64 = records
        .iter()
        .map(|r| r.unfolded(5))
        .map(|r| r.valid_configuration_count(&mut memo))
        .sum();

    println!(
        "Number of valid configurations after unfolding: {}",
        sum_unfolded
    );

    Ok(())
}
