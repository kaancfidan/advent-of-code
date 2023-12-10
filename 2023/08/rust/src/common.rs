use lazy_static::lazy_static;
use regex::Regex;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

pub(crate) struct Steps(pub(crate) Vec<Step>);

pub enum Step {
    Left,
    Right,
}

#[derive(Debug, PartialEq)]
pub struct Node {
    pub key: String,
    pub left: String,
    pub right: String,
}

lazy_static! {
    static ref NODE_REGEX: Regex =
        Regex::new(r"^([0-9A-Z]{3}) = \(([0-9A-Z]{3}), ([0-9A-Z]{3})\)$").unwrap();
}

impl FromStr for Node {
    type Err = LineParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match NODE_REGEX.captures(s) {
            Some(caps) => Ok(Node {
                key: caps[1].to_string(),
                left: caps[2].to_string(),
                right: caps[3].to_string(),
            }),
            None => Err(LineParseError::InvalidNodeLine),
        }
    }
}

impl FromStr for Steps {
    type Err = LineParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let steps: Vec<Step> = s
            .chars()
            .map(|c| match c {
                'L' => Ok(Step::Left),
                'R' => Ok(Step::Right),
                _ => Err(LineParseError::UnknownStepCharacter(c)),
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Steps(steps))
    }
}

#[derive(Debug, PartialEq)]
pub enum LineParseError {
    UnknownStepCharacter(char),
    InvalidNodeLine,
}

impl Display for LineParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            LineParseError::UnknownStepCharacter(c) => write!(f, "unknown step character: '{}'", c),
            LineParseError::InvalidNodeLine => write!(f, "invalid node line"),
        }
    }
}

impl std::error::Error for LineParseError {}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("AAA = (BBB, BBB)", Node{key: String::from("AAA"), left: String::from("BBB"), right: String::from("BBB")})]
    #[case("BBB = (AAA, ZZZ)", Node{key: String::from("BBB"), left: String::from("AAA"), right: String::from("ZZZ")})]
    #[case("ZZZ = (ZZZ, ZZZ)", Node{key: String::from("ZZZ"), left: String::from("ZZZ"), right: String::from("ZZZ")})]
    fn node_parsing(#[case] input: &str, #[case] expectation: Node) {
        let node = input.parse::<Node>();
        assert_eq!(Ok(expectation), node);
    }
}
