use lazy_static::lazy_static;
use regex::Regex;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

pub struct Record {
    parts: Vec<Part>,
    seq: Vec<u8>,
}

#[derive(Clone)]
struct Part {
    damaged: Option<bool>,
}

impl Record {
    pub fn valid_configurations(&self) -> Vec<Vec<bool>> {
        Self::explore_configurations(
            &[],
            &self
                .parts
                .iter()
                .map(|p| p.damaged)
                .collect::<Vec<Option<bool>>>(),
            &self.seq,
        )
    }

    fn explore_configurations(
        decided: &[bool],
        undecided: &[Option<bool>],
        seq: &[u8],
    ) -> Vec<Vec<bool>> {
        if undecided.is_empty() {
            return if Self::is_valid(decided, seq) {
                vec![decided.to_vec()]
            } else {
                vec![]
            };
        }

        // early prune inconsistencies
        if Self::is_invalid(decided, seq) {
            return vec![];
        }

        let next_decided: Vec<_> = undecided
            .iter()
            .take_while(|d| d.is_some())
            .map(|d| d.unwrap())
            .collect();

        if next_decided.len() == undecided.len() {
            return Self::explore_configurations(
                &decided
                    .iter()
                    .cloned()
                    .chain(next_decided)
                    .collect::<Vec<bool>>(),
                &[],
                seq,
            );
        }

        let next_undecided: Vec<_> = undecided
            .iter()
            .cloned()
            .skip(next_decided.len() + 1)
            .collect();

        Self::explore_configurations(
            &decided
                .iter()
                .cloned()
                .chain(
                    next_decided
                        .iter()
                        .cloned()
                        .chain(vec![true])
                        .collect::<Vec<bool>>(),
                )
                .collect::<Vec<bool>>(),
            &next_undecided,
            seq,
        )
        .into_iter()
        .chain(Self::explore_configurations(
            &decided
                .iter()
                .cloned()
                .chain(
                    next_decided
                        .iter()
                        .cloned()
                        .chain(vec![false])
                        .collect::<Vec<bool>>(),
                )
                .collect::<Vec<bool>>(),
            &next_undecided,
            seq,
        ))
        .collect()
    }

    pub fn display(config: &[bool]) -> String {
        config
            .iter()
            .map(|c| if *c { '#' } else { '.' })
            .collect::<String>()
    }

    fn is_valid(config: &[bool], seq: &[u8]) -> bool {
        let config_seq = &Self::find_sequences(config);
        seq == config_seq
    }

    fn is_invalid(config: &[bool], seq: &[u8]) -> bool {
        let config_seq = &Self::find_sequences(config);

        if config_seq.is_empty() {
            return false;
        }

        if config_seq.len() > seq.len() {
            return true;
        }

        let (last, prev) = config_seq.split_last().unwrap();

        !seq.starts_with(prev)
            || seq[config_seq.len() - 1] < *last
            || (seq[config_seq.len() - 1] > *last && !*config.last().unwrap())
    }

    fn find_sequences(config: &[bool]) -> Vec<u8> {
        config
            .split(|&b| !b)
            .filter_map(|seq| {
                let len = seq.len();
                (len > 0).then_some(len as u8)
            })
            .collect()
    }

    pub fn unfold(&mut self) {
        let unknown = Part::try_from('?').unwrap();

        let unfolded_parts: Vec<_> = vec![
            self.parts.to_vec(),
            vec![unknown.clone()],
            self.parts.to_vec(),
            vec![unknown.clone()],
            self.parts.to_vec(),
            vec![unknown.clone()],
            self.parts.to_vec(),
            vec![unknown.clone()],
            self.parts.to_vec(),
        ]
        .into_iter()
        .flatten()
        .collect();

        self.parts = unfolded_parts;

        let unfolded_seq: Vec<_> = vec![
            self.seq.iter().cloned(),
            self.seq.iter().cloned(),
            self.seq.iter().cloned(),
            self.seq.iter().cloned(),
            self.seq.iter().cloned(),
        ]
        .into_iter()
        .flatten()
        .collect();

        self.seq = unfolded_seq;
    }
}

impl FromStr for Record {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let cap = INVENTORY_REGEX.captures(s).ok_or(ParseError::InvalidForm)?;

        let parts: Vec<Part> = cap[1]
            .chars()
            .map(|c| c.try_into())
            .collect::<Result<Vec<_>, _>>()?;

        let seq: Vec<u8> = cap[2]
            .split(',')
            .map(|s| s.parse::<u8>().map_err(|_| ParseError::InvalidForm))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Record { parts, seq })
    }
}

impl TryFrom<char> for Part {
    type Error = ParseError;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        let damaged = match c {
            '.' => Some(false),
            '#' => Some(true),
            '?' => None,
            _ => return Err(ParseError::UnknownCharacter(c)),
        };

        Ok(Part { damaged })
    }
}

lazy_static! {
    static ref INVENTORY_REGEX: Regex = Regex::new(r"^([#\.\?]+) ((\d+,)*\d+)$").unwrap();
}

#[derive(Debug)]
pub enum ParseError {
    UnknownCharacter(char),
    InvalidForm,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnknownCharacter(c) => write!(f, "Unknown character: {}", c),
            ParseError::InvalidForm => write!(
                f,
                "Lines are expected to conform to '^([#\\.\\?]+) ((\\d+,)*\\d+)$'"
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("???.### 1,1,3", 1)]
    #[case(".??..??...?##. 1,1,3", 4)]
    #[case("?#?#?#?#?#?#?#? 1,3,1,6", 1)]
    #[case("????.#...#... 4,1,1", 1)]
    #[case("????.######..#####. 1,6,5", 4)]
    #[case("?###???????? 3,2,1", 10)]
    #[case("?.#.????.. 1,2", 3)]
    #[case("??????.???##??#?? 2,2,7", 14)]
    #[case("?#?????#.??#?#.????? 1,4,3,1,1,1", 7)]
    #[case(".#????..??.??# 4,1,3", 2)]
    fn integration(#[case] input: &str, #[case] expectation: usize) {
        let record = input.parse::<Record>().unwrap();

        let valid_configs = record.valid_configurations();

        println!("{}", input);
        for config in &valid_configs {
            println!("{}", Record::display(config))
        }

        assert_eq!(expectation, valid_configs.len())
    }
}
