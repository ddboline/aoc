use anyhow::Error;
use clap::Parser;
use smallvec::{smallvec, SmallVec};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
struct Input {
    input: PathBuf,
}

fn main() -> Result<(), Error> {
    let opts = Input::parse();

    let buf = fs::read_to_string(&opts.input)?;
    let mut cave = Cave::from_str(&buf);
    let height = cave.get_rock_height(2022, RockShape::Horizontal);
    println!("height {height}");
    assert_eq!(height, 3149);

    let mut cave = Cave::from_str(&buf);

    let total_rocks = 1_000_000_000_000;
    let n = (total_rocks - 1726) / 1695;
    let nrocks = total_rocks - (n * 1695 + 1726) + 1726;
    let height = cave.get_rock_height(nrocks, RockShape::Horizontal);
    let final_height = (height - 2690) + (n * 2634 + 2690);
    println!("height {final_height}");
    assert_eq!(final_height, 1553982300884);
    Ok(())
}

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
enum RockShape {
    #[default]
    Horizontal = 0,
    Cross = 1,
    Scythe = 2,
    Vertical = 3,
    Square = 4,
}

impl RockShape {
    fn next(self) -> Self {
        match self {
            Self::Horizontal => Self::Cross,
            Self::Cross => Self::Scythe,
            Self::Scythe => Self::Vertical,
            Self::Vertical => Self::Square,
            Self::Square => Self::Horizontal,
        }
    }

    fn width(self) -> usize {
        match self {
            Self::Horizontal => 3,
            Self::Cross => 2,
            Self::Scythe => 2,
            Self::Vertical => 0,
            Self::Square => 1,
        }
    }

    fn height(self) -> usize {
        match self {
            Self::Horizontal => 0,
            Self::Cross => 2,
            Self::Scythe => 2,
            Self::Vertical => 3,
            Self::Square => 1,
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct Position {
    x: usize,
    y: usize,
}

impl Default for Position {
    fn default() -> Self {
        Self { x: 2, y: 3 }
    }
}

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct Rock {
    shape: RockShape,
    position: Position,
}

impl Rock {
    fn right_edge(self) -> Option<usize> {
        let width = self.shape.width();
        if self.position.x + width > 6 {
            None
        } else {
            Some(self.position.x + width)
        }
    }

    fn bottom_edge(self) -> Option<usize> {
        let height = self.shape.height();
        if self.position.y < height {
            None
        } else {
            Some(self.position.y - height)
        }
    }

    fn filled_positions(self) -> SmallVec<[Position; 5]> {
        match self.shape {
            RockShape::Horizontal => (0..4)
                .map(|x| Position {
                    x: self.position.x + x,
                    y: self.position.y,
                })
                .collect(),
            RockShape::Cross => {
                let mut positions = smallvec![Position {
                    x: self.position.x + 1,
                    y: self.position.y
                }];
                for x in 0..3 {
                    positions.push(Position {
                        x: self.position.x + x,
                        y: self.position.y - 1,
                    });
                }
                positions.push(Position {
                    x: self.position.x + 1,
                    y: self.position.y - 2,
                });
                positions
            }
            RockShape::Scythe => {
                let mut positions = smallvec![
                    Position {
                        x: self.position.x + 2,
                        y: self.position.y
                    },
                    Position {
                        x: self.position.x + 2,
                        y: self.position.y - 1
                    }
                ];
                for x in 0..3 {
                    positions.push(Position {
                        x: self.position.x + x,
                        y: self.position.y - 2,
                    });
                }
                positions
            }
            RockShape::Vertical => (0..4)
                .map(|y| Position {
                    x: self.position.x,
                    y: self.position.y - y,
                })
                .collect(),
            RockShape::Square => {
                let mut positions = SmallVec::new();
                for x in 0..2 {
                    for y in 0..2 {
                        positions.push(Position {
                            x: self.position.x + x,
                            y: self.position.y - y,
                        });
                    }
                }
                positions
            }
        }
    }
}

#[derive(Default, Debug)]
struct Cave {
    rocks: Vec<[bool; 7]>,
    jet_pattern: Vec<JetDirection>,
}

impl Cave {
    fn from_str(buf: &str) -> Self {
        let jet_pattern = buf
            .bytes()
            .filter_map(|c| match c {
                b'>' => Some(JetDirection::Right),
                b'<' => Some(JetDirection::Left),
                _ => None,
            })
            .collect();
        Self {
            jet_pattern,
            ..Self::default()
        }
    }

    fn rock_is_valid(&self, rock: Rock) -> bool {
        for position in rock.filled_positions() {
            if let Some(row) = self.rocks.get(position.y) {
                if row[position.x] {
                    return false;
                }
            }
        }
        true
    }

    fn move_horizontal(&mut self, mut rock: Rock, direction: JetDirection) -> Option<Rock> {
        match direction {
            JetDirection::Left => {
                if rock.position.x == 0 || rock.position.x >= 7 {
                    None
                } else {
                    rock.position.x -= 1;
                    if self.rock_is_valid(rock) {
                        Some(rock)
                    } else {
                        None
                    }
                }
            }
            JetDirection::Right => {
                if rock.right_edge()? < 6 {
                    rock.position.x += 1;
                    if self.rock_is_valid(rock) {
                        Some(rock)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
    }

    fn move_down(&mut self, mut rock: Rock) -> Option<Rock> {
        if rock.bottom_edge()? == 0 {
            return None;
        }
        rock.position.y -= 1;
        rock.bottom_edge()?;
        if self.rock_is_valid(rock) {
            Some(rock)
        } else {
            None
        }
    }

    fn new_rock(&mut self, shape: RockShape) -> Rock {
        Rock {
            shape,
            position: Position {
                x: 2,
                y: self.rocks.len() + 3 + shape.height(),
            },
        }
    }

    fn rock_fall(&mut self, rock_shape: RockShape, mut jet_idx: usize) -> usize {
        let mut rock = self.new_rock(rock_shape);
        let jet_len = self.jet_pattern.len();
        loop {
            let direction = &self.jet_pattern[jet_idx % jet_len];
            if let Some(new_rock) = self.move_horizontal(rock, *direction) {
                rock = new_rock;
            }
            if let Some(new_rock) = self.move_down(rock) {
                rock = new_rock;
            } else {
                for position in rock.filled_positions().iter().rev() {
                    if let Some(row) = self.rocks.get_mut(position.y) {
                        if !row[position.x] {
                            row[position.x] = true;
                        } else {
                            unreachable!("Something has gone terribly wrong");
                        }
                    } else {
                        let mut new_row = [false; 7];
                        new_row[position.x] = true;
                        assert_eq!(self.rocks.len(), position.y);
                        self.rocks.push(new_row);
                    }
                }
                jet_idx += 1;
                return jet_idx % jet_len;
            }
            jet_idx += 1;
        }
    }

    fn get_rock_height(&mut self, n_rocks: usize, mut shape: RockShape) -> usize {
        let mut jet_idx = 0;
        let jet_len = self.jet_pattern.len();
        for i in 0..n_rocks {
            jet_idx = self.rock_fall(shape, jet_idx);
            if jet_idx % jet_len == 1 || jet_idx % jet_len == 0 {
                println!(
                    "i {i} jet_idx {jet_idx} shape {shape:?} {}",
                    self.rocks.len()
                );
            }
            shape = shape.next();
        }
        self.rocks.len()
    }
}

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum JetDirection {
    #[default]
    Left,
    Right,
}

pub static TEST_DATA: &str = ">>><<><>><<<>><>>><<<>>><<<><<<>><>><<>>";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let cave = Cave::from_str(TEST_DATA);
        assert_eq!(cave.jet_pattern[0], JetDirection::Right);
        assert_eq!(cave.jet_pattern[3], JetDirection::Left);
        assert_eq!(
            cave.jet_pattern[cave.jet_pattern.len() - 1],
            JetDirection::Right
        );
    }

    #[test]
    fn test_filled_positions() {
        let rock = Rock {
            shape: RockShape::Horizontal,
            position: Position::default(),
        };
        let filled_positions = rock.filled_positions();
        assert!(filled_positions.iter().all(|p| p.y == 3));
        assert!(filled_positions
            .iter()
            .enumerate()
            .all(|(i, p)| { p.x == i + 2 }));
        let rock = Rock {
            shape: RockShape::Vertical,
            position: Position::default(),
        };
        let filled_positions = rock.filled_positions();
        assert!(filled_positions.iter().all(|p| p.x == 2));
        assert!(filled_positions
            .iter()
            .enumerate()
            .all(|(i, p)| { p.y == 3 - i }));
        let rock = Rock {
            shape: RockShape::Cross,
            position: Position::default(),
        };
        let filled_positions = rock.filled_positions();
        assert_eq!(filled_positions[0], Position { x: 3, y: 3 });
        assert!(filled_positions[1..4]
            .iter()
            .enumerate()
            .all(|(i, p)| { p.y == 2 && p.x == i + 2 }));
        assert_eq!(filled_positions[4], Position { x: 3, y: 1 });
        let rock = Rock {
            shape: RockShape::Scythe,
            position: Position::default(),
        };
        let filled_positions = rock.filled_positions();
        assert_eq!(filled_positions[0], Position { x: 4, y: 3 });
        assert_eq!(filled_positions[1], Position { x: 4, y: 2 });
        assert!(filled_positions[2..]
            .iter()
            .enumerate()
            .all(|(i, p)| { p.y == 1 && p.x == i + 2 }));
        let rock = Rock {
            shape: RockShape::Square,
            position: Position::default(),
        };
        let filled_positions = rock.filled_positions();
        assert_eq!(filled_positions[0], Position { x: 2, y: 3 });
        assert_eq!(filled_positions[1], Position { x: 2, y: 2 });
        assert_eq!(filled_positions[2], Position { x: 3, y: 3 });
        assert_eq!(filled_positions[3], Position { x: 3, y: 2 });
    }

    #[test]
    fn test_rock_fall() {
        let mut cave = Cave::from_str(TEST_DATA);
        let jet_idx = cave.rock_fall(RockShape::Horizontal, 0);
        assert_eq!(jet_idx, 4);
        assert_eq!(cave.rocks.len(), 1);
        assert_eq!(cave.rocks[0], [false, false, true, true, true, true, false]);
        let jet_idx = cave.rock_fall(RockShape::Cross, jet_idx);
        assert_eq!(jet_idx, 8);
        assert_eq!(
            cave.rocks[3],
            [false, false, false, true, false, false, false]
        );
    }

    #[test]
    fn get_rock_height() {
        let mut cave = Cave::from_str(TEST_DATA);
        let height = cave.get_rock_height(2022, RockShape::Horizontal);
        assert_eq!(height, 3068);
    }

    #[test]
    fn get_rock_height1() {
        let mut cave = Cave::from_str(TEST_DATA);
        let total_rocks = 2022;
        let n = (total_rocks - 22) / 35;
        let nrocks = total_rocks - (n * 35 + 22) + 22;
        let height = cave.get_rock_height(nrocks, RockShape::Horizontal);
        println!("n {n} nrocks {nrocks} height {height}");
        let final_height = (height - 42) + (n * 53 + 42);
        println!("n {n} height {final_height}");
        assert_eq!(final_height, 3068);
    }

    #[test]
    fn test_repeat() {
        let mut cave = Cave::from_str(TEST_DATA);
        let total_rocks: usize = 1_000_000_000_000;
        let n = (total_rocks - 22) / 35;
        let nrocks = total_rocks - (n * 35 + 22) + 22;
        let height = cave.get_rock_height(nrocks, RockShape::Horizontal);
        let final_height = (height - 42) + (n * 53 + 42);
        println!("n {n} height {final_height}");
        assert_eq!(final_height, 1514285714288);
    }
}
