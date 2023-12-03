use lazy_static::lazy_static;
use regex::Regex;

#[derive(Debug, PartialEq)]
pub struct Game {
    pub id: u32,
    pub sets: Vec<Set>,
}

#[derive(Debug, PartialEq)]
pub struct Set {
    pub red: u32,
    pub green: u32,
    pub blue: u32,
}

impl Set {
    pub fn is_valid(&self, red_limit: u32, green_limit: u32, blue_limit: u32) -> bool {
        self.red <= red_limit && self.green <= green_limit && self.blue <= blue_limit
    }

    pub fn min_set(sets: Vec<Set>) -> Set {
        Set {
            red: sets.iter().map(|s| s.red).max().unwrap_or(0),
            green: sets.iter().map(|s| s.green).max().unwrap_or(0),
            blue: sets.iter().map(|s| s.blue).max().unwrap_or(0),
        }
    }
}

lazy_static! {
    static ref GAME_REGEX: Regex = Regex::new(r"^Game (\d+): (.*)$").unwrap();
    static ref COLOR_REGEX: Regex = Regex::new(r"(\d+) (blue|red|green)").unwrap();
}

impl Game {
    pub fn parse_from_string(input: &str) -> Game {
        let mut game = Game {
            id: 0,
            sets: vec![],
        };

        let res = GAME_REGEX.captures(input);

        let captures = res.unwrap();

        game.id = captures[1].parse().unwrap();

        for set_str in captures[2].split(';') {
            let mut set = Set {
                red: 0,
                blue: 0,
                green: 0,
            };

            for color_cap in COLOR_REGEX.captures_iter(set_str) {
                let count: u32 = color_cap[1].parse().unwrap();
                match &color_cap[2] {
                    "red" => set.red = count,
                    "green" => set.green = count,
                    "blue" => set.blue = count,
                    _ => {}
                }
            }
            game.sets.push(set);
        }

        game
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green",
    Game{id: 1, sets: vec ! [Set{blue: 3, red: 4, green: 0}, Set{red: 1, green: 2, blue: 6}, Set{green: 2, red: 0, blue: 0}]})]
    #[case("Game 2: 1 blue, 2 green; 3 green, 4 blue, 1 red; 1 green, 1 blue",
    Game{id: 2, sets: vec ! [Set{blue: 1, green: 2, red: 0}, Set{green: 3, blue: 4, red: 1}, Set{green: 1, blue: 1, red: 0}]})]
    #[case("Game 3: 8 green, 6 blue, 20 red; 5 blue, 4 red, 13 green; 5 green, 1 red",
    Game{id: 3, sets: vec ! [Set{green: 8, blue: 6, red: 20}, Set{blue: 5, red: 4, green: 13}, Set{green: 5, red: 1, blue: 0}]})]
    #[case("Game 4: 1 green, 3 red, 6 blue; 3 green, 6 red; 3 green, 15 blue, 14 red",
    Game{id: 4, sets: vec ! [Set{green: 1, red: 3, blue: 6}, Set{green: 3, red: 6, blue: 0}, Set{green: 3, blue: 15, red: 14}]})]
    #[case("Game 5: 6 red, 1 blue, 3 green; 2 blue, 1 red, 2 green",
    Game{id: 5, sets: vec ! [Set{red: 6, blue: 1, green: 3}, Set{blue: 2, red: 1, green: 2}]})]
    fn examples(#[case] input: String, #[case] expected: Game) {
        assert_eq!(Ok(expected), Game::parse_from_string(&input));
    }
}
