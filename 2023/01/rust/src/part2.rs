use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    static ref NUMBERS: HashMap<&'static str, u32> = {
        let mut m = HashMap::new();
        m.insert("zero", 0);
        m.insert("one", 1);
        m.insert("two", 2);
        m.insert("three", 3);
        m.insert("four", 4);
        m.insert("five", 5);
        m.insert("six", 6);
        m.insert("seven", 7);
        m.insert("eight", 8);
        m.insert("nine", 9);
        m
    };
}

fn to_digit(input: &str, pos: usize) -> Option<u32> {
    let sub: String = input.chars().skip(pos).take(5).collect();
    NUMBERS.keys().find_map(|k| {
        if sub.starts_with(k) {
            Some(NUMBERS[k])
        } else {
            None
        }
    })
}

pub fn extract_code(input: String) -> Option<u32> {
    let chars: Vec<(usize, char)> = input.chars().enumerate().collect();

    let first: Option<u32> = chars
        .iter()
        .find_map(|(i, c)| c.to_digit(10).or(to_digit(&input, *i)));

    let last: Option<u32> = chars
        .iter()
        .rev()
        .find_map(|(i, c)| c.to_digit(10).or(to_digit(&input, *i)));

    match (first, last) {
        (Some(f), Some(l)) => Some(10 * f + l),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("two1nine", 29)]
    #[case("eightwothree", 83)]
    #[case("abcone2threexyz", 13)]
    #[case("xtwone3four", 24)]
    #[case("4nineeightseven2", 42)]
    #[case("zoneight234", 14)]
    #[case("7pqrstsixteen", 76)]
    fn examples(#[case] input: String, #[case] expected: u32) {
        assert_eq!(Some(expected), extract_code(input));
    }
}
