use anyhow::{format_err, Error};
use clap::Parser;
use smallvec::SmallVec;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::cmp::Ordering;
use std::cmp::PartialEq;

#[derive(Parser)]
struct Input {
    input: PathBuf,
}

fn main() -> Result<(), Error> {
    let opts = Input::parse();

    let buf = fs::read_to_string(&opts.input)?;
    let directions = Directions::from_str(&buf)?;
    let number_tail_positions = directions.number_tail_visits();
    println!("number_tail_positions {number_tail_positions}");
    assert_eq!(number_tail_positions, 6212);

    let directions = Directions::from_str(&buf)?;
    let number_long_rope_tail_visits = directions.number_long_rope_tail_visits().unwrap();
    println!("number_long_rope_tail_visits {number_long_rope_tail_visits}");
    assert_eq!(number_long_rope_tail_visits, 2522);
    Ok(())
}

#[derive(Debug, Default, PartialEq, Eq, Hash, Clone, Copy)]
struct Position {
    x: isize,
    y: isize,
}

impl Position {
    fn move_head(direction: Direction, mut head: Self) -> Self {
        match direction {
            Direction::Up => {
                head.y += 1;
            }
            Direction::Down => {
                head.y -= 1;
            }
            Direction::Left => {
                head.x -= 1;
            }
            Direction::Right => {
                head.x += 1;
            }
        }
        head
    }

    fn move_tail(head: Self, mut tail: Self) -> (Self, Self) {
        if (head.x - tail.x).abs() > 1 || (head.y - tail.y).abs() > 1 {
            match head.x.cmp(&tail.x) {
                Ordering::Greater => {
                    tail.x += 1;
                }
                Ordering::Less => {
                    tail.x -= 1;
                }
                _ => {},
            }
            match head.y.cmp(&tail.y) {
                Ordering::Greater => {
                    tail.y += 1;
                }
                Ordering::Less => {
                    tail.y -= 1;
                }
                _ => {},
            }
        }
        (head, tail)
    }
}

#[derive(Debug, Default)]
struct Rope {
    head: Position,
    tail: Position,
    tail_history: HashSet<Position>,
}

impl Rope {
    fn move_rope(&mut self, direction: Direction) {
        (self.head, self.tail) =
            Position::move_tail(Position::move_head(direction, self.head), self.tail);
        self.tail_history.insert(self.tail);
    }
}

#[derive(Debug, Default)]
struct LongRope {
    legs: Vec<Position>,
    tail_history: HashSet<Position>,
}

impl LongRope {
    fn new() -> Self {
        Self {legs: vec![Position::default(); 10], ..Self::default()}
    }

    fn move_rope(&mut self, direction: Direction) -> Option<()> {
        let rope_length = self.legs.len();
        let mut head = Position::move_head(direction, *self.legs.get(0)?);
        *self.legs.get_mut(0)? = head;
        for i in 1..(rope_length) {
            let tail = *self.legs.get(i)?;
            let (_, tail) = Position::move_tail(head, tail);
            *self.legs.get_mut(i)? = tail;
            head = tail;
        }
        self.tail_history.insert(*self.legs.get(rope_length - 1)?);
        Some(())
    }
}

#[derive(Debug, Default, PartialEq)]
struct Directions(Vec<(Direction, usize)>);

impl Directions {
    fn from_str(buf: &str) -> Result<Self, Error> {
        let results: Result<Vec<_>, Error> = buf
            .split('\n')
            .map(|line| {
                let entries: SmallVec<[&str; 2]> = line.split(' ').take(2).collect();
                if entries.len() != 2 {
                    Ok(None)
                } else {
                    let direction = Direction::from_str(entries[0])?;
                    let increment: usize = entries[1].parse()?;
                    Ok(Some((direction, increment)))
                }
            })
            .filter_map(Result::transpose)
            .collect();
        results.map(Self)
    }

    fn number_tail_visits(&self) -> usize {
        let mut rope = Rope::default();
        for (direction, increment) in &self.0 {
            for _ in 0..*increment {
                rope.move_rope(*direction);
            }
        }
        rope.tail_history.len()
    }

    fn number_long_rope_tail_visits(&self) -> Option<usize> {
        let mut rope = LongRope::new();
        for (direction, increment) in &self.0 {
            for _ in 0..*increment {
                rope.move_rope(*direction)?;
            }
        }
        Some(rope.tail_history.len())
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Default for Direction {
    fn default() -> Self {
        Self::Up
    }
}

impl Direction {
    fn from_str(s: &str) -> Result<Self, Error> {
        match s {
            "U" => Ok(Self::Up),
            "D" => Ok(Self::Down),
            "L" => Ok(Self::Left),
            "R" => Ok(Self::Right),
            _ => Err(format_err!("Bad string")),
        }
    }
}

pub static TEST_INPUT: &str = "
R 4
U 4
L 3
D 1
R 4
D 1
L 5
R 2
";

pub static TEST_INPUT2: &str = "
R 5
U 8
L 8
D 3
R 17
D 10
L 25
U 20
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() -> Result<(), Error> {
        let directions = Directions::from_str(TEST_INPUT)?;
        assert_eq!(directions.0[3], (Direction::Down, 1));
        assert_eq!(directions.number_tail_visits(), 13);

        let directions = Directions::from_str(TEST_INPUT)?;
        assert_eq!(directions.number_long_rope_tail_visits().unwrap(), 1);

        let directions = Directions::from_str(TEST_INPUT2)?;
        assert_eq!(directions.number_long_rope_tail_visits().unwrap(), 36);
        Ok(())
    }
}
