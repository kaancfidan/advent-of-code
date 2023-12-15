use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

pub struct Record {
    parts: Vec<Part>,
    seq: Vec<usize>,
}

#[derive(Clone)]
struct Part {
    damaged: Option<bool>,
}

impl Record {
    pub fn valid_configuration_count(&self, memo: &mut HashMap<String, u64>) -> u64 {
        // find sequences of # and ? mixes without . in them
        let seqs: Vec<Vec<Option<bool>>> = self
            .parts
            .iter()
            .map(|p| p.damaged)
            .collect::<Vec<Option<bool>>>()
            .split(|b| *b == Some(false))
            .filter(|g| !g.is_empty())
            .map(|g| g.to_vec())
            .collect();

        let ref_seq = self.seq.to_vec();

        Self::memoized_count(&seqs, &ref_seq, memo)
    }

    fn memoized_count(
        config: &[Vec<Option<bool>>],
        seq: &[usize],
        memo: &mut HashMap<String, u64>,
    ) -> u64 {
        let key = Self::encode_inputs(config, seq);
        return if let Some(res) = memo.get(&key) {
            *res
        } else {
            let res = Self::count_configurations(config, seq, memo);
            memo.insert(key, res);
            res
        };
    }

    fn count_configurations(
        curr: &[Vec<Option<bool>>],
        seq: &[usize],
        memo: &mut HashMap<String, u64>,
    ) -> u64 {
        if curr.is_empty() {
            return if seq.is_empty() { 1 } else { 0 };
        }

        // first group is all #
        if !curr[0].contains(&None) {
            // first number in ref sequence must be the same as first group's length
            return if !seq.is_empty() && seq[0] == curr[0].len() {
                Self::memoized_count(&curr[1..], &seq[1..], memo)
            } else {
                0
            };
        }

        // first sequence of #s
        let count_damaged = curr[0].iter().take_while(|b| **b == Some(true)).count();

        // first sequence of #s already larger than first number in ref sequence
        if count_damaged > 0 && (seq.is_empty() || count_damaged > seq[0]) {
            return 0;
        }

        // replace 1 unknown with damaged in first group
        let replaced_damaged = Self::replace_unknown(&curr[0], true);
        let replaced_ok = Self::replace_unknown(&curr[0], false);

        let ok_split: Vec<Vec<Option<bool>>> = replaced_ok
            .split(|b| *b == Some(false))
            .filter(|g| !g.is_empty())
            .map(|g| g.to_vec())
            .collect();

        let left = [&[replaced_damaged], &curr[1..]].concat();
        let right = [&ok_split, &curr[1..]].concat();

        Self::memoized_count(&left, seq, memo) + Self::memoized_count(&right, seq, memo)
    }

    fn encode_inputs(curr: &[Vec<Option<bool>>], seq: &[usize]) -> String {
        let encoded_curr = curr
            .iter()
            .map(|v| {
                v.iter()
                    .map(|b| match b {
                        None => '?',
                        Some(true) => '#',
                        Some(false) => '.',
                    })
                    .collect::<String>()
            })
            .collect::<Vec<String>>()
            .join(".");

        let encoded_seq = seq
            .iter()
            .map(|d| d.to_string())
            .collect::<Vec<_>>()
            .join(",");

        encoded_curr + " " + &encoded_seq
    }

    fn replace_unknown(v: &[Option<bool>], b: bool) -> Vec<Option<bool>> {
        let mut result = v.to_vec();

        if let Some(pos) = result.iter().position(|&x| x.is_none()) {
            result[pos] = Some(b);
        }

        result
    }

    pub fn unfolded(&self, times: u8) -> Record {
        let unknown = vec![Part::try_from('?').unwrap()];

        let unfolded_parts: Vec<_> = (0..times - 1).fold(self.parts.clone(), |mut acc, _| {
            acc = [&acc[..], &unknown.clone()[..], &self.parts.clone()[..]].concat();
            acc
        });

        let unfolded_seq: Vec<_> = (0..times - 1).fold(self.seq.clone(), |mut acc, _| {
            acc = [&acc[..], &self.seq.clone()[..]].concat();
            acc
        });

        Record {
            parts: unfolded_parts,
            seq: unfolded_seq,
        }
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

        let seq: Vec<_> = cap[2]
            .split(',')
            .map(|s| s.parse::<usize>().map_err(|_| ParseError::InvalidForm))
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
    fn integration_count(#[case] input: &str, #[case] expectation: u64) {
        let record = input.parse::<Record>().unwrap();

        let mut memo = HashMap::new();
        let count = record.valid_configuration_count(&mut memo);

        assert_eq!(expectation, count);
    }

    #[rstest]
    #[case("???.### 1,1,3", 1)]
    #[case(".??..??...?##. 1,1,3", 16384)]
    #[case("?#?#?#?#?#?#?#? 1,3,1,6", 1)]
    #[case("????.#...#... 4,1,1", 16)]
    #[case("????.######..#####. 1,6,5", 2500)]
    #[case("?###???????? 3,2,1", 506250)]
    fn integration_count_unfolded(#[case] input: &str, #[case] expectation: u64) {
        let record = input.parse::<Record>().unwrap();
        let unfolded = record.unfolded(5);

        let mut memo = HashMap::new();
        let count = unfolded.valid_configuration_count(&mut memo);

        assert_eq!(expectation, count);
    }
}
