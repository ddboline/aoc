use anyhow::{format_err, Error};
use clap::Parser;
use itertools::Itertools;
use smallvec::SmallVec;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Parser)]
struct Input {
    input: PathBuf,
}

fn main() -> Result<(), Error> {
    let opts = Input::parse();

    let buf = fs::read_to_string(opts.input)?;
    let plane = process_buf(&buf)?;
    println!("plane {}", plane.get_stack_top());
    let plane = process_buf_9001(&buf)?;
    println!("plane {}", plane.get_stack_top());
    Ok(())
}

fn process_buf(buf: &str) -> Result<Plane, Error> {
    let mut plane_buf = Vec::new();
    let mut instructions = Vec::new();
    for line in buf.split('\n') {
        if line.starts_with("move") {
            let instruction: MoveInstruction = line.parse()?;
            instructions.push(instruction);
        } else if !line.is_empty() {
            plane_buf.push(line);
        }
    }
    let mut plane = Plane::from_str(&plane_buf)?;
    for inst in instructions {
        plane.process_move(inst)?;
    }
    Ok(plane)
}

fn process_buf_9001(buf: &str) -> Result<Plane, Error> {
    let mut plane_buf = Vec::new();
    let mut instructions = Vec::new();
    for line in buf.split('\n') {
        if line.starts_with("move") {
            let instruction: MoveInstruction = line.parse()?;
            instructions.push(instruction);
        } else if !line.is_empty() {
            plane_buf.push(line);
        }
    }
    let mut plane = Plane::from_str(&plane_buf)?;
    for inst in instructions {
        plane.process_move_9001(inst)?;
    }
    Ok(plane)
}

#[derive(Debug)]
struct Plane(Vec<Vec<char>>);

impl Plane {
    fn get_stack_top(&self) -> String {
        self.0.iter().filter_map(|s| s.iter().last()).collect()
    }

    fn from_str(s: &[&str]) -> Result<Self, Error> {
        let rows = s.len();
        let nstacks = (s[rows - 1].len() + 1) / 4;
        let mut stacks = vec![Vec::new(); nstacks];
        for row in s.iter().rev().skip(1) {
            for (idx, mut chunk) in row.chars().chunks(4).into_iter().enumerate() {
                if let Some(x) = chunk.nth(1) {
                    if x == ' ' {
                        continue;
                    }
                    let s = stacks
                        .get_mut(idx)
                        .ok_or_else(|| format_err!("Bad index"))?;
                    s.push(x);
                }
            }
        }
        Ok(Self(stacks))
    }

    fn process_move(&mut self, inst: MoveInstruction) -> Result<(), Error> {
        for _ in 0..inst.ncrates {
            let cr = self
                .0
                .get_mut(inst.from_stack)
                .and_then(|s| s.pop())
                .ok_or_else(|| format_err!("No crates to move"))?;
            let s = self
                .0
                .get_mut(inst.to_stack)
                .ok_or_else(|| format_err!("Invalid Stack"))?;
            s.push(cr);
        }
        Ok(())
    }

    fn process_move_9001(&mut self, inst: MoveInstruction) -> Result<(), Error> {
        let mut crates_to_move = Vec::new();
        for _ in 0..inst.ncrates {
            let cr = self
                .0
                .get_mut(inst.from_stack)
                .and_then(|s| s.pop())
                .ok_or_else(|| format_err!("No crates to move"))?;
            crates_to_move.push(cr);
        }
        for cr in crates_to_move.into_iter().rev() {
            let s = self
                .0
                .get_mut(inst.to_stack)
                .ok_or_else(|| format_err!("Invalid Stack"))?;
            s.push(cr);
        }
        Ok(())
    }
}

#[derive(Debug)]
struct MoveInstruction {
    ncrates: usize,
    from_stack: usize,
    to_stack: usize,
}

impl FromStr for MoveInstruction {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let line: SmallVec<[&str; 6]> = s.split(' ').collect();
        if line[0] != "move" {
            return Err(format_err!("No Move"));
        }
        let ncrates = line[1].parse()?;
        let from_stack: usize = line[3].parse()?;
        let to_stack: usize = line[5].parse()?;
        if from_stack < 1 || to_stack < 1 {
            return Err(format_err!("Bad index"));
        }
        Ok(MoveInstruction {
            ncrates,
            from_stack: from_stack - 1,
            to_stack: to_stack - 1,
        })
    }
}

pub static TEST_BUF: &str = r#"    [D]    
[N] [C]    
[Z] [M] [P]
 1   2   3 

move 1 from 2 to 1
move 3 from 1 to 3
move 2 from 2 to 1
move 1 from 1 to 2
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_buf() {

        let plane = process_buf(TEST_BUF).unwrap();

        assert_eq!(
            plane.0,
            vec![vec!['C'], vec!['M'], vec!['P', 'D', 'N', 'Z']]
        );
        assert_eq!(&plane.get_stack_top(), "CMZ");
    }

    #[test]
    fn test_process_buf_9001() {
        let plane = process_buf_9001(TEST_BUF).unwrap();

        assert_eq!(
            plane.0,
            vec![vec!['M'], vec!['C'], vec!['P', 'Z', 'N', 'D']]
        );
        assert_eq!(&plane.get_stack_top(), "MCD");
    }
}
