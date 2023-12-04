use crate::schematic::Schematic;
use std::env;
use std::fs::File;
use std::io;
use std::path::Path;

mod schematic;

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

    let s = Schematic::parse_from_stream(file);

    if s.is_err() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Could not parse input file into schematics",
        ));
    }

    let schematic = s.unwrap();
    let sum_parts: u64 = schematic.find_part_numbers().map(|n| n.value).sum();

    println!("The sum of all part numbers is: {}", sum_parts);

    let sum_gears: u64 = schematic.find_gear_ratios().sum();

    println!("The sum of gear ratios is: {}", sum_gears);

    Ok(())
}
