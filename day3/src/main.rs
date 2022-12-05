use anyhow::Error;
use clap::Parser;
use itertools::Itertools;
use smallvec::SmallVec;
use std::collections::HashSet;
use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

#[derive(Parser)]
struct Input {
    input: PathBuf,
}

fn main() -> Result<(), Error> {
    let opts = Input::parse();

    let total_priority = simple_iterator(&opts.input)?;
    println!("total priority {total_priority}");
    let total_priority = use_bufreader(&opts.input)?;
    println!("total priority {total_priority}");

    let total_priority = simple_iterator2(&opts.input)?;
    println!("total priority {total_priority}");
    let total_priority = use_bufreader2(&opts.input)?;
    println!("total priority {total_priority}");
    Ok(())
}

fn simple_iterator(p: &Path) -> Result<u64, Error> {
    let buf = fs::read_to_string(p)?;

    let total_priority = buf
        .split('\n')
        .map(|s| {
            let (left, right) = split_elements(s);
            let common = common_element(left, right);
            common.and_then(get_priority).unwrap_or(0)
        })
        .sum();
    Ok(total_priority)
}

fn use_bufreader(p: &Path) -> Result<u64, Error> {
    let f = fs::File::open(p)?;
    let mut it = BufReadIter::new(f);
    let total_score = it.try_fold(0, |total_score, result| {
        result.map(|priority| total_score + priority.and_then(get_priority).unwrap_or(0))
    })?;
    Ok(total_score)
}

struct BufReadIter<T: Read> {
    reader: BufReader<T>,
    line: String,
}

impl<T: Read> BufReadIter<T> {
    fn new(read: T) -> Self {
        Self {
            reader: BufReader::new(read),
            line: String::new(),
        }
    }
}

impl<T: Read> Iterator for BufReadIter<T> {
    type Item = Result<Option<char>, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.reader.read_line(&mut self.line) {
            Ok(0) => None,
            Ok(_) => {
                let (left, right) = split_elements(self.line.trim());
                let common = common_element(left, right);
                self.line.clear();
                Some(Ok(common))
            }
            Err(e) => Some(Err(e.into())),
        }
    }
}

fn get_priority(c: char) -> Option<u64> {
    match c {
        'a'..='z' => Some(c as u64 - 'a' as u64 + 1),
        'A'..='Z' => Some(c as u64 - 'A' as u64 + 27),
        _ => None,
    }
}

fn split_elements(s: &str) -> (&str, &str) {
    assert!(s.len() % 2 == 0, "Cannot handle this");
    let split = s.len() / 2;
    (&s[..split], &s[split..])
}

fn common_element(left: &str, right: &str) -> Option<char> {
    let l: HashSet<char> = left.chars().collect();

    let mut it = right.chars().filter(|c| l.contains(c));
    let common_element = it.next();
    while let Some(next_element) = it.next() {
        assert_eq!(Some(next_element), common_element);
    }
    common_element
}

fn common_element2(e0: &str, e1: &str, e2: &str) -> Option<char> {
    let e0: HashSet<char> = e0.chars().collect();
    let e1: HashSet<char> = e1.chars().collect();
    let e2: HashSet<char> = e2.chars().collect();
    _common_element2(&e0, &e1, &e2)
}

fn _common_element2(e0: &HashSet<char>, e1: &HashSet<char>, e2: &HashSet<char>) -> Option<char> {
    let e01: HashSet<char> = e0.intersection(e1).copied().collect();

    let mut it = e1.intersection(e2).filter(|c| e01.contains(c));
    let common_element = it.next();
    while let Some(next_element) = it.next() {
        assert_eq!(Some(next_element), common_element);
    }
    common_element.copied()
}

fn simple_iterator2(p: &Path) -> Result<u64, Error> {
    let buf = fs::read_to_string(p)?;

    let total_priority = buf
        .split('\n')
        .chunks(3)
        .into_iter()
        .map(|chunk| {
            let elfs: SmallVec<[&str; 3]> = chunk.collect();
            if elfs.len() == 3 {
                common_element2(elfs[0], elfs[1], elfs[2])
                    .and_then(get_priority)
                    .unwrap_or(0)
            } else {
                0
            }
        })
        .sum();
    Ok(total_priority)
}

fn use_bufreader2(p: &Path) -> Result<u64, Error> {
    let f = fs::File::open(p)?;
    let it = BufReadIter2::new(f);
    let total_priority = it
        .chunks(3)
        .into_iter()
        .try_fold(0, |total_priority, chunk| {
            let result: Result<SmallVec<[HashSet<char>; 3]>, Error> = chunk.collect();
            result.map(|elfs| {
                total_priority
                    + _common_element2(&elfs[0], &elfs[1], &elfs[2])
                        .and_then(get_priority)
                        .unwrap_or(0)
            })
        })?;
    Ok(total_priority)
}

struct BufReadIter2<T: Read> {
    reader: BufReader<T>,
    line: String,
}

impl<T: Read> BufReadIter2<T> {
    fn new(read: T) -> Self {
        Self {
            reader: BufReader::new(read),
            line: String::new(),
        }
    }
}

impl<T: Read> Iterator for BufReadIter2<T> {
    type Item = Result<HashSet<char>, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.reader.read_line(&mut self.line) {
            Ok(0) => None,
            Ok(_) => {
                let h: HashSet<char> = self.line.trim().chars().collect();
                self.line.clear();
                Some(Ok(h))
            }
            Err(e) => Some(Err(e.into())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_priority() {
        assert_eq!(get_priority('a'), Some(1));
        assert_eq!(get_priority('A'), Some(27));
        for (p, c) in [(16, 'p'), (38, 'L'), (42, 'P'), (22, 'v'), (20, 't')] {
            assert_eq!(get_priority(c), Some(p));
        }
    }

    #[test]
    fn test_common_element() {
        let mut priorities = 0;
        for (s, c) in [
            ("vJrwpWtwJgWrhcsFMMfFFhFp", 'p'),
            ("jqHRNqRjqzjGDLGLrsFMfFZSrLrFZsSL", 'L'),
            ("PmmdzqPrVvPwwTWBwg", 'P'),
            ("wMqvLMZHhHMvwLHjbvcjnnSBnvTQFn", 'v'),
            ("ttgJtRGJQctTZtZT", 't'),
            ("CrZsJsPPZsGzwwsLwLmpwMDw", 's'),
        ] {
            let (left, right) = split_elements(s);
            let common = common_element(left, right);
            assert_eq!(common.unwrap(), c);
            priorities += get_priority(c).unwrap_or(0);
        }
        assert_eq!(priorities, 157);
    }

    #[test]
    fn test_common_element2() {
        assert_eq!(
            common_element2(
                "vJrwpWtwJgWrhcsFMMfFFhFp",
                "jqHRNqRjqzjGDLGLrsFMfFZSrLrFZsSL",
                "PmmdzqPrVvPwwTWBwg"
            ),
            Some('r')
        );
        assert_eq!(
            common_element2(
                "wMqvLMZHhHMvwLHjbvcjnnSBnvTQFn",
                "ttgJtRGJQctTZtZT",
                "CrZsJsPPZsGzwwsLwLmpwMDw"
            ),
            Some('Z')
        );
        let p0 = common_element2(
            "vJrwpWtwJgWrhcsFMMfFFhFp",
            "jqHRNqRjqzjGDLGLrsFMfFZSrLrFZsSL",
            "PmmdzqPrVvPwwTWBwg",
        )
        .and_then(get_priority)
        .unwrap_or(0);
        let p1 = common_element2(
            "wMqvLMZHhHMvwLHjbvcjnnSBnvTQFn",
            "ttgJtRGJQctTZtZT",
            "CrZsJsPPZsGzwwsLwLmpwMDw",
        )
        .and_then(get_priority)
        .unwrap_or(0);
        assert_eq!(p0 + p1, 70);
    }
}
