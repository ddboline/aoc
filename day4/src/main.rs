use clap::Parser;
use anyhow::Error;
use std::path::PathBuf;
use smallvec::SmallVec;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[derive(Parser)]
struct Input {
    input: PathBuf,
}

fn main() -> Result<(), Error> {
    let opts = Input::parse();

    let overlaps = simple_iterator(&opts.input)?;
    println!("overlaps {overlaps}");
    let overlaps = simple_iterator2(&opts.input)?;
    println!("overlaps {overlaps}");
    Ok(())
}

fn simple_iterator(p: &Path) -> Result<u64, Error> {
    let buf = fs::read_to_string(p)?;
    let overlaps = buf.split('\n').map(|s| {
        if s.is_empty() {
            0
        } else {
            check_if_ranges_fully_overlap(s) as u64
        }
    }).sum();
    Ok(overlaps)
}

fn convert_range_to_vec(s: &str) -> HashSet<u64> {
    let v: SmallVec<[&str; 2]> = s.split('-').take(2).collect();
    let start: u64 = v[0].parse().ok().unwrap_or(0);
    let end: u64 = v[1].parse().ok().unwrap_or(0);
    assert!(start <= end);
    (start..=end).collect()
}

fn check_if_ranges_fully_overlap(s: &str) -> bool {
    let v: SmallVec<[&str; 2]> = s.split(',').collect();
    let e0 = convert_range_to_vec(&v[0]);
    let e1 = convert_range_to_vec(&v[1]);
    let u01 = e0.union(&e1).count();
    u01 == e0.len() || u01 == e1.len()
}

fn check_if_ranges_overlap(s: &str) -> bool {
    let v: SmallVec<[&str; 2]> = s.split(',').collect();
    let e0 = convert_range_to_vec(&v[0]);
    let e1 = convert_range_to_vec(&v[1]);
    e0.intersection(&e1).count() > 0
}

fn simple_iterator2(p: &Path) -> Result<u64, Error> {
    let buf = fs::read_to_string(p)?;
    let overlaps = buf.split('\n').map(|s| {
        if s.is_empty() {
            0
        } else {
            check_if_ranges_overlap(s) as u64
        }
    }).sum();
    Ok(overlaps)
}

#[cfg(test)]
mod tests {
    use maplit::hashset;

    use super::*;

    #[test]
    fn test_convert_range_to_vec() {
        assert_eq!(convert_range_to_vec("2-4"), hashset![2,3,4]);
        assert_eq!(convert_range_to_vec("6-6"), hashset![6]);
    }

    #[test]
    fn test_check_if_ranges_fully_overlap() {
        assert!(!check_if_ranges_fully_overlap("2-4,6-8"));
        assert!(!check_if_ranges_fully_overlap("2-3,4-5"));
        assert!(!check_if_ranges_fully_overlap("5-7,7-9"));
        assert!(check_if_ranges_fully_overlap("2-8,3-7"));
        assert!(check_if_ranges_fully_overlap("6-6,4-6"));
        assert!(!check_if_ranges_fully_overlap("2-6,4-8"));
    }

    #[test]
    fn test_check_if_ranges_overlap() {
        assert!(!check_if_ranges_overlap("2-4,6-8"));
        assert!(!check_if_ranges_overlap("2-3,4-5"));
        assert!(check_if_ranges_overlap("5-7,7-9"));
        assert!(check_if_ranges_overlap("2-8,3-7"));
        assert!(check_if_ranges_overlap("6-6,4-6"));
        assert!(check_if_ranges_overlap("2-6,4-8"));
    }
}
