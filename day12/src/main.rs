use anyhow::Error;
use clap::Parser;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
struct Input {
    input: PathBuf,
}

fn main() -> Result<(), Error> {
    let opts = Input::parse();

    let buf = fs::read_to_string(&opts.input)?;
    let mut map = Map::from_str(&buf).unwrap();
    let fewest_steps = map.fewest_number_of_steps().unwrap();
    println!("fewest steps {fewest_steps}");
    let mut map = Map::from_str(&buf).unwrap();
    let fewest_steps_any_a = map.fewest_steps_any_a().unwrap();
    println!("fewest_steps_any_a {fewest_steps_any_a}");
    Ok(())
}

#[derive(Default, Debug, Clone, Copy)]
struct Node {
    height_char: char,
    min_steps: Option<usize>,
}

impl Node {
    fn from_char(c: char) -> Option<Self> {
        match c {
            'S' | 'E' | 'a'..='z' => Some(Self {
                height_char: c,
                min_steps: None,
            }),
            _ => None,
        }
    }

    fn height(self) -> i64 {
        match self.height_char {
            'S' => 0,
            'E' => 25,
            c => c as i64 - 'a' as i64,
        }
    }

    fn is_valid_next_step(self, node: Node) -> bool {
        node.height() - self.height() <= 1
    }

    fn increment_node(&mut self, new_steps: usize) -> Option<usize> {
        if let Some(current_steps) = self.min_steps {
            if current_steps < new_steps {
                return None;
            }
        }
        self.min_steps.replace(new_steps)
    }
}

#[derive(Default, Debug)]
struct Map {
    map: Vec<Vec<Node>>,
    width: usize,
    height: usize,
    start: (usize, usize),
    tentative: HashSet<(usize, usize)>,
    visited: HashSet<(usize, usize)>,
}

impl Map {
    fn set_new_start(&mut self, start: (usize, usize)) -> Option<()> {
        if start.0 > self.height || start.1 > self.width {
            println!("start {start:?}");
            return None;
        }
        for line in self.map.iter_mut() {
            for node in line.iter_mut() {
                node.min_steps = None;
            }
        }
        self.start = start;
        self.tentative.clear();
        self.visited.clear();
        Some(())
    }

    fn from_str(buf: &str) -> Option<Self> {
        let map = buf.split('\n').fold(Vec::new(), |mut acc, line| {
            let map_line: Vec<_> = line.chars().filter_map(Node::from_char).collect();
            if !map_line.is_empty() {
                acc.push(map_line);
            }
            acc
        });
        let height = map.len();
        if height == 0 {
            return None;
        }
        let width = map[0].len();
        let mut start = None;
        for (idx, x) in map.iter().enumerate() {
            if x.len() != width {
                return None;
            }
            for (idy, y) in x.iter().enumerate() {
                if y.height_char == 'S' {
                    start.replace((idx, idy));
                }
            }
        }
        let start = start?;
        Some(Self {
            map,
            width,
            height,
            start,
            ..Self::default()
        })
    }

    fn get_a_positions(&self) -> HashSet<(usize, usize)> {
        let mut a_positions = HashSet::new();
        for (idx, x) in self.map.iter().enumerate() {
            for (idy, y) in x.iter().enumerate() {
                if y.height_char == 'a' {
                    a_positions.insert((idx, idy));
                }
            }
        }
        a_positions
    }

    fn get_next_node(&self) -> Option<(usize, usize)> {
        if self.tentative.is_empty() {
            Some(self.start)
        } else {
            let mut min_step = None;
            let mut min_position = None;
            for (x, y) in &self.tentative {
                if self.visited.contains(&(*x, *y)) {
                    continue;
                }
                if let Some(m) = self.map.get(*x)?.get(*y)?.min_steps {
                    if let Some(cm) = min_step {
                        if m < cm {
                            min_step.replace(m);
                            min_position.replace((*x, *y));
                        }
                    } else {
                        min_step.replace(m);
                        min_position.replace((*x, *y));
                    }
                }
            }
            min_position
        }
    }

    fn fewest_number_of_steps(&mut self) -> Option<usize> {
        while let Some((x, y)) = self.get_next_node() {
            let current_steps = self.map.get(x)?.get(y)?.min_steps.unwrap_or(0);
            self.map
                .get_mut(x)?
                .get_mut(y)?
                .min_steps
                .replace(current_steps);
            let current_steps = current_steps + 1;

            let current_node = *self.map.get(x)?.get(y)?;
            if let Some(up_node) = self.map.get_mut(x + 1).and_then(|v| v.get_mut(y)) {
                if current_node.is_valid_next_step(*up_node) {
                    up_node.increment_node(current_steps);
                    self.tentative.insert((x + 1, y));
                }
            }
            if let Some(right_node) = self.map.get_mut(x).and_then(|v| v.get_mut(y + 1)) {
                if current_node.is_valid_next_step(*right_node) {
                    right_node.increment_node(current_steps);
                    self.tentative.insert((x, y + 1));
                }
            }
            if x > 0 {
                if let Some(down_node) = self.map.get_mut(x - 1).and_then(|v| v.get_mut(y)) {
                    if current_node.is_valid_next_step(*down_node) {
                        down_node.increment_node(current_steps);
                        self.tentative.insert((x - 1, y));
                    }
                }
            }
            if y > 0 {
                if let Some(left_node) = self.map.get_mut(x).and_then(|v| v.get_mut(y - 1)) {
                    if current_node.is_valid_next_step(*left_node) {
                        left_node.increment_node(current_steps);
                        self.tentative.insert((x, y - 1));
                    }
                }
            }
            self.visited.insert((x, y));
            if self.tentative.contains(&(x, y)) {
                self.tentative.remove(&(x, y));
            }
            if current_node.height_char == 'E' {
                return current_node.min_steps;
            }
        }
        None
    }

    fn fewest_steps_any_a(&mut self) -> Option<usize> {
        let mut min_steps = None;
        let a_positions = self.get_a_positions();
        println!("a_positions {}", a_positions.len());

        for (n, (x, y)) in a_positions.into_iter().enumerate() {
            self.set_new_start((x, y))?;
            if let Some(s) = self.fewest_number_of_steps() {
                if n % 100 == 0 {
                    println!("{n} number of steps {s}");
                }
                if let Some(ms) = min_steps {
                    if s < ms {
                        println!("{n} number of steps {s}");
                        min_steps.replace(s);
                    }
                } else {
                    println!("{n} number of steps {s}");
                    min_steps.replace(s);
                }
            }
        }
        min_steps
    }
}

pub static TEST_DATA: &str = "
Sabqponm
abcryxxl
accszExk
acctuvwj
abdefghi
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map() -> Result<(), Error> {
        let mut map = Map::from_str(TEST_DATA).unwrap();
        assert_eq!(map.height, 5);
        assert_eq!(map.width, 8);
        assert_eq!(map.start, (0, 0));
        println!("{:?}", map.map);

        assert_eq!(map.fewest_number_of_steps(), Some(31));

        let mut map = Map::from_str(TEST_DATA).unwrap();
        assert_eq!(map.fewest_steps_any_a(), Some(29));
        Ok(())
    }
}
