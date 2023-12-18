use either::{Either, Left, Right};
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::io;
use std::io::{BufRead, BufReader, Read};

pub struct Room {
    tiles: Vec<Vec<Tile>>,
    width: usize,
    height: usize,
}

struct Tile {
    instrument: Option<Box<dyn Instrument>>,
}

trait Instrument {
    fn direct_light(&self, dir: Direction) -> Either<Direction, [Direction; 2]>;
}

enum Mirror {
    NorthWest,
    SouthWest,
}

enum Splitter {
    Horizontal,
    Vertical,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

impl Instrument for Mirror {
    fn direct_light(&self, dir: Direction) -> Either<Direction, [Direction; 2]> {
        match (self, dir) {
            (Mirror::NorthWest, Direction::North) => Left(Direction::East),
            (Mirror::NorthWest, Direction::East) => Left(Direction::North),
            (Mirror::NorthWest, Direction::South) => Left(Direction::West),
            (Mirror::NorthWest, Direction::West) => Left(Direction::South),
            (Mirror::SouthWest, Direction::North) => Left(Direction::West),
            (Mirror::SouthWest, Direction::East) => Left(Direction::South),
            (Mirror::SouthWest, Direction::South) => Left(Direction::East),
            (Mirror::SouthWest, Direction::West) => Left(Direction::North),
        }
    }
}

impl Instrument for Splitter {
    fn direct_light(&self, dir: Direction) -> Either<Direction, [Direction; 2]> {
        match (self, dir) {
            (Splitter::Vertical, Direction::North | Direction::South) => Left(dir),
            (Splitter::Vertical, Direction::West | Direction::East) => {
                Right([Direction::North, Direction::South])
            }
            (Splitter::Horizontal, Direction::West | Direction::East) => Left(dir),
            (Splitter::Horizontal, Direction::North | Direction::South) => {
                Right([Direction::West, Direction::East])
            }
        }
    }
}

impl Room {
    pub fn from_stream(r: impl Read) -> Result<Self, ParseError> {
        let reader = BufReader::new(r);

        let tiles: Vec<Vec<Tile>> = reader
            .lines()
            .map(|l| {
                let line = l.map_err(ParseError::IO)?;
                line.chars().map(|c| c.try_into()).collect::<Result<_, _>>()
            })
            .collect::<Result<Vec<_>, _>>()?;

        let height = tiles.len();
        let width = tiles[0].len();

        if tiles.iter().any(|row| row.len() != width) {
            return Err(ParseError::UnevenGrid);
        }

        Ok(Room {
            tiles,
            width,
            height,
        })
    }

    pub fn energized(&self, pos: (i32, i32), dir: Direction) -> usize {
        let mut visited = HashSet::new();

        let mut beams = vec![(pos, dir)];

        while let Some(beam) = beams.pop() {
            let mut curr_pos = beam.0;
            let mut curr_dir = beam.1;

            while visited.get(&(curr_pos, curr_dir)).is_none()
                && curr_pos.0 >= 0
                && curr_pos.0 < self.height as i32
                && curr_pos.1 >= 0
                && curr_pos.1 < self.width as i32
            {
                visited.insert((curr_pos, curr_dir));

                let curr_ins = &self.tiles[curr_pos.0 as usize][curr_pos.1 as usize].instrument;

                curr_dir = match curr_ins {
                    None => curr_dir,
                    Some(i) => match i.direct_light(curr_dir) {
                        Left(dir) => dir,
                        Right(dirs) => {
                            beams.push((curr_pos, dirs[1]));
                            dirs[0]
                        }
                    },
                };

                match curr_dir {
                    Direction::North => curr_pos.0 -= 1,
                    Direction::East => curr_pos.1 += 1,
                    Direction::South => curr_pos.0 += 1,
                    Direction::West => curr_pos.1 -= 1,
                }
            }
        }

        visited
            .iter()
            .map(|(pos, _)| *pos)
            .collect::<HashSet<_>>()
            .len()
    }

    pub fn max_energized(&self) -> usize {
        let top: Vec<_> = vec![0; self.width]
            .into_iter()
            .zip(0..self.width as i32)
            .collect();

        let left: Vec<_> = (0..self.height as i32).zip(vec![0; self.height]).collect();

        let bottom: Vec<_> = vec![self.height as i32; self.width]
            .into_iter()
            .zip(0..self.width as i32)
            .collect();

        let right: Vec<_> = (0..self.height as i32)
            .zip(vec![self.width as i32; self.height])
            .collect();

        let top_illu = top
            .iter()
            .map(|pos| self.energized(*pos, Direction::South))
            .max()
            .unwrap();

        let left_illu = left
            .iter()
            .map(|pos| self.energized(*pos, Direction::East))
            .max()
            .unwrap();

        let bottom_illu = bottom
            .iter()
            .map(|pos| self.energized(*pos, Direction::North))
            .max()
            .unwrap();

        let right_illu = right
            .iter()
            .map(|pos| self.energized(*pos, Direction::West))
            .max()
            .unwrap();

        *[top_illu, left_illu, bottom_illu, right_illu]
            .iter()
            .max()
            .unwrap()
    }
}

impl TryFrom<char> for Tile {
    type Error = ParseError;

    fn try_from(sym: char) -> Result<Self, Self::Error> {
        let instrument: Option<Box<dyn Instrument>> = match sym {
            '.' => None,
            '|' => Some(Box::new(Splitter::Vertical)),
            '-' => Some(Box::new(Splitter::Horizontal)),
            '/' => Some(Box::new(Mirror::NorthWest)),
            '\\' => Some(Box::new(Mirror::SouthWest)),
            _ => return Err(ParseError::UnknownCharacter(sym)),
        };

        Ok(Self { instrument })
    }
}

#[derive(Debug)]
pub enum ParseError {
    IO(io::Error),
    UnknownCharacter(char),
    UnevenGrid,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::IO(e) => write!(f, "Could not read file: {e}"),
            ParseError::UnknownCharacter(c) => write!(f, "Unknown character: {c}"),
            ParseError::UnevenGrid => write!(f, "Expected even grid"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stringreader::StringReader;

    #[test]
    fn top_left_energize() {
        let input = ".|...\\....
|.-.\\.....
.....|-...
........|.
..........
.........\\
..../.\\\\..
.-.-/..|..
.|....-|.\\
..//.|....";

        let reader = StringReader::new(input);
        let room = Room::from_stream(reader).unwrap();

        let energized = room.energized((0, 0), Direction::East);

        assert_eq!(46, energized);
    }

    #[test]
    fn max_energized() {
        let input = ".|...\\....
|.-.\\.....
.....|-...
........|.
..........
.........\\
..../.\\\\..
.-.-/..|..
.|....-|.\\
..//.|....";

        let reader = StringReader::new(input);
        let room = Room::from_stream(reader).unwrap();

        let energized = room.max_energized();

        assert_eq!(51, energized);
    }
}
