use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::io;
use std::io::{BufRead, BufReader, Read};

#[derive(Copy, Clone, Eq, PartialEq)]
enum Tile {
    Round,
    Cube,
    Empty,
}

#[derive(Clone)]
pub struct Platform {
    tiles: Vec<Vec<Tile>>,
    width: usize,
    height: usize,
}

impl Platform {
    pub fn from_stream(r: impl Read) -> Result<Self, ParseError> {
        let reader = BufReader::new(r);

        let tiles: Vec<Vec<Tile>> = reader
            .lines()
            .map(|l| {
                let line = l.map_err(ParseError::IO)?;
                line.chars()
                    .map(|c| c.try_into())
                    .collect::<Result<Vec<_>, _>>()
            })
            .collect::<Result<Vec<_>, _>>()?;

        if tiles.is_empty() {
            return Err(ParseError::EmptyInput);
        }

        let height = tiles.len();
        let width = tiles[0].len();

        if tiles.iter().any(|v| v.len() != width) {
            return Err(ParseError::UnevenGrid);
        }

        Ok(Self {
            tiles,
            width,
            height,
        })
    }

    pub fn tilt_north(&mut self) {
        for y in 1..self.height {
            for x in 0..self.width {
                let curr = &self.tiles[y][x];

                if *curr != Tile::Round {
                    continue;
                }

                let mut dist = 0;
                for y_check in (0..y).rev() {
                    if self.tiles[y_check][x] == Tile::Empty {
                        dist += 1;
                    } else {
                        break;
                    }
                }

                if dist > 0 {
                    self.tiles[y - dist][x] = Tile::Round;
                    self.tiles[y][x] = Tile::Empty;
                }
            }
        }
    }

    #[allow(clippy::needless_range_loop)]
    pub fn rotated_cw_90(&self) -> Self {
        let mut tiles = vec![vec![Tile::Empty; self.height]; self.width];

        for x in 0..self.width {
            for y in 0..self.height {
                tiles[y][x] = self.tiles[self.width - x - 1][y];
            }
        }

        Self {
            tiles,
            width: self.height,
            height: self.width,
        }
    }

    fn cycled_once(&self) -> Self {
        let mut p: Platform = (*self).clone();

        for _ in 0..4 {
            p.tilt_north();
            p = p.rotated_cw_90();
        }

        p
    }

    pub fn cycled_many(&self, cycles: u64) -> Self {
        let mut p: Platform = (*self).clone();

        let mut configs = HashMap::new();

        let mut loop_period = 0;
        let mut curr = 0;

        for i in 0..cycles {
            curr = i;

            if let Some(idx) = configs.get(&String::from(&p)) {
                loop_period = i - idx;
                break;
            }

            configs.insert(String::from(&p), i);
            p = p.cycled_once();
        }

        if loop_period == 0 {
            return p;
        }

        let remainder = (cycles - curr) % loop_period;

        for _ in 0..remainder {
            p = p.cycled_once();
        }

        p
    }

    pub fn total_weight(&self) -> usize {
        self.tiles
            .iter()
            .enumerate()
            .map(|(y, row)| {
                row.iter()
                    .filter(|t| *t == &Tile::Round)
                    .map(|_| self.height - y)
                    .sum::<usize>()
            })
            .sum()
    }
}

impl From<&Platform> for String {
    fn from(val: &Platform) -> Self {
        val.tiles
            .iter()
            .map(|row| {
                row.iter()
                    .map(|t| {
                        let c: char = (*t).into();
                        c
                    })
                    .collect::<String>()
            })
            .collect::<Vec<String>>()
            .join("\n")
    }
}

impl Display for Platform {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(self))
    }
}

impl From<Tile> for char {
    fn from(value: Tile) -> Self {
        match value {
            Tile::Round => 'O',
            Tile::Cube => '#',
            Tile::Empty => '.',
        }
    }
}

impl TryFrom<char> for Tile {
    type Error = ParseError;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            'O' => Ok(Tile::Round),
            '#' => Ok(Tile::Cube),
            '.' => Ok(Tile::Empty),
            _ => Err(ParseError::UnknownCharacter(c)),
        }
    }
}

#[derive(Debug)]
pub enum ParseError {
    IO(io::Error),
    EmptyInput,
    UnknownCharacter(char),
    UnevenGrid,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnknownCharacter(c) => write!(f, "Unknown character: {c}"),
            ParseError::EmptyInput => write!(f, "Expected lines"),
            ParseError::UnevenGrid => write!(f, "Expected even grid"),
            ParseError::IO(e) => write!(f, "Could not read file: {e}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stringreader::StringReader;

    #[test]
    fn tilt_north_example() {
        let input = "O....#....
O.OO#....#
.....##...
OO.#O....O
.O.....O#.
O.#..O.#.#
..O..#O..O
.......O..
#....###..
#OO..#....";

        let expectation = "OOOO.#.O..
OO..#....#
OO..O##..O
O..#.OO...
........#.
..#....#.#
..O..#.O.O
..O.......
#....###..
#....#....";

        let r = StringReader::new(input);

        let mut platform = Platform::from_stream(r).unwrap();

        platform.tilt_north();

        let output = String::from(&platform);

        assert_eq!(output, expectation)
    }

    #[test]
    fn total_weight_example() {
        let input = "OOOO.#.O..
OO..#....#
OO..O##..O
O..#.OO...
........#.
..#....#.#
..O..#.O.O
..O.......
#....###..
#....#....";

        let r = StringReader::new(input);

        let platform = Platform::from_stream(r).unwrap();

        let total_weight = platform.total_weight();

        assert_eq!(136, total_weight)
    }

    #[test]
    fn rotated_cw_90_example() {
        let input = "OOOO.#.O..
OO..#....#
OO..O##..O
O..#.OO...
........#.
..#....#.#
..O..#.O.O
..O.......
#....###..
#....#....";

        let expectation = "##....OOOO
.......OOO
..OO#....O
......#..O
.......O#.
##.#..O#.#
.#....O#..
.#.O#....O
.....#....
...O#..O#.";

        let r = StringReader::new(input);

        let platform = Platform::from_stream(r).unwrap();

        let rotated = platform.rotated_cw_90();

        let output = String::from(&rotated);

        assert_eq!(output, expectation)
    }

    #[test]
    fn cycle_example() {
        let input = "OOOO.#.O..
OO..#....#
OO..O##..O
O..#.OO...
........#.
..#....#.#
..O..#.O.O
..O.......
#....###..
#....#....";

        let cycle1_expectation = ".....#....
....#...O#
...OO##...
.OO#......
.....OOO#.
.O#...O#.#
....O#....
......OOOO
#...O###..
#..OO#....";

        let cycle2_expectation = ".....#....
....#...O#
.....##...
..O#......
.....OOO#.
.O#...O#.#
....O#...O
.......OOO
#..OO###..
#.OOO#...O";

        let cycle3_expectation = ".....#....
....#...O#
.....##...
..O#......
.....OOO#.
.O#...O#.#
....O#...O
.......OOO
#...O###.O
#.OOO#...O";

        let r = StringReader::new(input);

        let platform = Platform::from_stream(r).unwrap();
        let cycle1 = platform.cycled_once();
        let cycle2 = platform.cycled_many(2);
        let cycle3 = platform.cycled_many(3);

        assert_eq!(cycle1_expectation, String::from(&cycle1));
        assert_eq!(cycle2_expectation, String::from(&cycle2));
        assert_eq!(cycle3_expectation, String::from(&cycle3));
    }

    #[test]
    fn billion_cycles_example() {
        let input = "O....#....
O.OO#....#
.....##...
OO.#O....O
.O.....O#.
O.#..O.#.#
..O..#O..O
.......O..
#....###..
#OO..#....";

        let r = StringReader::new(input);

        let mut platform = Platform::from_stream(r).unwrap();

        let cycled = platform.cycled_many(1_000_000_000);

        let total_weight = cycled.total_weight();

        assert_eq!(64, total_weight);
    }
}
