use crate::workflow::Decision::{Accept, Forward, Reject};
use lazy_static::lazy_static;
use regex::Regex;
use std::cmp::Ordering;
use std::cmp::Ordering::{Greater, Less};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::io;
use std::io::{BufRead, BufReader, Read};
use std::ops::Range;
use std::str::FromStr;

#[derive(Default)]
pub struct Elves {
    workflows: HashMap<String, Workflow>,
    parts: Vec<Part>,
}

struct Workflow {
    name: String,
    conditions: Vec<Condition>,
    fallback: Decision,
}

struct Condition {
    field: char,
    order: Ordering,
    value: u64,
    decision: Decision,
}

enum Decision {
    Accept,
    Reject,
    Forward(String),
}

#[derive(Debug, Clone)]
pub struct ParameterRange {
    pub ranges: HashMap<char, Range<u64>>,
}

#[derive(Copy, Clone)]
pub struct Part {
    x: u64,
    m: u64,
    a: u64,
    s: u64,
}

impl Part {
    pub fn rating(&self) -> u64 {
        self.x + self.m + self.a + self.s
    }
}

impl Elves {
    pub fn from_instructions(r: impl Read) -> Result<Self, ParseError> {
        let reader = BufReader::new(r);
        let mut reading_workflows = true;

        let mut manual = Elves::default();

        for l in reader.lines() {
            let line = l.map_err(ParseError::IO)?;

            if line.is_empty() {
                reading_workflows = false;
                continue;
            }

            if reading_workflows {
                let w = line.parse::<Workflow>()?;
                manual.workflows.insert(w.name.clone(), w);
            } else {
                let p = line.parse::<Part>()?;
                manual.parts.push(p);
            }
        }

        Ok(manual)
    }

    pub fn check_parts(&self) -> Result<Vec<Part>, WorkflowError> {
        let accepted: Vec<Part> = self
            .parts
            .iter()
            .map(|p| {
                let is_accepted = self.check_ratings(p.x, p.m, p.a, p.s)?;

                if is_accepted {
                    Ok(Some(*p))
                } else {
                    Ok(None)
                }
            })
            .collect::<Result<Vec<Option<Part>>, _>>()?
            .into_iter()
            .flatten()
            .collect();

        Ok(accepted)
    }

    pub fn check_ratings(&self, x: u64, m: u64, a: u64, s: u64) -> Result<bool, WorkflowError> {
        let mut w = self
            .workflows
            .get("in")
            .ok_or(WorkflowError::Inconsistent("in".to_owned()))?;

        loop {
            let c = w.conditions.iter().find(|c| {
                let v = match c.field {
                    'x' => x,
                    'm' => m,
                    'a' => a,
                    's' => s,
                    _ => unreachable!("regex guard should catch this"),
                };

                v.cmp(&c.value) == c.order
            });

            let d = match c {
                Some(c) => &c.decision,
                None => &w.fallback,
            };

            match d {
                Forward(n) => {
                    w = self
                        .workflows
                        .get(n)
                        .ok_or(WorkflowError::Inconsistent(n.to_owned()))?
                }
                Accept => return Ok(true),
                Reject => return Ok(false),
            }
        }
    }

    pub fn find_valid_ranges(
        &self,
        input_range: ParameterRange,
    ) -> Result<Vec<ParameterRange>, WorkflowError> {
        let mut valid_params: Vec<ParameterRange> = Vec::new();

        let mut stack: Vec<(&str, ParameterRange)> = vec![("in", input_range)];

        while let Some((curr, mut params)) = stack.pop() {
            let w = self
                .workflows
                .get(curr)
                .ok_or(WorkflowError::Inconsistent(curr.to_owned()))?;

            for c in &w.conditions {
                let (inside, outside) = params.split(c);

                match (&c.decision, &inside) {
                    (Accept, Some(r)) => valid_params.push(r.clone()),
                    (Forward(n), Some(r)) => stack.push((&n, r.clone())),
                    _ => {}
                }

                if let Some(r) = outside {
                    params = r.clone();
                } else {
                    break;
                }
            }

            match &w.fallback {
                Accept => valid_params.push(params.clone()),
                Forward(n) => stack.push((&n, params.clone())),
                Reject => {} // don't care
            }
        }

        Ok(valid_params)
    }
}

impl ParameterRange {
    fn split(&self, condition: &Condition) -> (Option<ParameterRange>, Option<ParameterRange>) {
        let range = self.ranges.get(&condition.field).unwrap();
        let value = condition.value;

        match condition.order {
            Less if (0..range.start).contains(&value) => (None, Some(self.clone())),
            Less if (range.end..).contains(&value) => (Some(self.clone()), None),
            Less => {
                let mut inside = self.clone();
                let mut outside = self.clone();

                inside.ranges.insert(condition.field, range.start..value);
                outside.ranges.insert(condition.field, value..range.end);

                (Some(inside), Some(outside))
            }
            Greater if (range.end..).contains(&value) => (None, Some(self.clone())),
            Greater if (0..range.start).contains(&value) => (Some(self.clone()), None),
            Greater => {
                let mut inside = self.clone();
                let mut outside = self.clone();

                inside.ranges.insert(condition.field, value + 1..range.end);
                outside.ranges.insert(condition.field, range.start..value + 1);

                (Some(inside), Some(outside))
            }
            _ => unreachable!("regex guard should catch this"),
        }
    }

    pub fn combinations(&self) -> u64 {
        self.ranges.values().map(|v| v.end - v.start).product()
    }
}

lazy_static! {
    static ref WORKFLOW_REGEX: Regex =
        Regex::new(r"^([a-z]+)\{((?:(?:x|m|a|s)(?:<|>)\d+:(?:A|R|[a-z]+),)*)(A|R|[a-z]+)\}$")
            .unwrap();
    static ref CONDITION_REGEX: Regex = Regex::new(r"^(x|m|a|s)(<|>)(\d+):(A|R|[a-z]+)$").unwrap();
    static ref PART_REGEX: Regex = Regex::new(r"^\{x=(\d+),m=(\d+),a=(\d+),s=(\d+)\}$").unwrap();
}

impl FromStr for Workflow {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let cap = WORKFLOW_REGEX
            .captures(s)
            .ok_or(ParseError::Workflow(s.to_owned()))?;

        Ok(Workflow {
            name: cap[1].to_owned(),
            conditions: cap[2]
                .split(',')
                .filter_map(|cs| match cs {
                    "" => None,
                    _ => Some(cs.parse::<Condition>()),
                })
                .collect::<Result<_, _>>()?,
            fallback: cap[3].parse::<Decision>()?,
        })
    }
}

impl FromStr for Condition {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let cap = CONDITION_REGEX
            .captures(s)
            .ok_or(ParseError::Condition(s.to_owned()))?;

        Ok(Condition {
            field: cap[1].chars().next().unwrap(),
            order: match cap[2].chars().next().unwrap() {
                '<' => Less,
                '>' => Greater,
                _ => unreachable!("regex guard should catch this"),
            },
            value: cap[3].parse::<u64>().unwrap(),
            decision: cap[4].parse::<Decision>()?,
        })
    }
}

impl FromStr for Decision {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "A" => Accept,
            "R" => Reject,
            s => Forward(s.to_owned()),
        })
    }
}

impl FromStr for Part {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let cap = PART_REGEX
            .captures(s)
            .ok_or(ParseError::Part(s.to_owned()))?;

        Ok(Part {
            x: cap[1].parse::<u64>().unwrap(),
            m: cap[2].parse::<u64>().unwrap(),
            a: cap[3].parse::<u64>().unwrap(),
            s: cap[4].parse::<u64>().unwrap(),
        })
    }
}

#[derive(Debug)]
pub enum ParseError {
    IO(io::Error),
    Workflow(String),
    Condition(String),
    Part(String),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::IO(e) => write!(f, "Could not read file: {e}"),
            ParseError::Workflow(s) => write!(f, "Invalid workflow: {s}"),
            ParseError::Condition(s) => write!(f, "Invalid condition: {s}"),
            ParseError::Part(s) => write!(f, "Invalid part: {s}"),
        }
    }
}

#[derive(Debug)]
pub enum WorkflowError {
    Inconsistent(String),
}

impl Display for WorkflowError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkflowError::Inconsistent(s) => {
                write!(f, "Workflow {s} was referenced but does not exist")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stringreader::StringReader;

    #[test]
    fn integration() {
        let input = "px{a<2006:qkq,m>2090:A,rfg}
pv{a>1716:R,A}
lnx{m>1548:A,A}
rfg{s<537:gd,x>2440:R,A}
qs{s>3448:A,lnx}
qkq{x<1416:A,crn}
crn{x>2662:A,R}
in{s<1351:px,qqz}
qqz{s>2770:qs,m<1801:hdj,R}
gd{a>3333:R,R}
hdj{m>838:A,pv}

{x=787,m=2655,a=1222,s=2876}
{x=1679,m=44,a=2067,s=496}
{x=2036,m=264,a=79,s=2244}
{x=2461,m=1339,a=466,s=291}
{x=2127,m=1623,a=2188,s=1013}";

        let r = StringReader::new(input);
        let elves = Elves::from_instructions(r).unwrap();

        let accepted = elves.check_parts().unwrap();

        let total_rating: u64 = accepted.iter().map(|p| p.rating()).sum();

        assert_eq!(total_rating, 19114);
    }

    #[test]
    fn integration_valid_ranges() {
        let input = "px{a<2006:qkq,m>2090:A,rfg}
pv{a>1716:R,A}
lnx{m>1548:A,A}
rfg{s<537:gd,x>2440:R,A}
qs{s>3448:A,lnx}
qkq{x<1416:A,crn}
crn{x>2662:A,R}
in{s<1351:px,qqz}
qqz{s>2770:qs,m<1801:hdj,R}
gd{a>3333:R,R}
hdj{m>838:A,pv}";

        let r = StringReader::new(input);
        let elves = Elves::from_instructions(r).unwrap();

        let mut r = ParameterRange {
            ranges: HashMap::new(),
        };
        r.ranges.insert('x', 1..4001);
        r.ranges.insert('m', 1..4001);
        r.ranges.insert('a', 1..4001);
        r.ranges.insert('s', 1..4001);

        let valid_ranges = elves.find_valid_ranges(r).unwrap();

        let combinations: u64 = valid_ranges.iter().map(|r| r.combinations()).sum();

        assert_eq!(combinations, 167_409_079_868_000);
    }
}
