use either::{Either, Left, Right};
use std::fmt::{Display, Formatter};

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
enum Tile {
    Ash,
    Rock,
}

#[derive(Clone)]
pub struct Valley {
    tiles: Vec<Tile>,
    height: usize,
    width: usize,
}

impl TryFrom<char> for Tile {
    type Error = ParseError;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            '.' => Ok(Tile::Ash),
            '#' => Ok(Tile::Rock),
            _ => Err(ParseError::UnknownCharacter(c)),
        }
    }
}

type HorizontalMirror = usize;
type VerticalMirror = usize;

impl Valley {
    pub fn smudged_mirror_pos(&self) -> Option<Either<HorizontalMirror, VerticalMirror>> {
        (0..self.width)
            .flat_map(move |x| (0..self.height).map(move |y| (x, y)))
            .find_map(|(x, y)| {
                let mut clone = self.clone();

                let i = x + y * self.width;
                clone.tiles[i] = match clone.tiles[i] {
                    Tile::Ash => Tile::Rock,
                    Tile::Rock => Tile::Ash,
                };

                let old_pos = self.mirror_pos(None);
                let new_pos = clone.mirror_pos(old_pos);

                if new_pos.is_some() {
                    new_pos
                } else {
                    None
                }
            })
    }

    pub fn mirror_pos(
        &self,
        except: Option<Either<HorizontalMirror, VerticalMirror>>,
    ) -> Option<Either<HorizontalMirror, VerticalMirror>> {
        let horizontal = self
            .rows()
            .enumerate()
            .find(|(y, _)| {
                if let Some(Left(e)) = except {
                    if e == *y + 1 {
                        return false;
                    }
                }

                let before = self
                    .rows()
                    .rev()
                    .skip(self.height - y - 1)
                    .collect::<Vec<_>>();
                let after = self.rows().skip(*y + 1).collect::<Vec<_>>();

                !after.is_empty() && before.starts_with(&after) || after.starts_with(&before)
            })
            .map(|v| v.0 + 1);

        if let Some(h) = horizontal {
            return Some(Left(h as HorizontalMirror));
        }

        let vertical = self
            .cols()
            .enumerate()
            .find(|(x, _)| {
                if let Some(Right(e)) = except {
                    if e == *x + 1 {
                        return false;
                    }
                }

                let before = self
                    .cols()
                    .rev()
                    .skip(self.width - x - 1)
                    .collect::<Vec<_>>();
                let after = self.cols().skip(*x + 1).collect::<Vec<_>>();
                !after.is_empty() && before.starts_with(&after) || after.starts_with(&before)
            })
            .map(|h| h.0 + 1);

        if let Some(v) = vertical {
            return Some(Right(v as VerticalMirror));
        }

        None
    }

    fn rows(&self) -> impl DoubleEndedIterator<Item = Vec<Tile>> {
        self.tiles
            .chunks(self.width)
            .map(|c| c.to_vec())
            .collect::<Vec<_>>()
            .into_iter()
    }

    fn cols(&self) -> impl DoubleEndedIterator<Item = Vec<Tile>> {
        (0..self.width)
            .map(move |x| {
                (0..self.height)
                    .map(move |y| self.tiles[x + y * self.width])
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
            .into_iter()
    }
}

impl TryFrom<&[&str]> for Valley {
    type Error = ParseError;

    fn try_from(lines: &[&str]) -> Result<Self, Self::Error> {
        if lines.is_empty() {
            return Err(ParseError::EmptyInput);
        }

        let tiles: Vec<Vec<Tile>> = lines
            .iter()
            .map(|l| {
                l.chars()
                    .map(|c| c.try_into())
                    .collect::<Result<Vec<_>, _>>()
            })
            .collect::<Result<Vec<_>, _>>()?;

        let n_rows = tiles.len();
        let n_cols = tiles[0].len();

        if tiles.iter().any(|v| v.len() != n_cols) {
            return Err(ParseError::UnevenGrid);
        }

        Ok(Self {
            tiles: tiles.into_iter().flatten().collect(),
            height: n_rows,
            width: n_cols,
        })
    }
}

#[derive(Debug)]
pub enum ParseError {
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example1() {
        let input = "#.##..##.
..#.##.#.
##......#
##......#
..#.##.#.
..##..##.
#.#.##.#.";

        let lines = &input.split('\n').collect::<Vec<_>>();

        let valley = Valley::try_from(&lines[..]).unwrap();

        let p = valley.mirror_pos(None);
        assert_eq!(Some(Right(5)), p);
    }

    #[test]
    fn example2() {
        let input = "#...##..#
#....#..#
..##..###
#####.##.
#####.##.
..##..###
#....#..#";

        let lines = &input.split('\n').collect::<Vec<_>>();

        let valley = Valley::try_from(&lines[..]).unwrap();

        let p = valley.mirror_pos(None);
        assert_eq!(Some(Left(4)), p);
    }

    #[test]
    fn smudged_example1() {
        let input = "#.##..##.
..#.##.#.
##......#
##......#
..#.##.#.
..##..##.
#.#.##.#.";

        let lines = &input.split('\n').collect::<Vec<_>>();

        let valley = Valley::try_from(&lines[..]).unwrap();

        let p = valley.smudged_mirror_pos();

        assert_eq!(Some(Left(3)), p);
    }

    #[test]
    fn smudged_example2() {
        let input = "#...##..#
#....#..#
..##..###
#####.##.
#####.##.
..##..###
#....#..#";

        let lines = &input.split('\n').collect::<Vec<_>>();

        let valley = Valley::try_from(&lines[..]).unwrap();

        let p = valley.smudged_mirror_pos();

        assert_eq!(Some(Left(1)), p);
    }

    #[test]
    fn smudged_example3() {
        let input = "#...##...#....##.
..#.##.#..###....
..##..##..##.####
...####.....##..#
#.######.#....###
###.##.####..#..#
##..##..##.##.##.
#........##.#####
...........#.#..#";

        let lines = &input.split('\n').collect::<Vec<_>>();

        let valley = Valley::try_from(&lines[..]).unwrap();

        let p = valley.smudged_mirror_pos();

        assert_eq!(Some(Right(15)), p);
    }
}
