use crate::path::Direction::{East, North, South, West};
use std::cmp::{Ordering, Reverse};
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::fmt::{Debug, Display, Formatter};
use std::io;
use std::io::{BufRead, BufReader, Read};

pub struct City {
    blocks: HashMap<Position, Block>,
    pub width: i32,
    pub height: i32,
}

pub struct Block {
    pub heat_loss: u32,
    pub pos: Position,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
struct State {
    pos: Position,
    dir: Direction,
    steps: u32,
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
enum Direction {
    North,
    East,
    South,
    West,
}

#[derive(PartialEq)]
pub enum Crucible {
    Small,
    Ultra,
}

impl Debug for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{})", self.x, self.y)
    }
}

impl Position {
    fn mv(&self, dir: Direction) -> Position {
        match dir {
            North => Position {
                x: self.x,
                y: self.y - 1,
            },
            East => Position {
                x: self.x + 1,
                y: self.y,
            },
            South => Position {
                x: self.x,
                y: self.y + 1,
            },
            West => Position {
                x: self.x - 1,
                y: self.y,
            },
        }
    }

    fn l1_distance(&self, p2: &Position) -> u32 {
        self.x.abs_diff(p2.x) + self.y.abs_diff(p2.y)
    }
}

impl State {
    fn mv(&self, dir: Direction) -> State {
        State {
            pos: self.pos.mv(dir),
            dir,
            steps: if self.dir == dir { self.steps + 1 } else { 1 },
        }
    }
}

impl<'a> City {
    pub fn from_stream(r: impl Read) -> Result<Self, ParseError> {
        let reader = BufReader::new(r);

        let binding: Vec<Vec<_>> = reader
            .lines()
            .enumerate()
            .map(|(y, l)| {
                let line = l.map_err(ParseError::IO)?;
                let blocks = line
                    .chars()
                    .enumerate()
                    .map(|(x, c)| {
                        let heat_loss = c.to_digit(10).ok_or(ParseError::UnknownCharacter(c))?;
                        let pos = Position {
                            x: x as i32,
                            y: y as i32,
                        };

                        Ok((pos, Block { pos, heat_loss }))
                    })
                    .collect::<Result<_, _>>()?;
                Ok(blocks)
            })
            .collect::<Result<_, _>>()?;

        let blocks: HashMap<Position, Block> = binding.into_iter().flatten().collect();

        if blocks.is_empty() {
            return Err(ParseError::EmptyCity);
        }

        let width = blocks.iter().map(|b| b.0.x).max().unwrap() + 1;
        let height = blocks.iter().map(|b| b.0.y).max().unwrap() + 1;

        if (0..width).any(|x| (0..height).any(|y| blocks.get(&Position { x, y }).is_none())) {
            return Err(ParseError::UnevenGrid);
        }

        Ok(City {
            blocks,
            width,
            height,
        })
    }

    // A* search algorithm (https://en.wikipedia.org/wiki/A*_search_algorithm)
    pub fn navigate(
        &'a self,
        start: Position,
        goal: Position,
        crucible: Crucible,
    ) -> Vec<&'a Block> {
        let valid_start_moves = self.valid_moves(
            &State {
                pos: start,
                steps: 0,
                dir: South, // unimportant
            },
            &crucible,
        );

        let mut open_set: BinaryHeap<_> = valid_start_moves
            .iter()
            .map(|s| {
                Reverse(Candidate {
                    state: *s,
                    distance: s.pos.l1_distance(&goal),
                })
            })
            .collect();

        let mut queued: HashSet<State> = valid_start_moves.iter().cloned().collect();

        let mut came_from = HashMap::new();

        let mut losses: HashMap<State, u32> = valid_start_moves
            .iter()
            .map(|s| (*s, self.blocks.get(&s.pos).unwrap().heat_loss))
            .collect();

        let mut best_state: Option<State> = None;
        let mut best_loss = u32::MAX;

        while let Some(Reverse(curr)) = open_set.pop() {
            queued.remove(&curr.state);

            let curr_loss = *losses.get(&curr.state).unwrap();

            if curr.state.pos == goal
                && curr_loss <= best_loss
                && (crucible == Crucible::Small || curr.state.steps >= 4)
            {
                best_state = Some(curr.state);
                best_loss = curr_loss;

                println!("Best loss {best_loss}, remaining set: {}", open_set.len())
            }

            for neighbor in self.valid_moves(&curr.state, &crucible) {
                let neighbor_loss = curr_loss + self.blocks.get(&neighbor.pos).unwrap().heat_loss;
                let prev_neighbor_loss = *losses.get(&neighbor).unwrap_or(&u32::MAX);

                if neighbor_loss < best_loss && neighbor_loss < prev_neighbor_loss {
                    came_from.insert(neighbor, curr.state);
                    losses.insert(neighbor, neighbor_loss);

                    if !queued.contains(&neighbor) {
                        open_set.push(Reverse(Candidate {
                            state: neighbor,
                            distance: neighbor.pos.l1_distance(&goal),
                        }));

                        queued.insert(neighbor);
                    }
                }
            }
        }

        self.reconstruct_path(&came_from, &best_state.unwrap())
    }

    fn reconstruct_path(
        &'a self,
        came_from: &HashMap<State, State>,
        last: &State,
    ) -> Vec<&'a Block> {
        let mut path = vec![self.blocks.get(&last.pos).unwrap()];

        let mut curr = last;

        while came_from.contains_key(curr) {
            curr = came_from.get(curr).unwrap();
            path.push(self.blocks.get(&curr.pos).unwrap());
        }

        path.into_iter().rev().collect()
    }

    fn valid_moves(&self, curr: &State, crucible: &Crucible) -> Vec<State> {
        let mut next: Vec<State> = vec![];

        let candidates = match curr.dir {
            North => [North, East, West],
            East => [North, East, South],
            South => [East, South, West],
            West => [North, South, West],
        };

        for dir in candidates {
            if self.can_move(curr, dir, crucible) {
                next.push(curr.mv(dir))
            }
        }

        next
    }

    fn can_move(&self, curr: &State, dir: Direction, crucible: &Crucible) -> bool {
        let in_bounds = match dir {
            North => curr.pos.y > 0,
            East => curr.pos.x < self.width - 1,
            South => curr.pos.y < self.height - 1,
            West => curr.pos.x > 0,
        };

        let steps_ok = match crucible {
            Crucible::Small => curr.dir != dir || curr.steps < 3,
            Crucible::Ultra => {
                curr.steps == 0
                    || (curr.dir != dir && curr.steps >= 4)
                    || (curr.dir == dir && curr.steps < 10)
            }
        };

        in_bounds && steps_ok
    }
}

struct Candidate {
    state: State,
    distance: u32,
}

impl Eq for Candidate {}

impl PartialEq<Self> for Candidate {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance
    }
}

impl PartialOrd<Self> for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Candidate {
    fn cmp(&self, other: &Self) -> Ordering {
        self.distance.cmp(&other.distance)
    }
}

#[derive(Debug)]
pub enum ParseError {
    IO(io::Error),
    UnknownCharacter(char),
    EmptyCity,
    UnevenGrid,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::IO(e) => write!(f, "Could not read file: {e}"),
            ParseError::UnknownCharacter(c) => write!(f, "Unknown character: {c}"),
            ParseError::UnevenGrid => write!(f, "Expected even grid"),
            ParseError::EmptyCity => write!(f, "Expected city blocks"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stringreader::StringReader;

    #[test]
    fn navigate_small() {
        let input = "2413432311323
3215453535623
3255245654254
3446585845452
4546657867536
1438598798454
4457876987766
3637877979653
4654967986887
4564679986453
1224686865563
2546548887735
4322674655533";

        let reader = StringReader::new(input);
        let city = City::from_stream(reader).unwrap();

        let path = city.navigate(
            Position { x: 0, y: 0 },
            Position {
                x: city.width - 1,
                y: city.height - 1,
            },
            Crucible::Small,
        );

        let total_loss: u32 = path.iter().map(|b| b.heat_loss).sum();

        assert_eq!(102, total_loss);
    }

    #[test]
    fn navigate_ultra1() {
        let input = "2413432311323
3215453535623
3255245654254
3446585845452
4546657867536
1438598798454
4457876987766
3637877979653
4654967986887
4564679986453
1224686865563
2546548887735
4322674655533";

        let reader = StringReader::new(input);
        let city = City::from_stream(reader).unwrap();

        let path = city.navigate(
            Position { x: 0, y: 0 },
            Position {
                x: city.width - 1,
                y: city.height - 1,
            },
            Crucible::Ultra,
        );

        let total_loss: u32 = path.iter().map(|b| b.heat_loss).sum();

        assert_eq!(94, total_loss);
    }

    #[test]
    fn navigate_ultra2() {
        let input = "111111111111
999999999991
999999999991
999999999991
999999999991";

        let reader = StringReader::new(input);
        let city = City::from_stream(reader).unwrap();

        let path = city.navigate(
            Position { x: 0, y: 0 },
            Position {
                x: city.width - 1,
                y: city.height - 1,
            },
            Crucible::Ultra,
        );

        let total_loss: u32 = path.iter().map(|b| b.heat_loss).sum();

        assert_eq!(71, total_loss);
    }
}
