use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::io;
use std::io::{BufRead, BufReader, Read};

pub struct Galaxy {
    pub stars: HashMap<(i64, i64), Star>,
    empty_x: HashSet<i64>,
    empty_y: HashSet<i64>,
}

pub struct Star {
    x: i64,
    y: i64,
}

impl Galaxy {
    pub fn from_stream(s: impl Read) -> Result<Galaxy, ParseError> {
        let reader = BufReader::new(s);

        let lines: Vec<(usize, String)> = reader
            .lines()
            .enumerate()
            .map(|(i, l)| {
                let line = l.map_err(ParseError::IO)?;
                Ok((i, line))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let stars = Self::parse_stars(lines)?;

        let empty_x = Self::find_empty_x(&stars);

        let empty_y = Self::find_empty_y(&stars);

        Ok(Galaxy {
            stars,
            empty_x,
            empty_y,
        })
    }

    fn parse_stars(lines: Vec<(usize, String)>) -> Result<HashMap<(i64, i64), Star>, ParseError> {
        let stars: HashMap<(i64, i64), Star> = lines
            .into_iter()
            .flat_map(|(y, s)| {
                s.chars()
                    .enumerate()
                    .filter_map(|(x, c)| match c {
                        '.' => None,
                        '#' => Some(Ok((
                            (x as i64, y as i64),
                            Star {
                                x: x as i64,
                                y: y as i64,
                            },
                        ))),
                        _ => Some(Err(ParseError::UnknownCharacter(c))),
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Result<HashMap<_, _>, _>>()?;
        Ok(stars)
    }

    fn find_empty_x(stars: &HashMap<(i64, i64), Star>) -> HashSet<i64> {
        let x_min = stars
            .keys()
            .min_by(|a, b| a.0.cmp(&b.0))
            .unwrap_or(&(0, 0))
            .0;

        let x_max = stars
            .keys()
            .max_by(|a, b| a.0.cmp(&b.0))
            .unwrap_or(&(0, 0))
            .0;

        (x_min..x_max)
            .filter(|x| !stars.keys().any(|(xs, _)| xs == x))
            .collect()
    }

    fn find_empty_y(stars: &HashMap<(i64, i64), Star>) -> HashSet<i64> {
        let y_min = stars
            .keys()
            .min_by(|a, b| a.1.cmp(&b.1))
            .unwrap_or(&(0, 0))
            .1;

        let y_max = stars
            .keys()
            .max_by(|a, b| a.1.cmp(&b.1))
            .unwrap_or(&(0, 0))
            .1;

        (y_min..y_max)
            .filter(|y| !stars.keys().any(|(_, ys)| ys == y))
            .collect()
    }

    pub fn calculate_total_distance(&self, universe_age: u64) -> u64 {
        self.stars
            .values()
            .enumerate()
            .flat_map(|(i1, s1)| {
                self.stars
                    .values()
                    .enumerate()
                    .filter(move |(i2, _)| i1 < *i2)
                    .map(|(_, s2)| self.measure_distance(s1, s2, universe_age))
            })
            .sum()
    }

    fn measure_distance(&self, star1: &Star, star2: &Star, universe_age: u64) -> u64 {
        let x_dist = star1.x.abs_diff(star2.x);
        let y_dist = star1.y.abs_diff(star2.y);

        let x_expansion = (star1.x.min(star2.x)..star1.x.max(star2.x))
            .filter(|x| self.empty_x.contains(x))
            .count() as u64
            * (universe_age - 1);

        let y_expansion = (star1.y.min(star2.y)..star1.y.max(star2.y))
            .filter(|y| self.empty_y.contains(y))
            .count() as u64
            * (universe_age - 1);

        x_dist + x_expansion + y_dist + y_expansion
    }
}

#[derive(Debug)]
pub(crate) enum ParseError {
    IO(io::Error),
    UnknownCharacter(char),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::IO(e) => write!(f, "Could not read file: {}", e),
            ParseError::UnknownCharacter(c) => write!(f, "Unknown character: {}", c),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stringreader::StringReader;

    #[test]
    fn parsing_happy() {
        let input = "...#......
.......#..
#.........
..........
......#...
.#........
.........#
..........
.......#..
#...#.....";

        let galaxy = Galaxy::from_stream(StringReader::new(input)).unwrap();

        assert_eq!(3, galaxy.empty_x.len());
        assert!(galaxy.empty_x.contains(&2));
        assert!(galaxy.empty_x.contains(&5));
        assert!(galaxy.empty_x.contains(&8));

        assert_eq!(2, galaxy.empty_y.len());
        assert!(galaxy.empty_y.contains(&3));
        assert!(galaxy.empty_y.contains(&7));
    }

    #[test]
    fn calculate_total_distance_part1() {
        let input = "...#......
.......#..
#.........
..........
......#...
.#........
.........#
..........
.......#..
#...#.....";

        let galaxy = Galaxy::from_stream(StringReader::new(input)).unwrap();

        let sum: u64 = galaxy.calculate_total_distance(2);

        assert_eq!(374, sum);
    }

    #[test]
    fn calculate_total_distance_part2() {
        let input = "...#......
.......#..
#.........
..........
......#...
.#........
.........#
..........
.......#..
#...#.....";

        let galaxy = Galaxy::from_stream(StringReader::new(input)).unwrap();

        let sum10: u64 = galaxy.calculate_total_distance(10);
        assert_eq!(1030, sum10);

        let sum100 = galaxy.calculate_total_distance(100);
        assert_eq!(8410, sum100);
    }

    #[test]
    fn parsing_unknown_char() {
        let input = "...K....";

        let galaxy = Galaxy::from_stream(StringReader::new(input));

        assert!(galaxy.is_err())
    }
}
