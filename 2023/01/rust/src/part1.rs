pub fn extract_code(input: String) -> Option<u32> {
    let first: Option<u32> = input.chars().find(|&c| c.is_numeric())?.to_digit(10);
    let last: Option<u32> = input.chars().rfind(|&c| c.is_numeric())?.to_digit(10);

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
    #[case("1abc2", 12)]
    #[case("pqr3stu8vwx", 38)]
    #[case("a1b2c3d4e5f", 15)]
    #[case("treb7uchet", 77)]
    fn examples(#[case] input: String, #[case] expected: u32) {
        assert_eq!(Some(expected), extract_code(input));
    }
}
