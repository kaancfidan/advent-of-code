use crate::dig::Direction::{Down, Left, Right, Up};
use crate::dig::Terrain::{Ground, Trench};
use lazy_static::lazy_static;
use regex::Regex;
use std::fmt::{Display, Formatter};
use std::io;
use std::io::{BufRead, BufReader, Read};
use std::str::FromStr;

pub struct Plan {
    instructions: Vec<Instruction>,
    handedness: Handedness,
}

enum Handedness {
    Right,
    Left,
}

#[derive(Debug, PartialEq)]
struct Instruction {
    dir: Direction,
    count: i64,
    color: String,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, PartialEq)]
enum Terrain {
    Trench,
    Ground,
}

pub struct LavaPool {
    terrain: Vec<Vec<Terrain>>,
    width: usize,
    height: usize,
}

pub struct Elves {}

impl Elves {
    fn plan_path(plan: &Plan) -> Vec<(i64, i64, Direction)> {
        let points = Self::plan_points(plan);

        points
            .windows(2)
            .zip(plan.instructions.iter().map(|i| i.dir))
            .flat_map(|(w, dir)| {
                let dug = (w[0].0.min(w[1].0)..=w[0].0.max(w[1].0)).flat_map(|x| {
                    (w[0].1.min(w[1].1)..=w[0].1.max(w[1].1)).map(move |y| (x, y, dir))
                });

                let last = dug.clone().last().unwrap();

                if last.0 == w[0].0 && last.1 == w[0].1 {
                    dug.rev().collect::<Vec<_>>()
                } else {
                    dug.collect::<Vec<_>>()
                }
            })
            .collect()
    }

    pub fn dig_sides(plan: &Plan) -> (LavaPool, (i64, i64)) {
        let path = Self::plan_path(plan);

        let x_min = path.iter().map(|p| p.0).min().unwrap();
        let x_max = path.iter().map(|p| p.0).max().unwrap();
        let y_min = path.iter().map(|p| p.1).min().unwrap();
        let y_max = path.iter().map(|p| p.1).max().unwrap();

        let width = (x_max - x_min + 1) as usize;
        let height = (y_max - y_min + 1) as usize;

        let mut terrain: Vec<Vec<Terrain>> = vec![vec![Ground; width]; height];

        for p in path {
            terrain[(p.1 - y_min) as usize][(p.0 - x_min) as usize] = Trench;
        }

        let mut inside_seed = Self::choose_inside_seed(plan);
        inside_seed.0 -= x_min;
        inside_seed.1 -= y_min;

        (
            LavaPool {
                terrain,
                width,
                height,
            },
            inside_seed,
        )
    }

    pub fn choose_inside_seed(plan: &Plan) -> (i64, i64) {
        match (&plan.handedness, &plan.instructions[0].dir) {
            (Handedness::Right, Up) => (1, 1),
            (Handedness::Right, Down) => (-1, -1),
            (Handedness::Right, Left) => (-1, -1),
            (Handedness::Right, Right) => (1, 1),
            (Handedness::Left, Up) => (-1, 1),
            (Handedness::Left, Down) => (1, -1),
            (Handedness::Left, Left) => (-1, 1),
            (Handedness::Left, Right) => (1, -1),
        }
    }

    // flood-fill
    pub fn dig_out_interior(pool: &mut LavaPool, seed: (i64, i64)) {
        let mut stack = vec![seed];

        while let Some(curr) = stack.pop() {
            if curr.0 < 0
                || curr.0 >= pool.width as i64
                || curr.1 < 0
                || curr.1 >= pool.height as i64
            {
                continue;
            }

            if pool.terrain[curr.1 as usize][curr.0 as usize] == Trench {
                continue;
            }

            pool.terrain[curr.1 as usize][curr.0 as usize] = Trench;

            stack.push((curr.0 - 1, curr.1));
            stack.push((curr.0 + 1, curr.1));
            stack.push((curr.0, curr.1 - 1));
            stack.push((curr.0, curr.1 + 1));
        }
    }

    pub fn decode_plan(plan: &mut Plan) -> Result<(), String> {
        for instruction in &mut plan.instructions {
            (instruction.dir, instruction.count) = Self::decode_color(&instruction.color)?;
        }

        plan.handedness = Plan::calculate_handedness(&plan.instructions);

        Ok(())
    }

    fn decode_color(color: &str) -> Result<(Direction, i64), String> {
        let (count_hex, dir_str) = color.split_at(color.len() - 1);

        let dir = match dir_str {
            "0" => Right,
            "1" => Down,
            "2" => Left,
            "3" => Up,
            _ => return Err("unknown direction character".to_owned()),
        };

        if let Ok(count) = i64::from_str_radix(count_hex, 16) {
            Ok((dir, count))
        } else {
            Err(format!("not a hex string: {count_hex}"))
        }
    }

    fn plan_points(plan: &Plan) -> Vec<(i64, i64)> {
        plan.instructions
            .iter()
            .fold(vec![(0, 0)], |mut acc, curr| {
                let pos = acc.last().unwrap();
                let moved = match curr.dir {
                    Up => (pos.0, pos.1 - curr.count),
                    Down => (pos.0, pos.1 + curr.count),
                    Left => (pos.0 - curr.count, pos.1),
                    Right => (pos.0 + curr.count, pos.1),
                };
                acc.push(moved);
                acc
            })
    }

    // https://en.wikipedia.org/wiki/Shoelace_formula
    pub fn calculate_volume(plan: &Plan) -> u64 {
        let points: Vec<_> = Self::plan_points(plan);

        let edges = points
            .windows(2)
            .map(|w| w[0].0.abs_diff(w[1].0) + w[0].1.abs_diff(w[1].1))
            .sum::<u64>();

        let area = points.windows(2).fold(0, |mut acc, w| {
            acc += w[0].0 * w[1].1 - w[1].0 * w[0].1;
            acc
        }) as u64 / 2;

        area + edges / 2 + 1
    }
}

impl LavaPool {
    pub fn volume(&self) -> usize {
        self.terrain
            .iter()
            .flatten()
            .filter(|t| **t == Trench)
            .count()
    }
}

impl Display for LavaPool {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.terrain
                .iter()
                .map(|row| row.iter().map(char::from).collect::<String>())
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

impl FromStr for LavaPool {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let terrain = s
            .split('\n')
            .map(|l| {
                l.chars()
                    .map(Terrain::try_from)
                    .collect::<Result<Vec<_>, _>>()
            })
            .collect::<Result<Vec<_>, _>>()?;

        let width = s.split('\n').next().unwrap().len();
        let height = s.split('\n').count();

        Ok(LavaPool {
            terrain,
            width,
            height,
        })
    }
}

impl From<&Terrain> for char {
    fn from(t: &Terrain) -> Self {
        match t {
            Trench => '#',
            Ground => '.',
        }
    }
}

impl TryFrom<char> for Terrain {
    type Error = ();

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            '#' => Ok(Trench),
            '.' => Ok(Ground),
            _ => Err(()),
        }
    }
}

impl Plan {
    pub fn from_stream(r: impl Read) -> Result<Self, ParseError> {
        let reader = BufReader::new(r);

        let instructions: Vec<_> = reader
            .lines()
            .map(|l| {
                let line = l.map_err(ParseError::IO)?;
                let instruction = line.parse::<Instruction>()?;
                Ok(instruction)
            })
            .collect::<Result<_, _>>()?;

        let handedness = Self::calculate_handedness(&instructions);

        Ok(Plan {
            instructions,
            handedness,
        })
    }

    fn calculate_handedness(instructions: &[Instruction]) -> Handedness {
        let sum: i64 = instructions
            .windows(2)
            .map(|w| match (&w[0].dir, &w[1].dir) {
                (Up, Right) => 1,
                (Right, Down) => 1,
                (Down, Left) => 1,
                (Left, Up) => 1,
                (Right, Up) => -1,
                (Down, Right) => -1,
                (Left, Down) => -1,
                (Up, Left) => -1,
                _ => 0,
            })
            .sum();

        if sum > 0 {
            Handedness::Right
        } else {
            Handedness::Left
        }
    }
}

lazy_static! {
    static ref INSTRUCTION_REGEX: Regex =
        Regex::new(r"^(U|D|L|R) (\d+) \(#([0-9a-f]{6})\)$").unwrap();
}

impl FromStr for Instruction {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let cap = INSTRUCTION_REGEX
            .captures(s)
            .ok_or(ParseError::InvalidInstruction(s.to_owned()))?;

        let dir = Direction::try_from(cap[1].chars().next().unwrap()).unwrap();
        let count = cap[2].parse::<i64>().unwrap();
        let color = cap[3].to_owned();

        Ok(Instruction { dir, count, color })
    }
}

impl TryFrom<char> for Direction {
    type Error = ();

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            'U' => Ok(Up),
            'D' => Ok(Down),
            'L' => Ok(Left),
            'R' => Ok(Right),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub enum ParseError {
    IO(io::Error),
    InvalidInstruction(String),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::IO(e) => write!(f, "Could not read file: {e}"),
            ParseError::InvalidInstruction(s) => write!(f, "Invalid instruction: {s}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use stringreader::StringReader;

    #[test]
    fn parse_plan() {
        let input = "R 6 (#70c710)
D 5 (#0dc571)
U 2 (#caa173)
L 1 (#1b58a2)";

        let reader = StringReader::new(input);
        let plan = Plan::from_stream(reader).unwrap();

        assert_eq!(
            Instruction {
                dir: Right,
                count: 6,
                color: "70c710".to_owned(),
            },
            plan.instructions[0]
        );

        assert_eq!(
            Instruction {
                dir: Down,
                count: 5,
                color: "0dc571".to_owned(),
            },
            plan.instructions[1]
        );

        assert_eq!(
            Instruction {
                dir: Up,
                count: 2,
                color: "caa173".to_owned(),
            },
            plan.instructions[2]
        );

        assert_eq!(
            Instruction {
                dir: Left,
                count: 1,
                color: "1b58a2".to_owned(),
            },
            plan.instructions[3]
        );
    }

    #[test]
    fn dig_sides() {
        let input = "R 6 (#70c710)
D 5 (#0dc571)
L 2 (#5713f0)
D 2 (#d2c081)
R 2 (#59c680)
D 2 (#411b91)
L 5 (#8ceee2)
U 2 (#caa173)
L 1 (#1b58a2)
U 2 (#caa171)
R 2 (#7807d2)
U 3 (#a77fa3)
L 2 (#015232)
U 2 (#7a21e3)";

        let expectation = "#######
#.....#
###...#
..#...#
..#...#
###.###
#...#..
##..###
.#....#
.######";

        let reader = StringReader::new(input);
        let plan = Plan::from_stream(reader).unwrap();

        let (pool, seed) = Elves::dig_sides(&plan);
        let output = pool.to_string();

        assert_eq!(output, expectation);
        assert_eq!((1, 1), seed);
    }

    #[test]
    fn dig_out_interior() {
        let input = "#######
#.....#
###...#
..#...#
..#...#
###.###
#...#..
##..###
.#....#
.######";

        let expectation = "#######
#######
#######
..#####
..#####
#######
#####..
#######
.######
.######";

        let mut pool = input.parse::<LavaPool>().unwrap();
        Elves::dig_out_interior(&mut pool, (1, 1));
        let output = pool.to_string();

        assert_eq!(output, expectation);
    }

    #[test]
    fn volume_sides() {
        let input = "#######
#.....#
###...#
..#...#
..#...#
###.###
#...#..
##..###
.#....#
.######";

        let pool = input.parse::<LavaPool>().unwrap();
        assert_eq!(pool.volume(), 38);
    }

    #[test]
    fn volume_filled() {
        let input = "#######
#######
#######
..#####
..#####
#######
#####..
#######
.######
.######";

        let pool = input.parse::<LavaPool>().unwrap();
        assert_eq!(pool.volume(), 62);
    }

    #[rstest]
    #[case("70c710", Right, 461937)]
    #[case("0dc571", Down, 56407)]
    #[case("5713f0", Right, 356671)]
    #[case("d2c081", Down, 863240)]
    #[case("59c680", Right, 367720)]
    #[case("411b91", Down, 266681)]
    #[case("8ceee2", Left, 577262)]
    #[case("caa173", Up, 829975)]
    #[case("1b58a2", Left, 112010)]
    #[case("caa171", Down, 829975)]
    #[case("7807d2", Left, 491645)]
    #[case("a77fa3", Up, 686074)]
    #[case("015232", Left, 5411)]
    #[case("7a21e3", Up, 500254)]
    fn decode_instruction(
        #[case] input: &str,
        #[case] expected_dir: Direction,
        #[case] expected_count: i64,
    ) {
        let (dir, count) = Elves::decode_color(input).unwrap();

        assert_eq!(dir, expected_dir);
        assert_eq!(count, expected_count);
    }

    #[test]
    fn calculate_volume() {
        let input = "R 6 (#70c710)
D 5 (#0dc571)
L 2 (#5713f0)
D 2 (#d2c081)
R 2 (#59c680)
D 2 (#411b91)
L 5 (#8ceee2)
U 2 (#caa173)
L 1 (#1b58a2)
U 2 (#caa171)
R 2 (#7807d2)
U 3 (#a77fa3)
L 2 (#015232)
U 2 (#7a21e3)";

        let reader = StringReader::new(input);
        let plan = Plan::from_stream(reader).unwrap();

        let volume = Elves::calculate_volume(&plan);
        assert_eq!(volume, 62);
    }

    #[test]
    fn calculate_volume_simple() {
        let input = "R 5 (#70c710)
D 5 (#0dc571)
L 5 (#5713f0)
U 5 (#d2c081)";

        let reader = StringReader::new(input);
        let plan = Plan::from_stream(reader).unwrap();

        let (mut pool, seed) = Elves::dig_sides(&plan);
        Elves::dig_out_interior(&mut pool, seed);

        let empirical = pool.volume() as u64;

        let calculated = Elves::calculate_volume(&plan);
        assert_eq!(calculated, empirical);
    }

    #[test]
    fn calculate_volume_decoded() {
        let input = "R 6 (#70c710)
D 5 (#0dc571)
L 2 (#5713f0)
D 2 (#d2c081)
R 2 (#59c680)
D 2 (#411b91)
L 5 (#8ceee2)
U 2 (#caa173)
L 1 (#1b58a2)
U 2 (#caa171)
R 2 (#7807d2)
U 3 (#a77fa3)
L 2 (#015232)
U 2 (#7a21e3)";

        let reader = StringReader::new(input);
        let mut plan = Plan::from_stream(reader).unwrap();

        Elves::decode_plan(&mut plan).unwrap();

        let volume = Elves::calculate_volume(&plan);
        assert_eq!(volume, 952408144115);
    }
}
