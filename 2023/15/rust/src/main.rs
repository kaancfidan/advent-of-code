mod lens;

use crate::lens::Manual;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::{env, io};

fn main() -> Result<(), InputError> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        return Err(InputError::MissingPath());
    }

    let path = Path::new(&args[1]);
    let mut file = File::open(path).map_err(InputError::IO)?;

    let mut line = "".to_owned();
    file.read_to_string(&mut line).map_err(InputError::IO)?;

    let steps: Vec<_> = Manual::parse_line(&line);
    let hash_sum = Manual::hash_sum(&steps);

    println!("HASH sum is: {hash_sum}");

    let manual = line.parse::<Manual>().map_err(InputError::ParseError)?;
    let config = manual.create_configuration();

    println!("Configuration focusing power: {}", config.focusing_power());

    Ok(())
}

#[derive(Debug)]
enum InputError {
    MissingPath(),
    IO(io::Error),
    ParseError(lens::ParseError)
}

impl Display for InputError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            InputError::MissingPath() => write!(f, "No input file path provided"),
            InputError::IO(e) => write!(f, "Could not read input file: {e}"),
            InputError::ParseError(e) => write!(f, "Could not parse input to manual: {e}")
        }
    }
}
