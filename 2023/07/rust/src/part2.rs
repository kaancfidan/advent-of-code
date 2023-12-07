use crate::poker::{Card, ComparableHand, Hand, HandType};
use std::collections::HashMap;

impl Card {
    pub fn value(&self) -> u8 {
        match self {
            Card::J => 1,
            Card::Num(n) => *n,
            Card::T => 10,
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

        let joker_count = count_map.get(&Card::J.value()).unwrap_or(&0);

        if joker_count == &5 {
            return HandType::FiveOfAKind;
        }

        let filtered_map: HashMap<&u8, &i32> = count_map
            .iter()
            .filter(|&(key, _)| key != &Card::J.value())
            .collect();

        let mut card_counts: Vec<i32> = filtered_map.values().map(|i| **i).collect();

        card_counts.sort_by(|a, b| b.cmp(a));
        *card_counts.get_mut(0).unwrap() += joker_count;

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
    #[case("AAJJA", HandType::FiveOfAKind)]
    #[case("JA8AA", HandType::FourOfAKind)]
    #[case("23J32", HandType::FullHouse)]
    #[case("TTJ98", HandType::ThreeOfAKind)]
    #[case("234J2", HandType::ThreeOfAKind)]
    #[case("A23J4", HandType::OnePair)]
    #[case("23456", HandType::HighCard)]
    #[case("725AJ", HandType::OnePair)]
    fn choose_hand_type(#[case] input: &str, #[case] expected_type: HandType) {
        let hand = input.parse::<Hand>().unwrap();
        assert_eq!(expected_type, hand.hand_type());
    }

    #[rstest]
    #[case("KTJJT", "KK677")]
    #[case("AK972", "AK857")]
    #[case("22222", "AAAA2")]
    #[case("33332", "2AAAA")]
    #[case("77888", "77788")]
    fn hand_comparison(#[case] winner: &str, #[case] loser: &str) {
        let w = &winner.parse::<Hand>().unwrap() as &dyn ComparableHand;
        let l = &loser.parse::<Hand>().unwrap() as &dyn ComparableHand;
        assert!(w > l)
    }

    #[rstest]
    #[case(HandType::FiveOfAKind, HandType::FourOfAKind)]
    #[case(HandType::FourOfAKind, HandType::FullHouse)]
    #[case(HandType::FullHouse, HandType::ThreeOfAKind)]
    #[case(HandType::ThreeOfAKind, HandType::TwoPairs)]
    #[case(HandType::TwoPairs, HandType::OnePair)]
    #[case(HandType::OnePair, HandType::HighCard)]
    fn type_comparison(#[case] winner: HandType, #[case] loser: HandType) {
        assert!(winner > loser)
    }
}
