use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

pub struct Manual {
    steps: Vec<Step>,
}

enum Step {
    Add(Lens),
    Remove(String),
}

#[derive(Debug, Clone, PartialEq)]
struct Lens {
    label: String,
    focal: u32,
}

pub struct Configuration {
    lenses: HashMap<u8, Vec<Lens>>,
}

impl Manual {
    pub fn hash_sum(steps: &[String]) -> u64 {
        steps.iter().map(|s| Step::calculate_hash(s) as u64).sum()
    }

    pub fn create_configuration(&self) -> Configuration {
        let mut config = Configuration {
            lenses: HashMap::new(),
        };

        for step in &self.steps {
            match &step {
                Step::Add(lens) => {
                    config.add_lens(Step::calculate_hash(&lens.label), lens.clone());
                }
                Step::Remove(label) => {
                    config.remove_lens(Step::calculate_hash(label), label);
                }
            }
        }

        config
    }

    pub fn parse_line(line: &str) -> Vec<String> {
        line.split(',')
            .map(|s| {
                s.trim()
                    .chars()
                    .filter(|c| !c.is_control())
                    .collect::<String>()
            })
            .collect()
    }
}

impl Configuration {
    pub fn focusing_power(&self) -> u32 {
        self.lenses
            .iter()
            .flat_map(|(bx, list)| {
                list.iter()
                    .enumerate()
                    .map(|(slot, lens)| (*bx as u32 + 1) * (slot + 1) as u32 * lens.focal)
            })
            .sum()
    }

    fn add_lens(&mut self, hash: u8, lens: Lens) {
        let lens_box = self.lenses.entry(hash).or_default();

        if let Some(pos) = lens_box.iter().position(|l| l.label == lens.label) {
            lens_box.insert(pos, lens.clone());
            lens_box.remove(pos + 1);
        } else {
            lens_box.push(lens.clone());
        }
    }

    fn remove_lens(&mut self, hash: u8, label: &str) {
        let lens_box = self.lenses.entry(hash).or_default();

        if let Some(pos) = lens_box.iter().position(|l| l.label == label) {
            lens_box.remove(pos);
        }
    }
}

impl Step {
    fn calculate_hash(s: &str) -> u8 {
        let mut curr = 0;

        for c in s.chars() {
            curr += c as u32;
            curr *= 17;
            curr %= 256;
        }

        curr as u8
    }
}

impl FromStr for Manual {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            steps: Self::parse_line(s)
                .into_iter()
                .map(|s| s.parse::<Step>())
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

lazy_static! {
    static ref STEP_REGEX: Regex = Regex::new(r"^([a-z]+)(=|-)(\d+)?$").unwrap();
}

impl FromStr for Step {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let cap = STEP_REGEX
            .captures(s)
            .ok_or(ParseError::UnknownOperation(s.to_owned()))?;

        let step = match cap[2].to_owned().as_str() {
            "=" => Step::Add(Lens {
                label: cap[1].to_owned(),
                focal: cap[3]
                    .parse::<u32>()
                    .map_err(|_| ParseError::UnknownOperation(s.to_owned()))?,
            }),
            "-" => Step::Remove(cap[1].to_owned()),
            _ => return Err(ParseError::UnknownOperation(s.to_owned()))
        };

        Ok(step)
    }
}

#[derive(Debug)]
pub enum ParseError {
    UnknownOperation(String),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnknownOperation(s) => write!(f, "Unknown operation specification: {s}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_example() {
        let input = "HASH";
        let output = Step::calculate_hash(input);

        assert_eq!(52, output);
    }

    #[test]
    fn hash_line() {
        let input = "rn=1,cm-,qp=3,cm=2,qp-,pc=4,ot=9,ab=5,pc-,pc=6,ot=7";
        let steps: Vec<_> = Manual::parse_line(input);
        let output = Manual::hash_sum(&steps);

        assert_eq!(1320, output);
    }

    #[test]
    fn configuration() {
        let input = "rn=1,cm-,qp=3,cm=2,qp-,pc=4,ot=9,ab=5,pc-,pc=6,ot=7";
        let manual = input.parse::<Manual>().unwrap();

        let config = manual.create_configuration();

        assert_eq!(
            vec![
                Lens {
                    label: "rn".to_owned(),
                    focal: 1,
                },
                Lens {
                    label: "cm".to_owned(),
                    focal: 2,
                },
            ],
            config.lenses[&0]
        );

        assert_eq!(
            vec![
                Lens {
                    label: "ot".to_owned(),
                    focal: 7,
                },
                Lens {
                    label: "ab".to_owned(),
                    focal: 5,
                },
                Lens {
                    label: "pc".to_owned(),
                    focal: 6,
                },
            ],
            config.lenses[&3]
        );
    }

    #[test]
    fn focusing_power() {
        let input = "rn=1,cm-,qp=3,cm=2,qp-,pc=4,ot=9,ab=5,pc-,pc=6,ot=7";
        let manual = input.parse::<Manual>().unwrap();

        let config = manual.create_configuration();

        assert_eq!(145, config.focusing_power())
    }
}
