mod part1;
mod part2;

use std::env;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

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
    let mut reader = io::BufReader::new(file);

    let input = &mut String::new();
    reader.read_to_string(input)?;

    let lines: Vec<_> = input.split('\n').collect();
    if lines.len() != 2 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Expected 2 lines",
        ));
    }

    let nums = part2::parse_nums(lines)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Expected numbers"))?;

    let n_ways = calculate_ways_to_beat_record(nums);

    println!("Number of ways to beat the record: {}", n_ways);

    Ok(())
}

fn calculate_ways_to_beat_record(nums: Vec<Vec<u64>>) -> u32 {
    let n_ways: u32 = nums[0]
        .clone()
        .into_iter()
        .zip(nums[1].clone())
        .map(|t| {
            let time = t.0 as f64;
            let dist = t.1 as f64;

            let sqrt_delta = (time.powf(2.0) - 4.0 * dist).sqrt();

            let min: u32 = ((time - sqrt_delta) / 2.0).floor() as u32;
            let max: u32 = ((time + sqrt_delta) / 2.0).floor() as u32;

            max - min
        })
        .product();
    n_ways
}
