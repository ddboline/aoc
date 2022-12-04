use anyhow::Error;
use clap::Parser;
use itertools::Itertools;
use smallvec::SmallVec;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
struct Input {
    input: PathBuf,
}

fn main() -> Result<(), Error> {
    let opts = Input::parse();

    let total_priority = simple_iterator(&opts.input)?;
    println!("total priority {total_priority}");
    let total_priority = simple_iterator2(&opts.input)?;
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

fn get_priority(c: char) -> Option<u64> {
    match c {
        'a'..='z' => Some(c as u64 - 'a' as u64 + 1),
        'A'..='Z' => Some(c as u64 - 'A' as u64 + 27),
        _ => None,
    }
}

fn split_elements<'a>(s: &'a str) -> (&'a str, &'a str) {
    assert!(s.len() % 2 == 0, "Cannot handle this");
    let split = s.len() / 2;
    (&s[..split], &s[split..])
}

fn common_element(left: &str, right: &str) -> Option<char> {
    let l: HashSet<char> = left.chars().collect();
    let r: HashSet<char> = right.chars().collect();
    l.intersection(&r).copied().next()
}

fn common_element2(e0: &str, e1: &str, e2: &str) -> Option<char> {
    let e0: HashSet<char> = e0.chars().collect();
    let e1: HashSet<char> = e1.chars().collect();
    let e2: HashSet<char> = e2.chars().collect();
    let e01: HashSet<char> = e0.intersection(&e1).copied().collect();
    let e12: HashSet<char> = e1.intersection(&e2).copied().collect();
    assert!(e01.intersection(&e12).count() == 1);
    e01.intersection(&e12).copied().next()
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
