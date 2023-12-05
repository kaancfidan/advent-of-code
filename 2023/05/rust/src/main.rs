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
        if a.seeds.is_empty() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Empty seeds"));
        }

        let min_loc = a
            .seeds
            .iter()
            .map(|s| a.find_location(*s))
            .min()
            .unwrap();

        println!("Part 1 - nearest location is {}", min_loc);
    } else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Could not parse almanac",
        ))
    }

    if file.seek(SeekFrom::Start(0)).is_err() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Could not seek file to start",
        ));
    }

    return if let Ok(a) = part2::Almanac::parse_from_stream(&mut file) {
        if a.seeds.is_empty() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Empty seeds"));
        }

        let mut loc_conn = a.connections.last().unwrap();

        let mut exceptions = loc_conn.exceptions.clone();
        exceptions.sort_by(|a,b| b.dst.cmp(&a.dst));

        let last_ex = exceptions.first().unwrap();
        let max_loc = last_ex.dst + last_ex.count;

        let min_loc = (0..max_loc).find(|loc| {
            let s = a.find_seed(*loc);
            a.seeds.iter().any(|r| s >= r.start && s < r.start + r.count)
        }).unwrap();

        println!("Part 2 - nearest location is {}", min_loc);
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Could not parse almanac",
        ))
    }
}
