use lazy_static::lazy_static;
use regex::Regex;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io::{BufRead, BufReader, Read};

#[derive(Debug)]
pub struct Almanac {
    pub seeds: Vec<u64>,
    connections: Vec<Connection>,
}

#[derive(Debug)]
struct Connection {
    src: String,
    dst: String,
    exceptions: Vec<ConnectionException>,
}

#[derive(Debug)]
struct ConnectionException {
    src: u64,
    dst: u64,
    count: u64,
}

lazy_static! {
    static ref SEEDS_REGEX: Regex = Regex::new(r"^seeds: ([\d\s]*\d+)$").unwrap();
    static ref CONNECTION_HEADER_REGEX: Regex = Regex::new(r"^(\w+)-to-(\w+) map:$").unwrap();
    static ref CONNECTION_EXCEPTION_REGEX: Regex = Regex::new(r"^(\d+)\s+(\d+)\s+(\d+)$").unwrap();
}

#[derive(Debug)]
pub enum AlmanacParseError {
    IoError(std::io::Error),
    FormError(String),
}

impl Display for AlmanacParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AlmanacParseError::IoError(e) => write!(f, "Could not read: {}", e),
            AlmanacParseError::FormError(s) => write!(f, "Unexpected input: {}", s),
        }
    }
}

impl Error for AlmanacParseError {}

impl Almanac {
    pub fn parse_from_stream(input: &mut impl Read) -> Result<Almanac, AlmanacParseError> {
        let mut almanac = Almanac {
            seeds: vec![],
            connections: vec![],
        };

        let reader = BufReader::new(input);

        for line_result in reader.lines() {
            let line = line_result.map_err(AlmanacParseError::IoError)?;

            if line.is_empty() {
                continue;
            }

            if almanac.seeds.is_empty() {
                if let Some(seeds) = Self::parse_seeds(&line) {
                    almanac.seeds = seeds;
                    continue;
                } else {
                    return Err(AlmanacParseError::FormError(
                        "Expected seed list".to_string(),
                    ));
                }
            }

            if let Some(new_conn) = Self::parse_header(&line) {
                almanac.connections.push(new_conn);
                continue;
            }

            let last_conn = almanac.connections.last_mut();
            if last_conn.is_none() {
                return Err(AlmanacParseError::FormError(
                    "Expected connection header".to_string(),
                ));
            }

            if let Some(ex) = Self::parse_exception(&line) {
                last_conn.unwrap().exceptions.push(ex);
            } else {
                return Err(AlmanacParseError::FormError(
                    "Expected connection exception".to_string(),
                ));
            }
        }

        Ok(almanac)
    }

    pub fn find_location(&self, seed: u64) -> u64 {
        let mut curr = seed;
        for c in self.connections.iter() {
            let next = if let Some(ex) = c
                .exceptions
                .iter()
                .find(|e| curr >= e.src && curr < e.src + e.count)
            {
                ex.dst + (curr - ex.src)
            } else {
                curr
            };

            println!("{} {} -> {} {}", c.src, curr, c.dst, next);
            curr = next
        }
        println!("Found location {}\n", curr);
        curr
    }

    fn parse_seeds(line: &String) -> Option<Vec<u64>> {
        let cap = SEEDS_REGEX.captures(&line)?;
        Some(
            cap[1]
                .split(' ')
                .map(|part| part.trim().parse::<u64>().unwrap())
                .collect(),
        )
    }

    fn parse_header(line: &String) -> Option<Connection> {
        let cap = CONNECTION_HEADER_REGEX.captures(&line)?;
        Some(Connection {
            src: cap[1].to_string(),
            dst: cap[2].to_string(),
            exceptions: vec![],
        })
    }

    fn parse_exception(line: &String) -> Option<ConnectionException> {
        let cap = CONNECTION_EXCEPTION_REGEX.captures(&line)?;
        Some(ConnectionException {
            dst: cap[1].parse().unwrap(),
            src: cap[2].parse().unwrap(),
            count: cap[3].parse().unwrap(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stringreader::StringReader;

    #[test]
    fn integration() {
        let input = "seeds: 79 14 55 13

seed-to-soil map:
50 98 2
52 50 48

soil-to-fertilizer map:
0 15 37
37 52 2
39 0 15

fertilizer-to-water map:
49 53 8
0 11 42
42 0 7
57 7 4

water-to-light map:
88 18 7
18 25 70

light-to-temperature map:
45 77 23
81 45 19
68 64 13

temperature-to-humidity map:
0 69 1
1 0 69

humidity-to-location map:
60 56 37
56 93 4";

        let a = Almanac::parse_from_stream(&mut StringReader::new(input)).unwrap();

        assert_eq!(82, a.find_location(79));
        assert_eq!(43, a.find_location(14));
        assert_eq!(86, a.find_location(55));
        assert_eq!(35, a.find_location(13));
    }
}
