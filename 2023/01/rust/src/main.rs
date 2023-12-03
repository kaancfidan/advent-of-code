mod part1;
mod part2;

use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
// use part1::extract_code;
use part2::extract_code;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "No input file path provided",
        ));
    }

    let path = Path::new(&args[1]);
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);

    let sum: Option<u32> = reader
        .lines()
        .map(|line| match line {
            Ok(l) => extract_code(l),
            Err(_) => None,
        })
        .sum();

    if sum.is_none() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Could not extract code from all lines",
        ));
    }

    println!("The sum of all calibration values is {}", sum.unwrap());
    Ok(())
}
