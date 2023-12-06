use std::num::ParseIntError;

#[allow(unused)]
pub fn parse_nums(lines: Vec<&str>) -> Result<Vec<Vec<u64>>, ParseIntError> {
    let num_results: Result<Vec<Vec<u64>>, _> = lines
        .into_iter()
        .map(|l| {
            l.split_whitespace()
                .skip(1)
                .map(|s| s.parse::<u64>())
                .collect()
        })
        .collect();

    num_results
}
