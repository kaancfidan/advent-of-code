use std::io::{self, BufRead, Read};

pub struct Schematic {
    pub numbers: Vec<Number>,
    pub symbols: Vec<Symbol>,
}

#[derive(Debug, PartialEq)]
pub struct Symbol {
    pub row: u32,
    pub col: u32,
    pub char: char,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Number {
    pub row: u32,
    pub col_min: u32,
    pub col_max: u32,
    pub value: u64,
}

impl Schematic {
    pub fn parse_from_stream(input: impl Read) -> Result<Schematic, String> {
        let mut s = Schematic {
            numbers: vec![],
            symbols: vec![],
        };

        let reader = io::BufReader::new(input);

        for (row, line) in reader.lines().enumerate() {
            if line.is_err() {
                return Err(line.err().unwrap().to_string());
            }

            let chars = Self::enumerate_chars(line.unwrap());

            let numbers = Self::parse_numbers(row as u32, chars.clone());
            s.numbers.extend(numbers);

            let symbols = Self::parse_symbols(row as u32, chars);
            s.symbols.extend(symbols);
        }

        Ok(s)
    }

    fn enumerate_chars(line: String) -> Vec<(u32, char)> {
        line.chars()
            .enumerate()
            .map(|(i, c)| (i as u32, c))
            .collect::<Vec<(u32, char)>>()
    }

    fn parse_numbers(row: u32, chars: Vec<(u32, char)>) -> Vec<Number> {
        let numeric: Vec<(u32, char)> = chars
            .into_iter()
            .filter(|(_, c)| c.is_numeric())
            .collect::<Vec<_>>();

        if numeric.len() == 1 {
            return vec![Number {
                value: numeric[0].1.to_digit(10).unwrap() as u64,
                col_min: numeric[0].0,
                col_max: numeric[0].0,
                row,
            }];
        }

        let last_index = numeric.iter().map(|(i, _)| i).last().unwrap_or(&0u32);

        numeric
            .windows(2)
            .fold(vec![], |acc: Vec<Vec<(u32, char)>>, w| {
                let prev = acc.last().and_then(|v| v.last());
                let curr: (u32, char) = w[0];
                let next: (u32, char) = w[1];

                let new_acc: Vec<Vec<(u32, char)>> = if prev.is_none() || prev.unwrap().0 != curr.0
                {
                    let new_group = if next.0 == curr.0 + 1 {
                        vec![curr, next]
                    } else {
                        vec![curr]
                    };

                    acc.into_iter().chain(vec![new_group]).collect()
                } else if next.0 == curr.0 + 1 {
                    let len = acc.len();
                    let mut curr_group: Vec<(u32, char)> = acc.last().unwrap().clone();
                    curr_group.push(next);

                    acc.into_iter()
                        .take(len - 1)
                        .chain(vec![curr_group])
                        .collect()
                } else {
                    acc
                };

                if next.0 == *last_index && new_acc.iter().last().unwrap().last().unwrap() != &next
                {
                    let last_group = vec![next];
                    new_acc.into_iter().chain(vec![last_group]).collect()
                } else {
                    new_acc
                }
            })
            .into_iter()
            .map(|chars| Number {
                value: chars
                    .iter()
                    .map(|c| c.1.to_digit(10).unwrap())
                    .enumerate()
                    .map(|(i, d)| 10u64.pow((chars.len() - i) as u32 - 1) * d as u64)
                    .sum(),
                col_min: chars.iter().map(|c| c.0).min().unwrap(),
                col_max: chars.iter().map(|c| c.0).max().unwrap(),
                row,
            })
            .collect()
    }

    fn parse_symbols(row: u32, chars: Vec<(u32, char)>) -> Vec<Symbol> {
        chars
            .into_iter()
            .filter(|(_, c)| !c.is_numeric() && *c != '.')
            .map(|(i, c)| Symbol {
                col: i,
                row,
                char: c,
            })
            .collect()
    }

    pub fn find_part_numbers(&self) -> impl Iterator<Item = Number> + '_ {
        self.numbers
            .iter()
            .filter(|n| {
                let rows = (n.row as i32 - 1)..=(n.row as i32 + 1);
                let cols = (n.col_min as i32 - 1)..=(n.col_max as i32 + 1);
                let coords: Vec<_> = rows
                    .flat_map(|r| cols.clone().map(move |c| (r, c)))
                    .collect();

                self.symbols.iter().any(|s| {
                    coords
                        .iter()
                        .any(|(r, c)| s.row as i32 == *r && s.col as i32 == *c)
                })
            })
            .copied()
    }

    pub fn find_gear_ratios(&self) -> impl Iterator<Item = u64> + '_ {
        self.symbols
            .iter()
            .filter(|s| s.char == '*')
            .map(|s| {
                let rows = (s.row as i32 - 1)..=(s.row as i32 + 1);
                let cols = (s.col as i32 - 1)..=(s.col as i32 + 1);

                let coords: Vec<_> = rows
                    .flat_map(|r| cols.clone().map(move |c| (r, c)))
                    .collect();

                self.numbers
                    .iter()
                    .filter(|n| {
                        coords.iter().any(|(r, c)| {
                            n.row as i32 == *r && n.col_min as i32 <= *c && n.col_max as i32 >= *c
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .filter(|p| p.len() == 2)
            .map(|p| p[0].value * p[1].value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use stringreader::StringReader;

    #[rstest]
    fn integration_part_numbers() {
        let example: &str = "467..114..\n\
                             ...*......\n\
                             ..35..633.\n\
                             ......#...\n\
                             617*......\n\
                             .....+.58.\n\
                             ..592.....\n\
                             ......755.\n\
                             ...$.*....\n\
                             .664.598..";

        let schematic = Schematic::parse_from_stream(StringReader::new(example));
        let sum: u64 = schematic
            .unwrap()
            .find_part_numbers()
            .map(|n| n.value)
            .sum();

        assert_eq!(4361, sum)
    }

    #[rstest]
    fn integration_gears() {
        let example: &str = "467..114..\n\
                             ...*......\n\
                             ..35..633.\n\
                             ......#...\n\
                             617*......\n\
                             .....+.58.\n\
                             ..592.....\n\
                             ......755.\n\
                             ...$.*....\n\
                             .664.598..";

        let schematic = Schematic::parse_from_stream(StringReader::new(example));
        let sum: u64 = schematic.unwrap().find_gear_ratios().sum();

        assert_eq!(467835, sum)
    }

    #[rstest]
    #[case("467..114..", 0, vec ! [Number{value: 467, row: 0, col_min: 0, col_max: 2}, Number{value: 114, row: 0, col_min: 5, col_max: 7}])]
    #[case("...*......", 1, vec ! [])]
    #[case("..35..633.", 2, vec ! [Number{value: 35, row: 2, col_min: 2, col_max: 3}, Number{value: 633, row: 2, col_min: 6, col_max: 8}])]
    #[case("......#...", 3, vec ! [])]
    #[case("617*......", 4, vec ! [Number{value: 617, row: 4, col_min: 0, col_max: 2}])]
    #[case(".....+.58.", 5, vec ! [Number{value: 58, row: 5, col_min: 7, col_max: 8}])]
    #[case("..592.....", 6, vec ! [Number{value: 592, row: 6, col_min: 2, col_max: 4}])]
    #[case("......755.", 7, vec ! [Number{value: 755, row: 7, col_min: 6, col_max: 8}])]
    #[case("...$.*....", 8, vec ! [])]
    #[case(".664.598..", 9, vec ! [Number{value: 664, row: 9, col_min: 1, col_max: 3}, Number{value: 598, row: 9, col_min: 5, col_max: 7}])]
    #[case("0.........", 10, vec ! [Number{value: 0, row: 10, col_min: 0, col_max: 0}])]
    #[case(".1.3......", 11, vec ! [Number{value: 1, row: 11, col_min: 1, col_max: 1}, Number{value: 3, row: 11, col_min: 3, col_max: 3}])]
    #[case(".........9", 12, vec ! [Number{value: 9, row: 12, col_min: 9, col_max: 9}])]
    fn parse_numbers(#[case] input: String, #[case] row: u32, #[case] expected: Vec<Number>) {
        let chars = Schematic::enumerate_chars(input);
        assert_eq!(expected, Schematic::parse_numbers(row, chars));
    }

    #[rstest]
    #[case("467..114..", 0, vec ! [])]
    #[case("...*......", 1, vec ! [Symbol{row: 1, col: 3, char:'*'}])]
    #[case("..35..633.", 2, vec ! [])]
    #[case("......#...", 3, vec ! [Symbol{row: 3, col: 6, char:'#'}])]
    #[case("617*......", 4, vec ! [Symbol{row: 4, col: 3, char:'*'}])]
    #[case(".....+.58.", 5, vec ! [Symbol{row: 5, col: 5, char:'+'}])]
    #[case("..592.....", 6, vec ! [])]
    #[case("......755.", 7, vec ! [])]
    #[case("...$.*....", 8, vec ! [Symbol{row: 8, col: 3, char:'$'}, Symbol{row: 8, col: 5, char:'*'}])]
    #[case(".664.598..", 9, vec ! [])]
    fn parse_symbols(#[case] input: String, #[case] row: u32, #[case] expected: Vec<Symbol>) {
        let chars = Schematic::enumerate_chars(input);
        assert_eq!(expected, Schematic::parse_symbols(row, chars));
    }
}
