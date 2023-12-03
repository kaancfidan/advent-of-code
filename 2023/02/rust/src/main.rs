use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

use crate::game::{Game, Set};

mod game;

static REDS: u32 = 12;
static GREENS: u32 = 13;
static BLUES: u32 = 14;

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

    let games: Vec<Game> = reader
        .lines()
        .map(|line| match line {
            Ok(l) => Game::parse_from_string(&l),
            Err(_) => Game {
                id: 0,
                sets: vec![],
            },
        })
        .collect();

    let sum: u32 = games
        .iter()
        .map(|game| {
            if game.sets.iter().all(|s| s.is_valid(REDS, GREENS, BLUES)) {
                game.id
            } else {
                0
            }
        })
        .sum();

    println!("Sum of valid game IDs is {}", sum);

    let sum_powers: u32 = games
        .into_iter()
        .map(|game| Set::min_set(game.sets))
        .map(|set| set.red * set.green * set.blue)
        .sum();

    println!("Sum of powers is {}", sum_powers);

    Ok(())
}
