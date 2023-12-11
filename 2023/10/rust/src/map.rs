use lazy_static::lazy_static;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::io;
use std::io::{BufRead, BufReader, Read};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
enum Direction {
    North,
    East,
    South,
    West,
}

lazy_static! {
    static ref PIPES: HashMap<char, Vec<Direction>> = {
        let mut m = HashMap::new();
        m.insert(
            'S',
            vec![
                Direction::North,
                Direction::East,
                Direction::South,
                Direction::West,
            ],
        );
        m.insert('.', vec![]);
        m.insert('|', vec![Direction::North, Direction::South]);
        m.insert('-', vec![Direction::East, Direction::West]);
        m.insert('L', vec![Direction::North, Direction::East]);
        m.insert('J', vec![Direction::North, Direction::West]);
        m.insert('7', vec![Direction::South, Direction::West]);
        m.insert('F', vec![Direction::South, Direction::East]);
        m
    };
}

#[derive(Debug, PartialEq)]
struct Node {
    pos: (i64, i64),
    kind: char,
}

impl Node {
    fn is_connecting(&self, other: &Node) -> bool {
        self.is_adjacent(other) && self.can_connect(other)
    }

    fn is_adjacent(&self, other: &Node) -> bool {
        (self.pos.0.abs_diff(other.pos.0) + self.pos.1.abs_diff(other.pos.1)) == 1
    }

    fn can_connect(&self, other: &Node) -> bool {
        let s = PIPES.get(&self.kind).unwrap();
        let o = PIPES.get(&other.kind).unwrap();

        (self.pos.0 > other.pos.0 && s.contains(&Direction::West) && o.contains(&Direction::East))
            || (self.pos.0 < other.pos.0
                && s.contains(&Direction::East)
                && o.contains(&Direction::West))
            || (self.pos.1 > other.pos.1
                && s.contains(&Direction::North)
                && o.contains(&Direction::South))
            || (self.pos.1 < other.pos.1
                && s.contains(&Direction::South)
                && o.contains(&Direction::North))
    }

    fn is_opposite(&self, other: &Node) -> bool {
        let s = PIPES.get(&self.kind).unwrap();
        let o = PIPES.get(&other.kind).unwrap();

        s.iter().all(|d| !o.contains(d))
    }
}

#[derive(Debug, PartialEq)]
pub struct Map {
    start: (i64, i64),
    nodes: HashMap<(i64, i64), Node>,
}

impl Map {
    pub fn from_stream(s: impl Read) -> Result<Map, ParseError> {
        let reader = BufReader::new(s);

        let lines: Vec<(usize, String)> = reader
            .lines()
            .enumerate()
            .map(|(i, l)| {
                let line = l.map_err(ParseError::IO)?;
                Ok((i, line))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let nodes = Self::parse_nodes(lines)?;

        match nodes.iter().find(|(_, n)| n.kind == 'S') {
            Some(((x, y), _)) => Ok(Map {
                start: (*x, *y),
                nodes,
            }),
            None => Err(ParseError::MissingStart),
        }
    }

    fn parse_nodes(lines: Vec<(usize, String)>) -> Result<HashMap<(i64, i64), Node>, ParseError> {
        let nodes: HashMap<(i64, i64), Node> = lines
            .into_iter()
            .flat_map(|(y, s)| {
                s.chars()
                    .enumerate()
                    .map(|(x, c)| {
                        if !PIPES.contains_key(&c) {
                            Err(ParseError::UnknownCharacter(c))
                        } else {
                            let pos = (x as i64, y as i64);
                            Ok((pos, Node { pos, kind: c }))
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Result<HashMap<_, _>, _>>()?;
        Ok(nodes)
    }

    fn connecting_nodes(&self, node: &Node) -> Vec<&Node> {
        let top = self.nodes.get(&(node.pos.0, node.pos.1 - 1));
        let left = self.nodes.get(&(node.pos.0 - 1, node.pos.1));
        let bottom = self.nodes.get(&(node.pos.0, node.pos.1 + 1));
        let right = self.nodes.get(&(node.pos.0 + 1, node.pos.1));

        vec![top, left, bottom, right]
            .into_iter()
            .flatten()
            .filter(|n| node.is_connecting(n))
            .collect()
    }

    // depth-first search to find the loop
    #[allow(unused)]
    pub fn find_loop_recursive(&self) -> Option<Vec<(i64, i64)>> {
        let start = self.nodes.get(&self.start).unwrap();

        self.connecting_nodes(start)
            .into_iter()
            .filter_map(|n| self.explore(vec![start.pos], n))
            .next()
    }

    #[allow(unused)]
    fn explore(&self, visited: Vec<(i64, i64)>, node: &Node) -> Option<Vec<(i64, i64)>> {
        if node.kind == 'S' {
            return Some(visited);
        }

        self.connecting_nodes(node)
            .into_iter()
            .filter(|n| visited.iter().last() != Some(&n.pos))
            .filter_map(|n| {
                self.explore(
                    visited.clone().into_iter().chain(vec![node.pos]).collect(),
                    n,
                )
            })
            .next()
    }

    pub fn find_loop_iteration(&self) -> Option<Vec<(i64, i64)>> {
        let start = self.nodes.get(&self.start).unwrap();

        // Stack for depth-first search (current node, path visited so far)
        let mut stack: Vec<(&Node, Vec<(i64, i64)>)> = vec![(start, vec![])];

        while let Some((node, path)) = stack.pop() {
            if !path.is_empty() && node.kind == 'S' {
                return Some(path);
            }

            for next_node in self.connecting_nodes(node) {
                if path.iter().last() != Some(&next_node.pos) {
                    let mut new_path = path.clone();
                    new_path.push(node.pos);
                    stack.push((next_node, new_path));
                }
            }
        }

        None
    }

    pub fn find_nests(&self, path: Vec<(i64, i64)>) -> Vec<(i64, i64)> {
        let path_pos: HashSet<_> = path.iter().collect();
        let path_nodes: Vec<_> = path
            .clone()
            .into_iter()
            .map(|p| self.nodes.get(&p).unwrap())
            .collect();

        self.nodes
            .iter()
            .filter(|((x, y), _)| !path_pos.contains(&(*x, *y)))
            .filter(|((x, y), _)| {
                // vertical ray casting from outside
                let mut v_crosses: Vec<_> = path_nodes
                    .iter()
                    .filter(|pn| pn.pos.0 == *x && pn.pos.1 < *y)
                    .filter(|pn| {
                        let pipes = PIPES.get(&pn.kind).unwrap();
                        pipes.contains(&Direction::West) || pipes.contains(&Direction::East)
                    })
                    .collect();

                v_crosses.sort_by(|a,b| a.pos.1.cmp(&b.pos.1));

                // eliminate double count for opposite connections (L7,FJ)
                let connected_v_count = v_crosses
                    .windows(2)
                    .filter(|w| w[0].can_connect(w[1]) && w[0].is_opposite(w[1]))
                    .count();

                let mut h_crosses: Vec<_> = path_nodes
                    .iter()
                    .filter(|pn| pn.pos.0 < *x && pn.pos.1 == *y)
                    .filter(|pn| {
                        let pipes = PIPES.get(&pn.kind).unwrap();
                        pipes.contains(&Direction::North) || pipes.contains(&Direction::South)
                    })
                    .collect();

                h_crosses.sort_by(|a,b| a.pos.0.cmp(&b.pos.0));

                let connected_h_count = h_crosses
                    .windows(2)
                    .filter(|w| w[0].can_connect(w[1]) && w[0].is_opposite(w[1]))
                    .count();

                ((h_crosses.len() - connected_h_count) % 2 == 1)
                    && ((v_crosses.len() - connected_v_count) % 2 == 1)
            })
            .map(|((x, y), _)| (*x, *y))
            .collect()
    }
}

#[derive(Debug)]
pub(crate) enum ParseError {
    IO(io::Error),
    UnknownCharacter(char),
    MissingStart,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::IO(e) => write!(f, "Could not read file: {}", e),
            ParseError::UnknownCharacter(c) => write!(f, "Unknown character: {}", c),
            ParseError::MissingStart => write!(f, "Missing start node"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stringreader::StringReader;

    #[test]
    fn parsing_happy() {
        let input = ".....
.S-7.
.|.|.
.L-J.
.....";

        let map = Map::from_stream(StringReader::new(input)).unwrap();

        assert_eq!((1, 1), map.start);
        assert_eq!(25, map.nodes.len());
        assert_eq!(17, map.nodes.iter().filter(|(_, n)| n.kind == '.').count());
        assert_eq!('-', map.nodes.get(&(2, 1)).unwrap().kind);
        assert_eq!('7', map.nodes.get(&(3, 1)).unwrap().kind);
        assert_eq!('|', map.nodes.get(&(1, 2)).unwrap().kind);
        assert_eq!('|', map.nodes.get(&(3, 2)).unwrap().kind);
        assert_eq!('L', map.nodes.get(&(1, 3)).unwrap().kind);
        assert_eq!('-', map.nodes.get(&(2, 3)).unwrap().kind);
        assert_eq!('J', map.nodes.get(&(3, 3)).unwrap().kind);
    }

    #[test]
    fn find_loop_recursive() {
        let input = "..F7.
.FJ|.
SJ.L7
|F--J
LJ...";

        let map = Map::from_stream(StringReader::new(input)).unwrap();

        let l = map.find_loop_recursive().unwrap();

        assert_eq!(16, l.len());
    }

    #[test]
    fn find_loop_iteration() {
        let input = "..F7.
.FJ|.
SJ.L7
|F--J
LJ...";

        let map = Map::from_stream(StringReader::new(input)).unwrap();

        let l = map.find_loop_iteration().unwrap();

        assert_eq!(16, l.len());
    }

    #[test]
    fn find_nests_1() {
        let input = "...........
.S-------7.
.|F-----7|.
.||.....||.
.||.....||.
.|L-7.F-J|.
.|..|.|..|.
.L--J.L--J.
...........";

        let map = Map::from_stream(StringReader::new(input)).unwrap();

        let l = map.find_loop_iteration().unwrap();
        let nests = map.find_nests(l);

        assert_eq!(4, nests.len());

        assert!(nests.contains(&(2, 6)));
        assert!(nests.contains(&(3, 6)));
        assert!(nests.contains(&(7, 6)));
        assert!(nests.contains(&(8, 6)));
    }

    #[test]
    fn find_nests_2() {
        let input = ".F----7F7F7F7F-7....
.|F--7||||||||FJ....
.||.FJ||||||||L7....
FJL7L7LJLJ||LJ.L-7..
L--J.L7...LJS7F-7L7.
....F-J..F7FJ|L7L7L7
....L7.F7||L7|.L7L7|
.....|FJLJ|FJ|F7|.LJ
....FJL-7.||.||||...
....L---J.LJ.LJLJ...";

        let map = Map::from_stream(StringReader::new(input)).unwrap();

        let l = map.find_loop_iteration().unwrap();
        let nests = map.find_nests(l);

        assert_eq!(8, nests.len());
    }

    #[test]
    fn find_nests_3() {
        let input = "FF7FSF7F7F7F7F7F---7
L|LJ||||||||||||F--J
FL-7LJLJ||||||LJL-77
F--JF--7||LJLJ7F7FJ-
L---JF-JLJ.||-FJLJJ7
|F|F-JF---7F7-L7L|7|
|FFJF7L7F-JF7|JL---7
7-L-JL7||F7|L7F-7F7|
L.L7LFJ|||||FJL7||LJ
L7JLJL-JLJLJL--JLJ.L";

        let map = Map::from_stream(StringReader::new(input)).unwrap();

        let l = map.find_loop_iteration().unwrap();
        let nests = map.find_nests(l);

        assert_eq!(10, nests.len());
    }

    #[test]
    fn parsing_without_start() {
        let input = ".....
.F-7.
.|.|.
.L-J.
.....";

        let map = Map::from_stream(StringReader::new(input));

        assert!(map.is_err());
    }

    #[test]
    fn parsing_with_unknown_char() {
        let input = ".....
.S-K.
.|.|.
.L-J.
.....";

        let map = Map::from_stream(StringReader::new(input));

        assert!(map.is_err());
    }
}
