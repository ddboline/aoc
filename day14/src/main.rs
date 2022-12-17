use anyhow::Error;
use clap::Parser;
use std::fs;
use std::path::PathBuf;
use std::collections::HashSet;

#[derive(Parser)]
struct Input {
    input: PathBuf,
}

fn main() -> Result<(), Error> {
    let opts = Input::parse();

    let buf = fs::read_to_string(&opts.input)?;
    let mut cave = Cave::from_str(&buf).unwrap();
    let sand_count = cave.count_sand();
    println!("sand count {sand_count}");
    assert_eq!(sand_count, 1001);

    let mut cave = Cave::from_str(&buf).unwrap();
    let sand_count_with_floor = cave.count_sand_with_floor();
    println!("sand count with floor {sand_count_with_floor}");
    assert_eq!(sand_count_with_floor, 27976);
    Ok(())
}

#[derive(Debug, Default)]
struct Cave {
    occupied_tiles: HashSet<(i64, i64)>,
    current_sand: Option<(i64, i64)>,
    max_y: Option<i64>,
}

impl Cave {
    fn from_str(buf: &str) -> Option<Self> {
        let mut occupied_tiles = HashSet::new();
        for line in buf.split('\n') {
            if line.is_empty() {continue;}
            let mut verticies = Vec::new();
            for entry in line.split(" -> ") {
                let mut iter = entry.split(',');
                let x: i64 = iter.next()?.parse().ok()?;
                let y: i64 = iter.next()?.parse().ok()?;
                verticies.push((x, y));
            }
            let mut iter = verticies.iter();
            let (mut x0, mut y0) = iter.next().copied()?;
            while let Some((x1, y1)) = iter.next().copied() {
                if x0 != x1 && y0 != y1 {
                    return None;
                } else if x0 == x1 && y0 == y1 {
                    occupied_tiles.insert((x0, y0));
                } else if x0 == x1 {
                    if y0 < y1 {
                        for y in y0..=y1 {
                            occupied_tiles.insert((x0, y));
                        }
                    } else if y1 < y0 {
                        for y in y1..=y0 {
                            occupied_tiles.insert((x0, y));
                        }
                    }
                } else if y0 == y1 {
                    if x0 < x1 {
                        for x in x0..=x1 {
                            occupied_tiles.insert((x, y0));
                        }
                    } else if x1 < x0 {
                        for x in x1..=x0 {
                            occupied_tiles.insert((x, y0));
                        }
                    }
                }
                x0 = x1;
                y0 = y1;
            }
        }
        let max_y = occupied_tiles.iter().map(|(_, y)| y).max().copied();
        Some(Self{occupied_tiles, max_y, ..Self::default()})
    }

    fn pour_sand(&mut self) -> Option<(i64, i64)> {
        let (x, y) = self.current_sand.unwrap_or((500, 0));
        if self.occupied_tiles.contains(&(x, y+1)) {
            if self.occupied_tiles.contains(&(x-1, y+1)) {
                if self.occupied_tiles.contains(&(x+1, y+1)) {
                    if self.occupied_tiles.contains(&(500, 0)) {
                        self.current_sand.take()
                    } else {
                        self.occupied_tiles.insert((x, y));
                        self.current_sand.replace((500, 0));
                        None    
                    }
                } else {
                    self.current_sand.replace((x+1, y+1));
                    None
                }
            } else {
                self.current_sand.replace((x-1, y+1));
                None
            }
        } else {
            if y + 1 >= self.max_y? + 2 {
                self.current_sand.take()
            } else {
                self.current_sand.replace((x, y+1));
                None
            }
        }
    }

    fn count_sand(&mut self) -> usize {
        let initial_occupation = self.occupied_tiles.len();
        loop {
            if self.pour_sand().is_some() {
                break;
            }
        }
        self.occupied_tiles.len() - initial_occupation
    }

    fn count_sand_with_floor(&mut self) -> usize {
        let initial_occupation = self.occupied_tiles.len();
        loop {
            if let Some((x, y)) = self.pour_sand() {
                if self.occupied_tiles.contains(&(500, 0)) {
                    break;
                } else {
                    self.occupied_tiles.insert((x, y));
                    self.current_sand.replace((500, 0));
                }
                if self.occupied_tiles.len() > 93 {
                }
            }
        }
        self.occupied_tiles.len() - initial_occupation
    }
}

pub static TEST_DATA: &str = "
498,4 -> 498,6 -> 496,6
503,4 -> 502,4 -> 502,9 -> 494,9
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() -> Result<(), Error> {
        let mut occupied_tiles = Cave::from_str(TEST_DATA).unwrap();
        assert_eq!(occupied_tiles.occupied_tiles.len(), 20);
        println!("{occupied_tiles:?}");
        let sand_count = occupied_tiles.count_sand();
        assert_eq!(sand_count, 24);
        Ok(())
    }

    #[test]
    fn test_with_floor() -> Result<(), Error> {
        let mut cave = Cave::from_str(TEST_DATA).unwrap();
        assert_eq!(cave.count_sand_with_floor(), 93);
        Ok(())
    }
}