use std::env;
use std::fs::File;
use std::io;
use std::io::{Seek, SeekFrom};
use std::path::Path;

mod part1;
mod part2;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "No input file path provided",
        ));
    }

    let path = Path::new(&args[1]);
    let mut file = File::open(path)?;

    if let Ok(a) = part1::Almanac::parse_from_stream(&mut file) {
        let closest_seed_loc = a.closest_seed_loc().unwrap();
        println!("Part 1 - nearest location is {}", closest_seed_loc);
    } else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Could not parse almanac",
        ));
    };

    if file.seek(SeekFrom::Start(0)).is_err() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Could not seek file to start",
        ));
    }

    if let Ok(a) = part2::Almanac::parse_from_stream(&mut file) {
        let closest_seed_loc = a.closest_seed_loc_optimized().unwrap();
        println!("Part 2 - nearest location is {}", closest_seed_loc);
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Could not parse almanac",
        ))
    }
}
