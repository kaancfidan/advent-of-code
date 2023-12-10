use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::io::{BufRead, BufReader, Read};
use std::{fmt, io};

use crate::common::{Node, Step, Steps};

pub struct Map {
    steps: Vec<Step>,
    nodes: HashMap<String, Node>,
}

impl Map {
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

        if !nodes.contains_key("AAA") {
            return Err(MapParseError::MissingNode(String::from("AAA")));
        }

        if !nodes.contains_key("ZZZ") {
            return Err(MapParseError::MissingNode(String::from("ZZZ")));
        }

        Ok(Map { steps, nodes })
    }

    pub fn count_steps(&self) -> u64 {
        let mut curr = &self.nodes["AAA"];
        let mut count: u64 = 0;

        while curr.key != "ZZZ" {
            let step = &self.steps[count as usize % self.steps.len()];

            curr = match step {
                Step::Left => &self.nodes[&curr.left],
                Step::Right => &self.nodes[&curr.right],
            };

            count += 1;
        }

        count
    }
}

#[derive(Debug)]
pub enum MapParseError {
    IO(io::Error),
    ExpectedStepsLine,
    ExpectedNodeLine,
    MissingNode(String),
}

impl Display for MapParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            MapParseError::IO(e) => write!(f, "error while reading input: {}", e),
            MapParseError::ExpectedStepsLine => write!(f, "expected steps line"),
            MapParseError::ExpectedNodeLine => write!(f, "expected node line"),
            MapParseError::MissingNode(s) => write!(f, "missing node: {}", s),
        }
    }
}

impl std::error::Error for MapParseError {}

#[cfg(test)]
mod tests {
    use super::*;
    use stringreader::StringReader;

    #[test]
    fn integration_example1() {
        let input = "RL

AAA = (BBB, CCC)
BBB = (DDD, EEE)
CCC = (ZZZ, GGG)
DDD = (DDD, DDD)
EEE = (EEE, EEE)
GGG = (GGG, GGG)
ZZZ = (ZZZ, ZZZ)";

        let map = Map::parse_from_stream(StringReader::new(input)).unwrap();
        let count = map.count_steps();
        assert_eq!(2, count);
    }

    #[test]
    fn integration_example2() {
        let input = "LLR

AAA = (BBB, BBB)
BBB = (AAA, ZZZ)
ZZZ = (ZZZ, ZZZ)";

        let map = Map::parse_from_stream(StringReader::new(input)).unwrap();
        let count = map.count_steps();
        assert_eq!(6, count);
    }
}
