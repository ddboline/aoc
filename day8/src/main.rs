use anyhow::Error;
use clap::Parser;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
struct Input {
    input: PathBuf,
}

fn main() -> Result<(), Error> {
    let opts = Input::parse();

    let buf = fs::read_to_string(&opts.input)?;
    let grid = TreeGrid::from_str(&buf);
    let n_visible = grid.number_visible();
    println!("n_visible {n_visible}");
    let max_score = grid.maximum_visibility_score();
    println!("max score {max_score}");
    Ok(())
}

#[derive(Debug, Default)]
struct TreeGrid(Vec<Vec<u8>>);

impl TreeGrid {
    fn from_str(buf: &str) -> Self {
        let mut grid = Self::default();
        for line in buf.split('\n') {
            let mut row = Vec::new();
            for c in line.chars() {
                assert!(('0'..='9').contains(&c));
                let height: u8 = c as u8 - b'0';
                row.push(height);
            }
            if !row.is_empty() {
                grid.0.push(row);
            }
        }
        let grid_size = grid.0.len();
        for row in &grid.0 {
            assert_eq!(row.len(), grid_size);
        }
        grid
    }

    fn number_visible(&self) -> usize {
        let grid_size = self.0.len();
        let mut n_visible = 4 * grid_size - 4;
        for idx in 1..grid_size - 1 {
            for jdx in 1..grid_size - 1 {
                let height = self.0[idx][jdx];
                let mut is_visible = true;
                for x in (0..idx).rev() {
                    if self.0[x][jdx] >= height {
                        is_visible = false;
                        break;
                    }
                }
                if is_visible {
                    n_visible += 1;
                    continue;
                }
                let mut is_visible = true;
                for x in idx + 1..grid_size {
                    if self.0[x][jdx] >= height {
                        is_visible = false;
                        break;
                    }
                }
                if is_visible {
                    n_visible += 1;
                    continue;
                }

                let mut is_visible = true;
                for y in (0..jdx).rev() {
                    if self.0[idx][y] >= height {
                        is_visible = false;
                        break;
                    }
                }
                if is_visible {
                    n_visible += 1;
                    continue;
                }
                let mut is_visible = true;
                for y in jdx + 1..grid_size {
                    if self.0[idx][y] >= height {
                        is_visible = false;
                        break;
                    }
                }
                if is_visible {
                    n_visible += 1;
                    continue;
                }
            }
        }
        n_visible
    }

    fn visibility_score(&self, x: usize, y: usize) -> usize {
        let grid_size = self.0.len();
        let mut visibility_score = 1;
        let height = self.0[x][y];
        let mut n_trees = 0;
        for idx in (0..x).rev() {
            n_trees += 1;
            if self.0[idx][y] >= height {
                break;
            }
        }
        visibility_score *= n_trees;

        let mut n_trees = 0;
        for idx in x + 1..grid_size {
            n_trees += 1;
            if self.0[idx][y] >= height {
                break;
            }
        }
        visibility_score *= n_trees;

        let mut n_trees = 0;
        for idy in (0..y).rev() {
            n_trees += 1;
            if self.0[x][idy] >= height {
                break;
            }
        }
        visibility_score *= n_trees;

        let mut n_trees = 0;
        for idy in y + 1..grid_size {
            n_trees += 1;
            if self.0[x][idy] >= height {
                break;
            }
        }
        visibility_score *= n_trees;

        visibility_score
    }

    fn maximum_visibility_score(&self) -> usize {
        let mut max_score = 0;
        let grid_size = self.0.len();
        for idx in 0..grid_size {
            for jdx in 0..grid_size {
                let score = self.visibility_score(idx, jdx);
                if score > max_score {
                    max_score = score;
                }
            }
        }
        max_score
    }
}

pub static TEST_GRID: &str = "\
30373
25512
65332
33549
35390
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_from_str() -> Result<(), Error> {
        let grid = TreeGrid::from_str(TEST_GRID);
        println!("{grid:?}");

        assert_eq!(grid.number_visible(), 21);
        Ok(())
    }

    #[test]
    fn test_maximum_visibility_score() -> Result<(), Error> {
        let grid = TreeGrid::from_str(TEST_GRID);
        assert_eq!(grid.visibility_score(1, 2), 4);
        assert_eq!(grid.visibility_score(3, 2), 8);
        assert_eq!(grid.maximum_visibility_score(), 8);
        Ok(())
    }
}
