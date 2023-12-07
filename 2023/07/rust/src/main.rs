use std::env;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{self, BufRead};
use std::num::ParseIntError;
use std::path::Path;

mod poker;
// mod part1;
mod part2;

#[derive(Debug)]
enum InputError {
    MissingPath(),
    IO(io::Error),
    Form(),
    ParseHand(poker::HandParseError),
    ParseBid(ParseIntError),
}

impl From<io::Error> for InputError {
    fn from(e: io::Error) -> Self {
        InputError::IO(e)
    }
}

impl Display for InputError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            InputError::MissingPath() => write!(f, "No input file path provided"),
            InputError::Form() => {
                write!(
                    f,
                    "Input lines are expected to be in form similar to \"32T3K 765\""
                )
            }
            InputError::IO(e) => write!(f, "Could not read input file: {}", e),
            InputError::ParseHand(e) => write!(f, "Could not parse hand: {}", e),
            InputError::ParseBid(e) => write!(f, "Could not parse bid: {}", e),
        }
    }
}

fn main() -> Result<(), InputError> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        return Err(InputError::MissingPath());
    }

    let path = Path::new(&args[1]);
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);

    let mut plays = reader
        .lines()
        .map(|line| {
            let l = line.map_err(InputError::IO)?;
            let split: Vec<_> = l.split_whitespace().collect();

            if split.len() != 2 {
                return Err(InputError::Form());
            }

            let hand = split[0]
                .parse::<poker::Hand>()
                .map_err(InputError::ParseHand)?;
            let bid = split[1].parse::<u32>().map_err(InputError::ParseBid)?;
            Ok((hand, bid))
        })
        .collect::<Result<Vec<(poker::Hand, u32)>, InputError>>()?;

    plays.sort_unstable_by(|a, b| (&a.0 as &dyn poker::ComparableHand).cmp(&b.0));

    let winnings: u32 = plays
        .into_iter()
        .enumerate()
        .map(|(i, p)| p.1 * (i as u32 + 1))
        .sum();

    println!("Total winnings are: {}", winnings);

    Ok(())
}
