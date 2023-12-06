use lazy_static::lazy_static;
use regex::Regex;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io::{BufRead, BufReader, Read};

#[derive(Debug)]
pub struct Almanac {
    seed_ranges: Vec<SeedRange>,
    connections: Vec<Connection>,
}

#[derive(Debug)]
struct SeedRange {
    start: u64,
    count: u64,
}

#[derive(Debug)]
struct Connection {
    src: String,
    dst: String,
    exceptions: Vec<ConnectionException>,
}

#[derive(Debug, Clone)]
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
            seed_ranges: vec![],
            connections: vec![],
        };

        let reader = BufReader::new(input);

        for line_result in reader.lines() {
            let line = line_result.map_err(AlmanacParseError::IoError)?;

            if line.is_empty() {
                continue;
            }

            if almanac.seed_ranges.is_empty() {
                if let Some(seeds) = Self::parse_seeds(&line) {
                    almanac.seed_ranges = seeds;
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

        if almanac.seed_ranges.is_empty() {
            return Err(AlmanacParseError::FormError(
                "Seeds ranges empty".to_string(),
            ));
        }

        if almanac.connections.is_empty() {
            return Err(AlmanacParseError::FormError(
                "Connections empty".to_string(),
            ));
        }

        Ok(almanac)
    }

    pub fn find_location(&self, seed: u64) -> u64 {
        Self::propagate_forward(seed, self.connections.iter())
    }

    pub fn find_seed(&self, location: u64) -> u64 {
        Self::propagate_backwards(location, self.connections.iter())
    }

    fn propagate_forward<'a>(start: u64, connections: impl Iterator<Item = &'a Connection>) -> u64 {
        let mut curr = start;
        for c in connections {
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

    fn propagate_backwards<'a>(
        start: u64,
        connections: impl DoubleEndedIterator<Item = &'a Connection>,
    ) -> u64 {
        let mut curr = start;
        for c in connections.rev() {
            let prev = if let Some(ex) = c
                .exceptions
                .iter()
                .find(|e| curr >= e.dst && curr < e.dst + e.count)
            {
                ex.src + (curr - ex.dst)
            } else {
                curr
            };

            println!("{} {} -> {} {}", c.dst, curr, c.src, prev);
            curr = prev
        }
        println!("Found seed {}\n", curr);
        curr
    }

    #[allow(unused)]
    pub fn closest_seed_loc_reverse(&self) -> Option<u64> {
        if self.connections.is_empty() {
            return None;
        }

        let loc_conn = self.connections.last().unwrap();

        let mut exceptions = loc_conn.exceptions.clone();
        exceptions.sort_by(|a, b| b.dst.cmp(&a.dst));

        let last_ex = exceptions.first().unwrap();
        let max_loc = last_ex.dst + last_ex.count;

        (0..max_loc).find(|loc| {
            let s = self.find_seed(*loc);
            self.seed_ranges
                .iter()
                .any(|r| s >= r.start && s < r.start + r.count)
        })
    }

    pub fn closest_seed_loc_optimized(&self) -> Option<u64> {
        if self.connections.is_empty() {
            return None;
        }

        println!("------- Building keypoints -------");
        let mut key_points: Vec<u64> = vec![];

        let loc_key_points: Vec<u64> = self
            .connections
            .last()
            .unwrap()
            .exceptions
            .iter()
            .flat_map(|e| vec![e.dst, e.dst + e.count])
            .collect();

        key_points.push(0); // ensure 0 is in.
        key_points.extend(loc_key_points); // add the location connection's native ranges

        println!("Added native location key points: {:?}", key_points);

        // propagate each connection's key points forwards to locations
        // to find key locations at the end
        let mut key_points: Vec<_> = self
            .connections
            .iter()
            .enumerate()
            .flat_map(|(i, c)| {
                println!("------- Find projections from {} -------", c.src);
                let mut points: Vec<_> = c
                    .exceptions
                    .iter()
                    .flat_map(|e| vec![e.src, e.src + e.count])
                    .map(|p| (i, p))
                    .collect();

                if points[0].1 != 0 {
                    points.push((i, 0));
                }

                points.into_iter()
            })
            .map(move |(i, p)| Self::propagate_forward(p, self.connections.iter().skip(i)))
            .collect();

        key_points.sort_unstable();
        key_points.dedup();

        println!("------- Checking seed ranges -------");
        key_points
            .windows(2)
            .flat_map(|w| {
                let seed_start = self.find_seed(w[0]);
                let seed_end = self.find_seed(w[1]);

                self.seed_ranges.iter().filter_map(move |r| {
                    let intersection = seed_start.max(r.start)..seed_end.min(r.start + r.count);
                    if intersection.end > intersection.start {
                        println!("Found intersecting range {:?}", intersection);
                        Some(vec![intersection.start, intersection.end])
                    } else {
                        None
                    }
                })
            })
            .flatten()
            .map(|s| self.find_location(s))
            .min()
    }

    fn parse_seeds(line: &str) -> Option<Vec<SeedRange>> {
        let cap = SEEDS_REGEX.captures(line)?;
        Some(
            cap[1]
                .split(' ')
                .map(|part| part.trim().parse::<u64>().unwrap())
                .collect::<Vec<u64>>()
                .chunks(2)
                .map(|c| SeedRange {
                    start: c[0],
                    count: c[1],
                })
                .collect(),
        )
    }

    fn parse_header(line: &str) -> Option<Connection> {
        let cap = CONNECTION_HEADER_REGEX.captures(line)?;
        Some(Connection {
            src: cap[1].to_string(),
            dst: cap[2].to_string(),
            exceptions: vec![],
        })
    }

    fn parse_exception(line: &str) -> Option<ConnectionException> {
        let cap = CONNECTION_EXCEPTION_REGEX.captures(line)?;
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

    lazy_static! {
        static ref INPUT: String = "seeds: 79 14 55 13

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
56 93 4"
            .to_string();
    }

    #[test]
    fn integration_reverse_brute() {
        let a = Almanac::parse_from_stream(&mut StringReader::new(&INPUT)).unwrap();
        let closest_loc = a.closest_seed_loc_reverse().unwrap();
        assert_eq!(46, closest_loc);
    }

    #[test]
    fn integration_optimized() {
        let a = Almanac::parse_from_stream(&mut StringReader::new(&INPUT)).unwrap();
        let closest_loc = a.closest_seed_loc_optimized().unwrap();

        assert_eq!(46, closest_loc);
    }
}
