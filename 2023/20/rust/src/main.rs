mod comms;

use crate::comms::{Message, System};
use std::env;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io;
use std::path::Path;

#[derive(Debug)]
enum InputError {
    MissingPath(),
    IO(io::Error),
    ParseError(comms::ParseError),
}

impl Display for InputError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            InputError::MissingPath() => write!(f, "No input file path provided"),
            InputError::IO(e) => write!(f, "Could not open file: {e}"),
            InputError::ParseError(e) => write!(f, "Could not parse communication diagram: {e}"),
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

    let mut system = System::from_stream(file).map_err(InputError::ParseError)?;

    let messages: Vec<Message> = (0..1000).flat_map(|_| system.push_button()).collect();

    let low_count = messages.iter().filter(|m| !m.pulse).count();
    let high_count = messages.iter().filter(|m| m.pulse).count();

    println!(
        "High count: {high_count}, low count: {low_count}, product: {}",
        high_count * low_count
    );

    system.reset();

    let mut push_count = 0;
    loop {
        let messages = system.push_button();
        push_count += 1;

        println!("{push_count}");

        if messages.iter().any(|m| m.dst == "rx" && !m.pulse) {
            break;
        }
    }

    println!("Push count until first low pulse to rx: {push_count}");

    Ok(())
}
