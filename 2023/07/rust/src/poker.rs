use lazy_static::lazy_static;
use regex::Regex;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

pub trait ComparableHand {
    fn cards(&self) -> [Card; 5];
    fn hand_type(&self) -> HandType;
}

pub struct Hand {
    pub cards: [Card; 5],
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum HandType {
    HighCard,
    OnePair,
    TwoPairs,
    ThreeOfAKind,
    FullHouse,
    FourOfAKind,
    FiveOfAKind,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Card {
    Num(u8),
    T,
    J,
    Q,
    K,
    A,
}

impl Card {
    fn from_char(c: char) -> Result<Card, UnknownCardError> {
        match c {
            '0'..='9' => Ok(Card::Num(c.to_digit(10).unwrap() as u8)),
            'T' => Ok(Card::T),
            'J' => Ok(Card::J),
            'Q' => Ok(Card::Q),
            'K' => Ok(Card::K),
            'A' => Ok(Card::A),
            _ => Err(UnknownCardError { card: c }),
        }
    }
}

impl PartialEq<Self> for Card {
    fn eq(&self, other: &Self) -> bool {
        self.value() == other.value()
    }
}

impl Eq for Card {}

impl PartialOrd for Card {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Card {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value().cmp(&other.value())
    }
}

impl PartialEq<Self> for dyn ComparableHand {
    fn eq(&self, other: &Self) -> bool {
        self.cards().eq(&other.cards())
    }
}

impl Eq for dyn ComparableHand {}

impl PartialOrd for dyn ComparableHand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for dyn ComparableHand {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.hand_type().cmp(&other.hand_type()) {
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
            Ordering::Equal => self
                .cards()
                .iter()
                .zip(&other.cards())
                .map(|(c1, c2)| c1.cmp(c2))
                .find(|o| !o.is_eq())
                .unwrap_or(Ordering::Equal),
        }
    }
}

#[derive(Debug)]
pub struct UnknownCardError {
    card: char,
}

#[derive(Debug)]
pub enum HandParseError {
    UnknownCardError(UnknownCardError),
    FormError(),
}

lazy_static! {
    static ref HAND_REGEX: Regex = Regex::new(r"^[2-9TJQKA]{5}$").unwrap();
}

impl Display for HandParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            HandParseError::FormError() => {
                write!(f, "Hands are expected to be in form ^[2-9TJQKA]{{5}}$")
            }
            HandParseError::UnknownCardError(e) => write!(f, "Unknown card specified: {}", e.card),
        }
    }
}

impl FromStr for Hand {
    type Err = HandParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !crate::poker::HAND_REGEX.is_match(s) {
            return Err(HandParseError::FormError());
        }

        let cards: [Card; 5] = s
            .chars()
            .map(|c| Card::from_char(c).map_err(HandParseError::UnknownCardError))
            .collect::<Result<Vec<Card>, HandParseError>>()?
            .try_into()
            .unwrap();

        Ok(Hand { cards })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("AAAAA", vec ! [Card::A, Card::A, Card::A, Card::A, Card::A])]
    #[case("AA8AA", vec ! [Card::A, Card::A, Card::Num(8), Card::A, Card::A])]
    #[case("23332", vec ! [Card::Num(2), Card::Num(3), Card::Num(3), Card::Num(3), Card::Num(2)])]
    #[case("TTT98", vec ! [Card::T, Card::T, Card::T, Card::Num(9), Card::Num(8)])]
    #[case("23432", vec ! [Card::Num(2), Card::Num(3), Card::Num(4), Card::Num(3), Card::Num(2)])]
    #[case("A23A4", vec ! [Card::A, Card::Num(2), Card::Num(3), Card::A, Card::Num(4)])]
    #[case("23456", vec ! [Card::Num(2), Card::Num(3), Card::Num(4), Card::Num(5), Card::Num(6)])]
    #[case("725A2", vec ! [Card::Num(7), Card::Num(2), Card::Num(5), Card::A, Card::Num(2)])]
    fn parsing(#[case] input: &str, #[case] expected_cards: Vec<Card>) {
        let hand = input.parse::<Hand>().unwrap();
        assert_eq!(expected_cards, hand.cards);
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
