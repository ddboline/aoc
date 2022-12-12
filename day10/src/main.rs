use anyhow::{format_err, Error};
use clap::Parser;
use core::fmt;
use smallvec::{SmallVec, smallvec};
use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Parser)]
struct Input {
    input: PathBuf,
}

fn main() -> Result<(), Error> {
    let opts = Input::parse();

    let buf = fs::read_to_string(&opts.input)?;
    let program = Program::from_str(&buf)?;
    let signal_strength = program.get_signal_strength();
    println!("signal_strength {signal_strength}");

    let result = program.draw();
    let output: Vec<String> = result.into_iter().map(|v| v.into_iter().collect::<String>()).collect();
    let output = output.join("\n");
    println!("{output}");
    Ok(())
}

#[derive(Clone, Copy)]
enum Instruction {
    Noop,
    Addx(isize),
}

struct Program(Vec<Instruction>);

impl Program {
    fn from_str(buf: &str) -> Result<Self, Error> {
        let instructions: Result<Vec<_>, Error> = buf.split('\n').filter_map(|s| {
            if s.is_empty() {
                None
            } else {
                Some(s.parse())
            }
        }).collect();
        instructions.map(Self)
    }

    fn get_signal_strength(&self) -> isize {
        let mut total_signal_strength = 0;
        let mut state = MachineState::new();
        for instruction in &self.0 {
            for substate in state.process(*instruction) {
                if substate.tick >= 19 && (substate.tick - 19) % 40 == 0 {
                    let signal_strenth = (substate.tick as isize + 1) * substate.register_x;
                    total_signal_strength += signal_strenth;
                    println!("state {substate:?} strength {signal_strenth}");
                }    
            }
        }
        println!("state {state:?}");
        total_signal_strength
    }

    fn draw(&self) -> Vec<Vec<char>> {
        let mut screen = vec![vec!['.'; 40]; 6];
        let mut state = MachineState::new();
        let mut sprite_center = 1;
        for instruction in &self.0 {
             for substate in state.process(*instruction) {
                let y_value = ((substate.tick - 1) / 40) % 6;
                let x_value = (substate.tick - 1) % 40;
                if x_value as isize >= sprite_center - 1 && x_value as isize <= sprite_center + 1 {
                    screen[y_value][x_value] = '#';
                }
                sprite_center = substate.register_x;
             }
        }
        screen
    }
}

#[derive(Debug, Clone, Copy)]
struct MachineState {
    tick: usize,
    register_x: isize,
}

impl MachineState {
    fn new() -> Self {
        Self {
            tick: 0,
            register_x: 1,
        }
    }

    fn process(&mut self, instruction: Instruction) -> SmallVec<[MachineState; 2]> {
        self.tick += 1;
        let mut result = smallvec![*self];
        if let Instruction::Addx(v) = instruction {
            self.tick += 1;
            self.register_x += v;
            result.push(*self);
        }
        result
    }
}

impl FromStr for Instruction {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let line: SmallVec<[&str; 2]> = s.split(' ').take(2).collect();
        let value: Option<isize> = line.get(1).and_then(|s| s.parse().ok());
        match line.get(0) {
            Some(&"noop") => Ok(Self::Noop),
            Some(&"addx") => {
                if let Some(v) = value {
                    Ok(Self::Addx(v))
                } else {
                    Err(format_err!("No value"))
                }
            }
            _ => Err(format_err!("Bad line")),
        }
    }
}

pub static TEST0: &str = "
noop
addx 3
addx -5
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_machine() -> Result<(), Error> {
        let mut machine = MachineState::new();
        let expected_state = [(1, 1), (2, 1), (3, 4), (4, 4), (5, -1)];
        for line in TEST0.split('\n') {
            if line.is_empty() {
                continue;
            }
            let instruction: Instruction = line.parse()?;
            machine.process(instruction);
            println!("state {machine:?}");
            for (etick, ereg) in expected_state {
                if machine.tick == etick {
                    assert_eq!(machine.register_x, ereg);
                }
            }
        }
        Ok(())
    }

    #[test]
    fn test_get_signal_strength() -> Result<(), Error> {
        let buf = include_str!("../test.txt");
        let program = Program::from_str(&buf)?;
        let signal_strength = program.get_signal_strength();
        assert_eq!(signal_strength, 13140);
        Ok(())
    }

    #[test]
    fn test_draw() -> Result<(), Error> {
        let expected = vec![
            "##..##..##..##..##..##..##..##..##..##..",
            "###...###...###...###...###...###...###.",
            "####....####....####....####....####....",
            "#####.....#####.....#####.....#####.....",
            "######......######......######......####",
            "#######.......#######.......#######.....",
        ];

        let buf = include_str!("../test.txt");
        let program = Program::from_str(&buf)?;
        let result = program.draw();
        let mut observed = Vec::new();
        for line in result {
            let line: String = line.into_iter().collect();
            observed.push(line);
        }
        assert_eq!(expected.join("\n"), observed.join("\n"));
        Ok(())
    }
}
