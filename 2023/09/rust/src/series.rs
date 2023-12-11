use std::fmt::{Display, Formatter};
use std::num::ParseIntError;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub struct Series {
    pub levels: Vec<Vec<i64>>,
}

impl Series {
    fn diff(series: &[i64]) -> Vec<i64> {
        series.windows(2).map(|w| w[1] - w[0]).collect()
    }

    pub fn extrapolate_forward(&mut self) {
        if self.levels.is_empty() {
            return;
        }

        let last_value = *self.levels.last().unwrap().last().unwrap();
        self.levels.last_mut().unwrap().push(last_value);

        for i in (0..self.levels.len() - 1).rev() {
            let sum = self.levels[i].last().unwrap() + self.levels[i + 1].last().unwrap();
            self.levels[i].push(sum)
        }
    }

    pub fn extrapolate_backwards(&mut self) {
        if self.levels.is_empty() {
            return;
        }

        let last_value = *self.levels.last().unwrap().first().unwrap();
        self.levels.last_mut().unwrap().insert(0, last_value);

        for i in (0..self.levels.len() - 1).rev() {
            let sum = self.levels[i].first().unwrap() - self.levels[i + 1].first().unwrap();
            self.levels[i].insert(0, sum)
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    NotAnInteger(ParseIntError),
    Empty,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::NotAnInteger(e) => write!(f, "Expected integer: {}", e),
            ParseError::Empty => write!(f, "Empty line provided"),
        }
    }
}

impl FromStr for Series {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let top = s
            .split_whitespace()
            .map(|s| str::parse(s).map_err(ParseError::NotAnInteger))
            .collect::<Result<Vec<i64>, _>>()?;

        if top.is_empty() {
            return Err(ParseError::Empty);
        }

        let levels = std::iter::successors(Some(top), |last_level| {
            let next_level = Series::diff(last_level);
            (!next_level.iter().all(|&v| v == 0)).then_some(next_level)
        })
        .collect::<Vec<_>>();

        Ok(Series { levels })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("0 3 6 9 12 15", Series{levels: vec ! [vec ! [0, 3, 6, 9, 12, 15], vec ! [3, 3, 3, 3, 3]]})]
    #[case("1 3 6 10 15 21", Series{levels: vec ! [vec ! [1, 3, 6, 10, 15, 21], vec ! [2, 3, 4, 5, 6], vec ! [1, 1, 1, 1]]})]
    #[case("10 13 16 21 30 45", Series{levels: vec ! [vec ! [10, 13, 16, 21, 30, 45], vec ! [3, 3, 5, 9, 15], vec ! [0, 2, 4, 6], vec ! [2, 2, 2]]})]
    fn parsing(#[case] input: &str, #[case] expected: Series) {
        assert_eq!(Ok(expected), input.parse::<Series>());
    }

    #[rstest]
    #[case("0 3 6 9 12 15", "0 3 6 9 12 15 18")]
    #[case("1 3 6 10 15 21", "1 3 6 10 15 21 28")]
    #[case("10 13 16 21 30 45", "10 13 16 21 30 45 68")]
    fn forward_extrapolation(#[case] input: &str, #[case] expected: &str) {
        let mut series = input.parse::<Series>().unwrap();

        series.extrapolate_forward();

        let output: String = series.levels[0]
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(" ");

        assert_eq!(expected, output);
    }

    #[rstest]
    #[case("0 3 6 9 12 15", "-3 0 3 6 9 12 15")]
    #[case("1 3 6 10 15 21", "0 1 3 6 10 15 21")]
    #[case("10 13 16 21 30 45", "5 10 13 16 21 30 45")]
    fn backwards_extrapolation(#[case] input: &str, #[case] expected: &str) {
        let mut series = input.parse::<Series>().unwrap();

        series.extrapolate_backwards();

        let output: String = series.levels[0]
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(" ");

        assert_eq!(expected, output);
    }
}
