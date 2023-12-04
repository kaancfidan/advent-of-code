use anyhow::Error;
use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

use crate::scratch::Card;

mod scratch;

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

    let cards_result: Result<Vec<Card>, _> = reader
        .lines()
        .map(|line| {
            let l = line.map_err(Error::new)?;
            l.parse::<Card>().map_err(Error::new)
        })
        .collect();

    if cards_result.is_err() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Could not parse file into cards",
        ));
    }

    let mut cards = cards_result.unwrap();

    // part 1
    let sum: u64 = cards.iter().map(|c| c.score()).sum();

    println!("The total naive score of scratch cards is: {}", sum);

    // part 2
    for i in 0..cards.len() {
        let count = cards[i].matching_count() as usize;
        if count == 0 {
            continue;
        }

        for j in 1..=count {
            if (i + j) >= cards.len() {
                break;
            }

            cards[i + j].n_instances += cards[i].n_instances;
        }
    }

    let sum_instances: u64 = cards.into_iter().map(|c| c.n_instances).sum();

    println!("The total sum of instances: {}", sum_instances);

    Ok(())
}
