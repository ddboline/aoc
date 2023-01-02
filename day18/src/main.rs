use anyhow::Error;
use clap::Parser;
use maplit::hashmap;
use smallvec::SmallVec;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
struct Input {
    input: PathBuf,
}

fn main() -> Result<(), Error> {
    let opts = Input::parse();

    let buf = fs::read_to_string(&opts.input)?;
    let grid = Grid::from_str(&buf);
    let surface_area = grid.surface_area();
    println!("surface_area {surface_area}");
    assert_eq!(surface_area, 3494);
    let surface_area = grid.exterior_surface_area();
    println!("surface_area {surface_area}");
    assert_eq!(surface_area, 2062);
    Ok(())
}

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
struct Position {
    x: isize,
    y: isize,
    z: isize,
}

struct Grid {
    grid: HashSet<Position>,
}

#[derive(Debug)]
enum VisitedPositions {
    Interior(HashSet<Position>),
    Exterior(HashSet<Position>),
}

impl Grid {
    fn from_str(buf: &str) -> Self {
        let grid = buf
            .split('\n')
            .filter_map(|l| {
                let xyz: SmallVec<[isize; 3]> = l
                    .split(',')
                    .filter_map(|s| s.parse().ok())
                    .take(3)
                    .collect();
                if xyz.len() != 3 {
                    None
                } else {
                    Some(Position {
                        x: xyz[0],
                        y: xyz[1],
                        z: xyz[2],
                    })
                }
            })
            .collect();
        Self { grid }
    }

    fn surface_area(&self) -> usize {
        let mut surface_area = 0;
        for position in &self.grid {
            let mut new_position = *position;
            new_position.x += 1;
            if !self.grid.contains(&new_position) {
                surface_area += 1;
            }
            let mut new_position = *position;
            new_position.x -= 1;
            if !self.grid.contains(&new_position) {
                surface_area += 1;
            }
            let mut new_position = *position;
            new_position.y += 1;
            if !self.grid.contains(&new_position) {
                surface_area += 1;
            }
            let mut new_position = *position;
            new_position.y -= 1;
            if !self.grid.contains(&new_position) {
                surface_area += 1;
            }
            let mut new_position = *position;
            new_position.z += 1;
            if !self.grid.contains(&new_position) {
                surface_area += 1;
            }
            let mut new_position = *position;
            new_position.z -= 1;
            if !self.grid.contains(&new_position) {
                surface_area += 1;
            }
        }
        surface_area
    }

    /// try to find shortest path between point and exterior_point, if it exists, return None
    /// if it does not exist, return all positions visited while finding path
    fn find_interior_points(&self, point: Position, exterior_point: Position) -> VisitedPositions {
        let mut tentative: HashMap<Position, usize> = hashmap! {point => 0};
        let mut current = point;
        let mut visited: HashSet<Position> = HashSet::new();
        while let Some(current_distance) = tentative.remove(&current) {
            let current_distance = current_distance + 1;
            let mut new_position = current;
            new_position.x += 1;
            if !self.grid.contains(&new_position) && !visited.contains(&new_position) {
                if let Some(distance) = tentative.get_mut(&new_position) {
                    if *distance > current_distance {
                        *distance = current_distance;
                    }
                } else {
                    tentative.insert(new_position, current_distance);
                }
            }
            let mut new_position = current;
            new_position.x -= 1;
            if !self.grid.contains(&new_position) && !visited.contains(&new_position) {
                if let Some(distance) = tentative.get_mut(&new_position) {
                    if *distance > current_distance {
                        *distance = current_distance;
                    }
                } else {
                    tentative.insert(new_position, current_distance);
                }
            }
            let mut new_position = current;
            new_position.y += 1;
            if !self.grid.contains(&new_position) && !visited.contains(&new_position) {
                if let Some(distance) = tentative.get_mut(&new_position) {
                    if *distance > current_distance {
                        *distance = current_distance;
                    }
                } else {
                    tentative.insert(new_position, current_distance);
                }
            }
            let mut new_position = current;
            new_position.y -= 1;
            if !self.grid.contains(&new_position) && !visited.contains(&new_position) {
                if let Some(distance) = tentative.get_mut(&new_position) {
                    if *distance > current_distance {
                        *distance = current_distance;
                    }
                } else {
                    tentative.insert(new_position, current_distance);
                }
            }
            let mut new_position = current;
            new_position.z += 1;
            if !self.grid.contains(&new_position) && !visited.contains(&new_position) {
                if let Some(distance) = tentative.get_mut(&new_position) {
                    if *distance > current_distance {
                        *distance = current_distance;
                    }
                } else {
                    tentative.insert(new_position, current_distance);
                }
            }
            let mut new_position = current;
            new_position.z -= 1;
            if !self.grid.contains(&new_position) && !visited.contains(&new_position) {
                if let Some(distance) = tentative.get_mut(&new_position) {
                    if *distance > current_distance {
                        *distance = current_distance;
                    }
                } else {
                    tentative.insert(new_position, current_distance);
                }
            }
            visited.insert(current);
            if current == exterior_point {
                break;
            }
            let mut minimum: Option<usize> = None;
            for (p, d) in &tentative {
                if let Some(m) = minimum {
                    if *d < m {
                        minimum.replace(*d);
                        current = *p;
                    }
                } else {
                    minimum.replace(*d);
                    current = *p;
                }
            }
        }
        if visited.contains(&exterior_point) {
            let mut exterior_points: HashSet<Position> =
                tentative.into_iter().map(|(p, _)| p).collect();
            exterior_points.extend(visited);
            VisitedPositions::Exterior(exterior_points)
        } else {
            let mut interior_points: HashSet<Position> =
                tentative.into_iter().map(|(p, _)| p).collect();
            interior_points.extend(visited);
            VisitedPositions::Interior(interior_points)
        }
    }

    fn exterior_surface_area(&self) -> usize {
        let mut exterior_surface_area = 0;
        let max_x = self.grid.iter().map(|p| p.x).max().unwrap();
        let max_y = self.grid.iter().map(|p| p.y).max().unwrap();
        let max_z = self.grid.iter().map(|p| p.z).max().unwrap();
        let exterior_point = Position {
            x: max_x + 1,
            y: max_y + 1,
            z: max_z + 1,
        };
        let mut interior_points: HashSet<Position> = HashSet::new();
        let mut exterior_points: HashSet<Position> = HashSet::new();

        for position in &self.grid {
            let mut new_position = *position;
            new_position.x += 1;
            if exterior_points.contains(&new_position) {
                exterior_surface_area += 1;
            } else if !interior_points.contains(&new_position) && !self.grid.contains(&new_position)
            {
                match self.find_interior_points(new_position, exterior_point) {
                    VisitedPositions::Interior(points) => {
                        interior_points.extend(points);
                    }
                    VisitedPositions::Exterior(points) => {
                        exterior_points.extend(points);
                        exterior_surface_area += 1;
                    }
                }
            }
            let mut new_position = *position;
            new_position.x -= 1;
            if exterior_points.contains(&new_position) {
                exterior_surface_area += 1;
            } else if !interior_points.contains(&new_position) && !self.grid.contains(&new_position)
            {
                match self.find_interior_points(new_position, exterior_point) {
                    VisitedPositions::Interior(points) => {
                        interior_points.extend(points);
                    }
                    VisitedPositions::Exterior(points) => {
                        exterior_points.extend(points);
                        exterior_surface_area += 1;
                    }
                }
            }
            let mut new_position = *position;
            new_position.y += 1;
            if exterior_points.contains(&new_position) {
                exterior_surface_area += 1;
            } else if !interior_points.contains(&new_position) && !self.grid.contains(&new_position)
            {
                match self.find_interior_points(new_position, exterior_point) {
                    VisitedPositions::Interior(points) => {
                        interior_points.extend(points);
                    }
                    VisitedPositions::Exterior(points) => {
                        exterior_points.extend(points);
                        exterior_surface_area += 1;
                    }
                }
            }
            let mut new_position = *position;
            new_position.y -= 1;
            if exterior_points.contains(&new_position) {
                exterior_surface_area += 1;
            } else if !interior_points.contains(&new_position) && !self.grid.contains(&new_position)
            {
                match self.find_interior_points(new_position, exterior_point) {
                    VisitedPositions::Interior(points) => {
                        interior_points.extend(points);
                    }
                    VisitedPositions::Exterior(points) => {
                        exterior_points.extend(points);
                        exterior_surface_area += 1;
                    }
                }
            }
            let mut new_position = *position;
            new_position.z += 1;
            if exterior_points.contains(&new_position) {
                exterior_surface_area += 1;
            } else if !interior_points.contains(&new_position) && !self.grid.contains(&new_position)
            {
                match self.find_interior_points(new_position, exterior_point) {
                    VisitedPositions::Interior(points) => {
                        interior_points.extend(points);
                    }
                    VisitedPositions::Exterior(points) => {
                        exterior_points.extend(points);
                        exterior_surface_area += 1;
                    }
                }
            }
            let mut new_position = *position;
            new_position.z -= 1;
            if exterior_points.contains(&new_position) {
                exterior_surface_area += 1;
            } else if !interior_points.contains(&new_position) && !self.grid.contains(&new_position)
            {
                match self.find_interior_points(new_position, exterior_point) {
                    VisitedPositions::Interior(points) => {
                        interior_points.extend(points);
                    }
                    VisitedPositions::Exterior(points) => {
                        exterior_points.extend(points);
                        exterior_surface_area += 1;
                    }
                }
            }
        }
        exterior_surface_area
    }
}

pub static TEST_DATA: &str = "
2,2,2
1,2,2
3,2,2
2,1,2
2,3,2
2,2,1
2,2,3
2,2,4
2,2,6
1,2,5
3,2,5
2,1,5
2,3,5";

#[cfg(test)]
mod tests {
    use super::*;
    use maplit::hashset;

    #[test]
    fn test() {
        let grid = Grid::from_str(TEST_DATA);
        assert_eq!(grid.grid.len(), 13);
        assert!(grid.grid.contains(&Position { x: 2, y: 2, z: 2 }));
        assert_eq!(grid.surface_area(), 64);
    }

    #[test]
    fn test_surface_area() {
        let grid = Grid {
            grid: hashset! {
                Position {x: 1, y: 1, z: 1},
                Position {x: 2, y: 1, z: 1},
            },
        };
        assert_eq!(grid.surface_area(), 10);
    }

    #[test]
    fn test_find_interior_points() {
        let grid = Grid::from_str(TEST_DATA);
        let max_x = grid.grid.iter().map(|p| p.x).max().unwrap();
        let max_y = grid.grid.iter().map(|p| p.y).max().unwrap();
        let max_z = grid.grid.iter().map(|p| p.z).max().unwrap();
        let exterior_point = Position {
            x: max_x + 1,
            y: max_y + 1,
            z: max_z + 1,
        };
        let point = Position { x: 2, y: 2, z: 5 };
        let interior_points = grid.find_interior_points(point, exterior_point);
        println!("interior_points {interior_points:?}");

        let point = Position { x: 1, y: 1, z: 1 };
        let interior_points = grid.find_interior_points(point, exterior_point);
        match interior_points {
            VisitedPositions::Interior(_) => assert!(false),
            VisitedPositions::Exterior(_) => assert!(true),
        }
    }

    #[test]
    fn test_exterior_surface_area() {
        let grid = Grid::from_str(TEST_DATA);
        let external_surface_area = grid.exterior_surface_area();
        println!("external_surface_area {external_surface_area}");
        assert_eq!(external_surface_area, 58);
    }
}
