use std::num::ParseIntError;

pub fn parse_nums(lines: Vec<&str>) -> Result<Vec<Vec<u64>>, ParseIntError> {
    let num_results: Result<Vec<Vec<u64>>, _> = lines
        .into_iter()
        .map(|l| {
            let res = l
                .split_whitespace()
                .skip(1)
                .collect::<Vec<&str>>()
                .join("")
                .parse::<u64>();

            match res {
                Ok(num) => Ok(vec![num]),
                Err(e) => Err(e),
            }
        })
        .collect();

    num_results
}
