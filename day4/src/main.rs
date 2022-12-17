use anyhow::Error;
use clap::Parser;
use smallvec::SmallVec;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

#[derive(Parser)]
struct Input {
    input: PathBuf,
}

fn main() -> Result<(), Error> {
    let opts = Input::parse();

    let overlaps = simple_iterator(&opts.input)?;
    println!("overlaps {overlaps}");
    assert_eq!(overlaps, 441);
    let overlaps = simple_iterator2(&opts.input)?;
    println!("overlaps {overlaps}");
    assert_eq!(overlaps, 861);
    Ok(())
}

fn simple_iterator(p: &Path) -> Result<u64, Error> {
    let buf = fs::read_to_string(p)?;
    let overlaps = buf
        .split('\n')
        .map(|s| {
            if s.is_empty() {
                0
            } else {
                check_if_ranges_fully_overlap(s) as u64
            }
        })
        .sum();
    Ok(overlaps)
}

fn simple_iterator2(p: &Path) -> Result<u64, Error> {
    let buf = fs::read_to_string(p)?;
    let overlaps = buf
        .split('\n')
        .map(|s| {
            if s.is_empty() {
                0
            } else {
                check_if_ranges_overlap(s) as u64
            }
        })
        .sum();
    Ok(overlaps)
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum RangeOverlaps {
    NoOverlap,
    PartialOverlap,
    FullOverlap,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Range {
    start: u64,
    end: u64,
}

fn convert_str_to_range(s: &str) -> Range {
    let v: SmallVec<[&str; 2]> = s.split('-').take(2).collect();
    let start: u64 = v[0].parse().ok().unwrap_or(0);
    let end: u64 = v[1].parse().ok().unwrap_or(0);
    assert!(start <= end);
    Range { start, end }
}

fn get_range_overlap(r0: Range, r1: Range) -> RangeOverlaps {
    let (lower, upper) = if r0.start <= r1.start {
        (r0, r1)
    } else {
        (r1, r0)
    };
    if lower.end < upper.start {
        RangeOverlaps::NoOverlap
    } else if lower.start < upper.start && lower.end < upper.end {
        RangeOverlaps::PartialOverlap
    } else {
        RangeOverlaps::FullOverlap
    }
}

fn check_if_ranges_fully_overlap(s: &str) -> bool {
    let v: SmallVec<[&str; 2]> = s.split(',').collect();
    let r0 = convert_str_to_range(v[0]);
    let r1 = convert_str_to_range(v[1]);
    get_range_overlap(r0, r1) == RangeOverlaps::FullOverlap
}

fn check_if_ranges_overlap(s: &str) -> bool {
    let v: SmallVec<[&str; 2]> = s.split(',').collect();
    let r0 = convert_str_to_range(v[0]);
    let r1 = convert_str_to_range(v[1]);
    get_range_overlap(r0, r1) != RangeOverlaps::NoOverlap
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_range_to_vec() {
        assert_eq!(convert_str_to_range("2-4"), Range { start: 2, end: 4 });
        assert_eq!(convert_str_to_range("6-6"), Range { start: 6, end: 6 });
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
