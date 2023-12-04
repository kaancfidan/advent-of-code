use lazy_static::lazy_static;
use regex::Regex;
use std::str::FromStr;

lazy_static! {
    static ref CARD_REGEX: Regex =
        Regex::new(r"^Card\s+(\d+):\s+((?:\d+\s+)*\d+)\s+\|\s+((?:\d+\s+)*\d+)$").unwrap();
}

#[derive(Debug, PartialEq)]
pub struct Card {
    pub id: u32,
    pub n_instances: u64,
    winning: Vec<u32>,
    played: Vec<u32>,
}

impl FromStr for Card {
    type Err = regex::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let caps = CARD_REGEX
            .captures(input)
            .ok_or(regex::Error::Syntax("Invalid format".into()))?;

        Ok(Card {
            id: caps[1].parse().unwrap(),
            winning: caps[2]
                .split_whitespace()
                .map(|s| s.parse().unwrap())
                .collect(),
            played: caps[3]
                .split_whitespace()
                .map(|s| s.parse().unwrap())
                .collect(),
            n_instances: 1,
        })
    }
}

impl Card {
    pub fn matching_count(&self) -> u32 {
        self.played
            .iter()
            .filter(|n| self.winning.contains(n))
            .count() as u32
    }

    pub fn score(&self) -> u64 {
        let matching = self.matching_count();

        if matching == 0 {
            0
        } else {
            2u64.pow(matching - 1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(Card{id: 1, n_instances: 1, winning: vec ! [41, 48, 83, 86, 17], played: vec ! [83, 86, 6, 31, 17, 9, 48, 53]}, 4)]
    #[case(Card{id: 2, n_instances: 1, winning: vec ! [13, 32, 20, 16, 61], played: vec ! [61, 30, 68, 82, 17, 32, 24, 19]}, 2)]
    #[case(Card{id: 3, n_instances: 1, winning: vec ! [1, 21, 53, 59, 44], played: vec ! [69, 82, 63, 72, 16, 21, 14, 1]}, 2)]
    #[case(Card{id: 4, n_instances: 1, winning: vec ! [41, 92, 73, 84, 69], played: vec ! [59, 84, 76, 51, 58, 5, 54, 83]}, 1)]
    #[case(Card{id: 5, n_instances: 1, winning: vec ! [87, 83, 26, 28, 32], played: vec ! [88, 30, 70, 12, 93, 22, 82, 36]}, 0)]
    #[case(Card{id: 6, n_instances: 1, winning: vec ! [31, 18, 13, 56, 72], played: vec ! [74, 77, 10, 23, 35, 67, 36, 11]}, 0)]
    fn matching_count_examples(#[case] input: Card, #[case] expected: u32) {
        assert_eq!(expected, input.matching_count());
    }

    #[rstest]
    #[case(Card{id: 1, n_instances: 1, winning: vec ! [41, 48, 83, 86, 17], played: vec ! [83, 86, 6, 31, 17, 9, 48, 53]}, 8)]
    #[case(Card{id: 2, n_instances: 1, winning: vec ! [13, 32, 20, 16, 61], played: vec ! [61, 30, 68, 82, 17, 32, 24, 19]}, 2)]
    #[case(Card{id: 3, n_instances: 1, winning: vec ! [1, 21, 53, 59, 44], played: vec ! [69, 82, 63, 72, 16, 21, 14, 1]}, 2)]
    #[case(Card{id: 4, n_instances: 1, winning: vec ! [41, 92, 73, 84, 69], played: vec ! [59, 84, 76, 51, 58, 5, 54, 83]}, 1)]
    #[case(Card{id: 5, n_instances: 1, winning: vec ! [87, 83, 26, 28, 32], played: vec ! [88, 30, 70, 12, 93, 22, 82, 36]}, 0)]
    #[case(Card{id: 6, n_instances: 1, winning: vec ! [31, 18, 13, 56, 72], played: vec ! [74, 77, 10, 23, 35, 67, 36, 11]}, 0)]
    fn score_examples(#[case] input: Card, #[case] expected: u64) {
        assert_eq!(expected, input.score());
    }

    #[rstest]
    #[case("Card 1: 41 48 83 86 17 | 83 86  6 31 17  9 48 53", Card{id: 1, n_instances: 1, winning: vec ! [41, 48, 83, 86, 17], played: vec ! [83, 86, 6, 31, 17, 9, 48, 53]})]
    #[case("Card 2: 13 32 20 16 61 | 61 30 68 82 17 32 24 19", Card{id: 2, n_instances: 1, winning: vec ! [13, 32, 20, 16, 61], played: vec ! [61, 30, 68, 82, 17, 32, 24, 19]})]
    #[case("Card 3:  1 21 53 59 44 | 69 82 63 72 16 21 14  1", Card{id: 3, n_instances: 1, winning: vec ! [1, 21, 53, 59, 44], played: vec ! [69, 82, 63, 72, 16, 21, 14, 1]})]
    #[case("Card 4: 41 92 73 84 69 | 59 84 76 51 58  5 54 83", Card{id: 4, n_instances: 1, winning: vec ! [41, 92, 73, 84, 69], played: vec ! [59, 84, 76, 51, 58, 5, 54, 83]})]
    #[case("Card 5: 87 83 26 28 32 | 88 30 70 12 93 22 82 36", Card{id: 5, n_instances: 1, winning: vec ! [87, 83, 26, 28, 32], played: vec ! [88, 30, 70, 12, 93, 22, 82, 36]})]
    #[case("Card 6: 31 18 13 56 72 | 74 77 10 23 35 67 36 11", Card{id: 6, n_instances: 1, winning: vec ! [31, 18, 13, 56, 72], played: vec ! [74, 77, 10, 23, 35, 67, 36, 11]})]
    fn parse_examples(#[case] input: String, #[case] expected: Card) {
        assert_eq!(Ok(expected), input.parse::<Card>());
    }
}
