use anyhow::{Error, format_err};
use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};
use std::convert::TryFrom;
use std::cmp::Ordering;
use std::io::{BufRead, BufReader};
use smallvec::SmallVec;

#[derive(Parser)]
struct Input {
    input: PathBuf,
}

fn main() -> Result<(), Error> {
    let opts = Input::parse();

    let total_score = simple_iterator(&opts.input)?;
    println!("total score: {total_score}");
    let total_score = use_bufreader(&opts.input)?;
    println!("total score: {total_score}");
    let total_score = simple_iterator2(&opts.input)?;
    println!("total score: {total_score}");
    let total_score = use_bufreader2(&opts.input)?;
    println!("total score: {total_score}");
    Ok(())
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum RPS {
    Rock,
    Paper,
    Scissors,
}

impl TryFrom<char> for RPS {
    type Error = Error;
    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'A' | 'X' => Ok(Self::Rock),
            'B' | 'Y' => Ok(Self::Paper),
            'C' | 'Z' => Ok(Self::Scissors),
            _ => Err(format_err!("Bad char"))
        }
    }
}

impl RPS {
    fn to_u8(self) -> u8 {
        match self {
            Self::Rock => 1,
            Self::Paper => 2,
            Self::Scissors => 3,
        }
    }
}

impl PartialOrd for RPS {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self == other {
            Some(Ordering::Equal)
        } else if *self == Self::Rock && *other == Self::Scissors {
            Some(Ordering::Greater)
        } else if *self == Self::Scissors && *other == Self::Rock {
            Some(Ordering::Less)
        } else {
            self.to_u8().partial_cmp(&other.to_u8())
        }
    }
}

impl From<RPS> for u64 {
    fn from(x: RPS) -> Self {
        match x {
            RPS::Rock => 1,
            RPS::Paper => 2,
            RPS::Scissors => 3,
        }
    }
}

fn calculate_score(play0: RPS, play1: RPS) -> u64 {
    let score = play1.to_u8() as u64;
    if play1 > play0 {
        6 + score
    } else if play1 == play0 {
        3 + score
    } else {
        score
    }
}

fn get_score(s: &str) -> Option<u64> {
    let chars: SmallVec<[char; 3]> = s.chars().collect();
    let play0: RPS = (*chars.get(0)?).try_into().ok()?;
    let play1: RPS = (*chars.get(2)?).try_into().ok()?;
    let score = calculate_score(play0, play1);
    Some(score)
}

fn simple_iterator(p: &Path) -> Result<u64, Error> {
    let buf = fs::read_to_string(p)?;
    let total_score = buf.split('\n').filter_map(get_score).sum();
    Ok(total_score)
}

fn use_bufreader(p: &Path) -> Result<u64, Error> {
    let f = fs::File::open(p)?;
    let mut buf = BufReader::new(f);
    let mut line = String::new();
    let mut total_score = 0;
    while buf.read_line(&mut line)? > 0 {
        total_score += get_score(&line).unwrap_or(0);
        line.clear();
    }
    Ok(total_score)
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum WLD {
    Lose,
    Win,
    Draw,
}

impl TryFrom<char> for WLD {
    type Error = Error;
    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'X' => Ok(Self::Lose),
            'Y' => Ok(Self::Draw),
            'Z' => Ok(Self::Win),
            _ => Err(format_err!("Bad char")),
        }
    }
}

fn choose_play(play0: RPS, result: WLD) -> RPS {
    match result {
        WLD::Draw => play0,
        WLD::Lose => {
            match play0 {
                RPS::Rock => RPS::Scissors,
                RPS::Paper => RPS::Rock,
                RPS::Scissors => RPS::Paper,
            }
        },
        WLD::Win => {
            match play0 {
                RPS::Rock => RPS::Paper,
                RPS::Paper => RPS::Scissors,
                RPS::Scissors => RPS::Rock,
            }
        }
    }
}

fn get_score2(s: &str) -> Option<u64> {
    let chars: SmallVec<[char; 3]> = s.chars().collect();
    let play0: RPS = (*chars.get(0)?).try_into().ok()?;
    let cond: WLD = (*chars.get(2)?).try_into().ok()?;
    let play1 = choose_play(play0, cond);
    let score = calculate_score(play0, play1);
    Some(score)
}

fn simple_iterator2(p: &Path) -> Result<u64, Error> {
    let buf = fs::read_to_string(p)?;
    let total_score = buf.split('\n').filter_map(get_score2).sum();
    Ok(total_score)
}

fn use_bufreader2(p: &Path) -> Result<u64, Error> {
    let f = fs::File::open(p)?;
    let mut buf = BufReader::new(f);
    let mut line = String::new();
    let mut total_score = 0;
    while buf.read_line(&mut line)? > 0 {
        total_score += get_score2(&line).unwrap_or(0);
        line.clear();
    }
    Ok(total_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_score() {
        assert_eq!(calculate_score(RPS::Rock, RPS::Paper), 8);
        assert_eq!(calculate_score(RPS::Paper, RPS::Rock), 1);
        assert_eq!(calculate_score(RPS::Scissors, RPS::Scissors), 6);
        assert_eq!(calculate_score(RPS::Rock, RPS::Scissors), 3);
    }

    #[test]
    fn test_calculate_score2() {
        assert_eq!(calculate_score(RPS::Rock, choose_play(RPS::Rock, WLD::Draw)), 4);
        assert_eq!(calculate_score(RPS::Paper, choose_play(RPS::Paper, WLD::Lose)), 1);
        assert_eq!(calculate_score(RPS::Scissors, choose_play(RPS::Scissors, WLD::Win)), 7);
    }
}