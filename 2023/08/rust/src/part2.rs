use crate::common::{Node, Step, Steps};
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::io::{BufRead, BufReader, Read};
use std::{fmt, io};

pub struct Map {
    steps: Vec<Step>,
    nodes: HashMap<String, Node>,
    start_keys: Vec<String>,
    end_keys: HashSet<String>,
}

#[derive(Debug)]
struct Loop {
    offset: u64,
    length: u64,
    stops: Vec<u64>,
}

struct StopIter {
    loops: Vec<Loop>,
    curr: Vec<u64>,
    next: Vec<u64>,
    i: Vec<usize>,
    is_first: bool,
}

impl StopIter {
    fn nth_stop(l: &Loop, n: usize) -> u64 {
        let len = l.stops.len();
        let rounds = (n / len) as u64;
        let pos = l.stops[n % len];

        l.offset + rounds * l.length + pos
    }

    fn new(loops: Vec<Loop>) -> StopIter {
        let len = loops.len();

        StopIter {
            loops,
            curr: vec![0; len],
            next: vec![0; len],
            i: vec![0; len],
            is_first: true,
        }
    }
}

impl Iterator for StopIter {
    type Item = Vec<u64>;

    fn next(&mut self) -> Option<Vec<u64>> {
        if self.is_first {
            self.curr = self.loops.iter().map(|l| Self::nth_stop(l, 0)).collect();
            self.next = self.loops.iter().map(|l| Self::nth_stop(l, 1)).collect();
            self.is_first = false;
        } else {
            let smallest_stop = self
                .next
                .iter()
                .enumerate()
                .min_by_key(|&(_, val)| val)
                .unwrap()
                .0;

            self.i[smallest_stop] += 1;
            self.curr[smallest_stop] = self.next[smallest_stop];
            self.next[smallest_stop] =
                Self::nth_stop(&self.loops[smallest_stop], self.i[smallest_stop]);
        }

        Some(self.curr.clone())
    }
}

impl Map {
    fn is_start_key(key: &str) -> bool {
        key.ends_with('A')
    }

    fn is_end_key(key: &str) -> bool {
        key.ends_with('Z')
    }

    pub fn parse_from_stream(input: impl Read) -> Result<Map, MapParseError> {
        let reader = BufReader::new(input);
        let mut lines = reader.lines();

        let first_line = match lines.next() {
            Some(Ok(line)) => line,
            Some(Err(e)) => return Err(MapParseError::IO(e)),
            None => return Err(MapParseError::ExpectedStepsLine),
        };

        let steps = first_line
            .parse::<Steps>()
            .map_err(|_| MapParseError::ExpectedStepsLine)?
            .0;

        if steps.is_empty() {
            return Err(MapParseError::ExpectedStepsLine);
        }

        let nodes: HashMap<String, Node> = lines
            .skip(1)
            .map(|l| match l {
                Ok(line) => line
                    .parse::<Node>()
                    .map_err(|_| MapParseError::ExpectedNodeLine),
                Err(e) => Err(MapParseError::IO(e)),
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|n| (n.key.clone(), n))
            .collect();

        let start_keys: Vec<_> = nodes
            .keys()
            .filter(|k| Self::is_start_key(k))
            .cloned()
            .collect();

        if start_keys.is_empty() {
            return Err(MapParseError::MissingStartNode);
        }

        let end_keys: HashSet<_> = nodes
            .keys()
            .filter(|k| Self::is_end_key(k))
            .cloned()
            .collect();

        if end_keys.is_empty() {
            return Err(MapParseError::MissingEndNode);
        }

        Ok(Map {
            steps,
            nodes,
            start_keys,
            end_keys,
        })
    }

    pub fn count_steps(&self) -> u64 {
        let mut count: u64 = 0;
        let mut curr: Vec<&Node> = self.start_keys.iter().map(|k| &self.nodes[k]).collect();

        while !curr.iter().all(|n| self.end_keys.contains(&n.key)) {
            let step = &self.steps[count as usize % self.steps.len()];

            curr = curr
                .into_iter()
                .map(|n| match step {
                    Step::Left => &self.nodes[&n.left],
                    Step::Right => &self.nodes[&n.right],
                })
                .collect();

            count += 1;
        }

        count
    }

    pub fn count_steps_optimized(&self) -> u64 {
        // find loops for each starting point and note down stops
        let loops = self.find_loops();

        println!("{:?}", loops);

        // this is actually an under-determined system of linear equations
        // number of starting points n - 1 equations & n variables (it's periodic)
        // there are an infinite number solutions to it
        // and finding the smallest integer solution is an NP-hard problem.
        // I'm just going to search by jumping between candidates and hope for the best.
        let mut iterator = StopIter::new(loops);

        loop {
            let stops: HashSet<u64> = iterator.next().unwrap().into_iter().collect();
            if stops.len() == 1 {
                return *stops.iter().next().unwrap();
            }
        }
    }

    // greatest common denominator
    fn gcd(mut a: u64, mut b: u64) -> u64 {
        while b != 0 {
            let t = b;
            b = a % b;
            a = t;
        }
        a
    }

    // least common multiple
    fn lcm(a: u64, b: u64) -> u64 {
        a / Self::gcd(a, b) * b
    }

    // this solution uses the fact that there is always 1 stop at -offset at the end of the period
    // it's not a generalized solution
    pub fn calculate_steps(&self) -> Result<u64, &str> {
        // find loops for each starting point and note down stops
        let loops = self.find_loops();

        // check assumption
        if loops
            .iter()
            .any(|l| l.stops.len() > 1 || l.length - l.stops[0] != l.offset)
        {
            return Err("input is not well-aligned with assumptions");
        }

        Ok(loops.iter().fold(1, |mut acc, l| Self::lcm(acc, l.length)))
    }

    fn find_loops(&self) -> Vec<Loop> {
        let loops: Vec<_> = self
            .start_keys
            .iter()
            .map(|k| &self.nodes[k])
            .map(|n| {
                let mut stops: Vec<u64> = vec![];
                let mut visited: HashMap<(String, usize), u64> = HashMap::new();

                let mut curr = n;
                let mut count: u64 = 0;
                let mut curr_step_offset: usize = 0;

                while count == 0 || !visited.contains_key(&(curr.key.clone(), curr_step_offset)) {
                    visited.insert((curr.key.clone(), curr_step_offset), count);

                    if Self::is_end_key(&curr.key) {
                        stops.push(count)
                    }

                    curr_step_offset = count as usize % self.steps.len();
                    curr = match self.steps[curr_step_offset] {
                        Step::Left => &self.nodes[&curr.left],
                        Step::Right => &self.nodes[&curr.right],
                    };

                    count += 1;
                }

                let offset = visited[&(curr.key.clone(), curr_step_offset)];

                Loop {
                    offset,
                    length: count - offset,
                    stops: stops.iter().map(|s| s - offset).collect(),
                }
            })
            .collect();
        loops
    }
}

#[derive(Debug)]
pub enum MapParseError {
    IO(io::Error),
    ExpectedStepsLine,
    ExpectedNodeLine,
    MissingStartNode,
    MissingEndNode,
}

impl Display for MapParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            MapParseError::IO(e) => write!(f, "error while reading input: {}", e),
            MapParseError::ExpectedStepsLine => write!(f, "expected steps line"),
            MapParseError::ExpectedNodeLine => write!(f, "expected node line"),
            MapParseError::MissingStartNode => write!(f, "there are no start nodes"),
            MapParseError::MissingEndNode => write!(f, "there are no end nodes"),
        }
    }
}

impl std::error::Error for MapParseError {}

#[cfg(test)]
mod tests {
    use super::*;
    use lazy_static::lazy_static;
    use stringreader::StringReader;

    lazy_static! {
        static ref INPUT: String = String::from(
            "LR

11A = (11B, XXX)
11B = (XXX, 11Z)
11Z = (11B, XXX)
22A = (22B, XXX)
22B = (22C, 22C)
22C = (22Z, 22Z)
22Z = (22B, 22B)
XXX = (XXX, XXX)"
        );
    }

    #[test]
    fn integration_count() {
        let map = Map::parse_from_stream(StringReader::new(&INPUT)).unwrap();
        let count = map.count_steps();
        assert_eq!(6, count);
    }

    #[test]
    fn integration_count_optimized() {
        let map = Map::parse_from_stream(StringReader::new(&INPUT)).unwrap();
        let count = map.count_steps_optimized();
        assert_eq!(6, count);
    }

    #[test]
    fn integration_calculate() {
        let map = Map::parse_from_stream(StringReader::new(&INPUT)).unwrap();
        let res = map.calculate_steps();
        assert_eq!(Err("input is not well-aligned with assumptions"), res);
    }
}
