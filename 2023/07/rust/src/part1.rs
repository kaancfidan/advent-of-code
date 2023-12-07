use crate::poker::{Card, ComparableHand, Hand, HandType};
use std::collections::HashMap;

impl Card {
    pub fn value(&self) -> u8 {
        match self {
            Card::Num(n) => *n,
            Card::T => 10,
            Card::J => 11,
            Card::Q => 12,
            Card::K => 13,
            Card::A => 14,
        }
    }
}

impl ComparableHand for Hand {
    fn cards(&self) -> [Card; 5] {
        self.cards
    }

    fn hand_type(&self) -> HandType {
        let count_map = self.cards.iter().fold(HashMap::new(), |mut acc, c| {
            *acc.entry(c.value()).or_insert(0) += 1;
            acc
        });

        let mut card_counts: Vec<_> = count_map.values().collect();

        card_counts.sort_by(|a, b| b.cmp(a));

        match card_counts[..] {
            [5, ..] => HandType::FiveOfAKind,
            [4, 1, ..] => HandType::FourOfAKind,
            [3, 2, ..] => HandType::FullHouse,
            [3, 1, 1, ..] => HandType::ThreeOfAKind,
            [2, 2, 1, ..] => HandType::TwoPairs,
            [2, 1, 1, 1, ..] => HandType::OnePair,
            _ => HandType::HighCard,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("AAAAA", HandType::FiveOfAKind)]
    #[case("AA8AA", HandType::FourOfAKind)]
    #[case("23332", HandType::FullHouse)]
    #[case("TTT98", HandType::ThreeOfAKind)]
    #[case("23432", HandType::TwoPairs)]
    #[case("A23A4", HandType::OnePair)]
    #[case("23456", HandType::HighCard)]
    #[case("725A2", HandType::OnePair)]
    fn choose_hand_type(#[case] input: &str, #[case] expected_type: HandType) {
        let hand = input.parse::<Hand>().unwrap();
        assert_eq!(expected_type, hand.hand_type());
    }

    #[rstest]
    #[case("AK972", "AK857")]
    #[case("22222", "AAAA2")]
    #[case("33332", "2AAAA")]
    #[case("77888", "77788")]
    fn hand_comparison(#[case] winner: &str, #[case] loser: &str) {
        let w = &winner.parse::<Hand>().unwrap() as &dyn ComparableHand;
        let l = &loser.parse::<Hand>().unwrap() as &dyn ComparableHand;
        assert!(w > l)
    }
}
