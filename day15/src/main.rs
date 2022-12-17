use anyhow::Error;
use clap::Parser;
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
struct Input {
    input: PathBuf,
}

fn main() -> Result<(), Error> {
    let opts = Input::parse();

    let buf = fs::read_to_string(&opts.input)?;
    let cave = Cave::from_str(&buf).unwrap();
    let number_covered_positions = cave.number_covered_positions(2000000);
    println!("number_covered_positions {number_covered_positions}");
    assert_eq!(number_covered_positions, 5100463);

    let freq = cave.find_tuning_frequency(4_000_000).unwrap();
    println!("freq {freq}");
    assert_eq!(freq, 11557863040754);
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Position {
    x: i64,
    y: i64,
}

impl Position {
    fn manhattan_distance(self, p: Position) -> u64 {
        (self.x - p.x).unsigned_abs() + (self.y - p.y).unsigned_abs()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum RangeOverlaps {
    None,
    Partial,
    Full,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Range {
    start: i64,
    end: i64,
}

impl Range {
    fn range_overlap(self, other: Self) -> RangeOverlaps {
        let (lower, upper) = if self.start <= other.start {
            (self, other)
        } else {
            (other, self)
        };
        if lower.end < upper.start {
            RangeOverlaps::None
        } else if lower.start < upper.start && lower.end < upper.end {
            RangeOverlaps::Partial
        } else {
            RangeOverlaps::Full
        }
    }

    fn merge(self, other: Self) -> Option<Self> {
        let (min, max) = if self.start <= other.start {
            (self, other)
        } else {
            (other, self)
        };
        match self.range_overlap(other) {
            RangeOverlaps::None => None,
            RangeOverlaps::Full => {
                let start = min.start;
                let end = if min.end < max.end { max.end } else { min.end };
                Some(Range { start, end })
            }
            RangeOverlaps::Partial => {
                let mut sections = [min.start, min.end, max.start, max.end];
                sections.sort();
                Some(Range {
                    start: sections[0],
                    end: sections[3],
                })
            }
        }
    }
}

#[derive(Default, Debug)]
struct NonOverlappingRanges(Vec<Range>);

impl NonOverlappingRanges {
    fn is_nonoverlapping(&self) -> bool {
        for ranges in self.0.windows(2) {
            if ranges.len() != 2 {
                continue;
            }
            if ranges[0].range_overlap(ranges[1]) != RangeOverlaps::None {
                return false;
            }
        }
        true
    }

    fn insert(&mut self, mut range: Range) {
        let mut to_remove: Vec<usize> = Vec::new();
        for (i, r) in self.0.iter().enumerate() {
            if r.end + 1 < range.start || r.start > range.end + 1 {
                continue;
            }
            if let Some(merged) = range.merge(*r) {
                range = merged;
                to_remove.push(i);
            }
        }
        to_remove.reverse();
        for i in to_remove {
            self.0.remove(i);
        }
        self.0.push(range);
        self.0.sort();
        assert!(self.is_nonoverlapping());
    }
}

#[derive(Debug)]
struct Sensor {
    sensor_position: Position,
    sensor_radius: u64,
}

#[derive(Debug)]
struct Cave {
    sensors: Vec<Sensor>,
    beacons: HashSet<Position>,
}

impl Cave {
    fn from_str(buf: &str) -> Option<Cave> {
        let re = Regex::new(
            r"Sensor at x=(\-??\d+), y=(\-??\d+): closest beacon is at x=(\-??\d+), y=(\-??\d+)",
        )
        .unwrap();
        let mut sensors = Vec::new();
        let mut beacons = HashSet::new();

        for cap in re.captures_iter(buf) {
            let x0: i64 = cap[1].parse().ok()?;
            let y0: i64 = cap[2].parse().ok()?;
            let x1: i64 = cap[3].parse().ok()?;
            let y1: i64 = cap[4].parse().ok()?;
            let sensor_position = Position { x: x0, y: y0 };
            let beacon_position = Position { x: x1, y: y1 };
            let sensor_radius = sensor_position.manhattan_distance(beacon_position);
            sensors.push(Sensor {
                sensor_position,
                sensor_radius,
            });
            beacons.insert(beacon_position);
        }

        Some(Cave { sensors, beacons })
    }

    fn get_ranges(&self, y: i64) -> NonOverlappingRanges {
        let mut covered_ranges = NonOverlappingRanges::default();
        for sensor in &self.sensors {
            let diffy = (y - sensor.sensor_position.y).abs();
            if diffy > sensor.sensor_radius as i64 {
                continue;
            }
            let available = sensor.sensor_radius as i64 - diffy;
            let start = sensor.sensor_position.x - available;
            let end = sensor.sensor_position.x + available;
            let range = Range { start, end };
            covered_ranges.insert(range);
        }
        covered_ranges
    }

    fn number_covered_positions(&self, y: i64) -> usize {
        let covered_ranges = self.get_ranges(y);
        let mut positions = 0;
        for range in covered_ranges.0 {
            positions += range.end - range.start + 1;
        }
        for beacon in &self.beacons {
            if beacon.y == y {
                positions -= 1;
            }
        }
        positions as usize
    }

    fn find_tuning_frequency(&self, max: i64) -> Option<i64> {
        for y in 0..=max {
            let covered_ranges = self.get_ranges(y);
            let mut position = None;
            for window in covered_ranges.0.windows(2) {
                let x = window[0].end + 1;
                if x == window[1].start {
                    continue;
                }
                if x >= 0 && x <= max {
                    position.replace(window[0].end + 1);
                }
            }
            if let Some(x) = position {
                return Some(x * 4_000_000 + y);
            }
        }
        None
    }
}

pub static TEST_DATA: &str = "
Sensor at x=2, y=18: closest beacon is at x=-2, y=15
Sensor at x=9, y=16: closest beacon is at x=10, y=16
Sensor at x=13, y=2: closest beacon is at x=15, y=3
Sensor at x=12, y=14: closest beacon is at x=10, y=16
Sensor at x=10, y=20: closest beacon is at x=10, y=16
Sensor at x=14, y=17: closest beacon is at x=10, y=16
Sensor at x=8, y=7: closest beacon is at x=2, y=10
Sensor at x=2, y=0: closest beacon is at x=2, y=10
Sensor at x=0, y=11: closest beacon is at x=2, y=10
Sensor at x=20, y=14: closest beacon is at x=25, y=17
Sensor at x=17, y=20: closest beacon is at x=21, y=22
Sensor at x=16, y=7: closest beacon is at x=15, y=3
Sensor at x=14, y=3: closest beacon is at x=15, y=3
Sensor at x=20, y=1: closest beacon is at x=15, y=3
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() -> Result<(), Error> {
        let cave = Cave::from_str(TEST_DATA).unwrap();
        println!("{cave:?}");
        assert_eq!(cave.number_covered_positions(10), 26);
        Ok(())
    }

    #[test]
    fn test_tuning_frequency() {
        let cave = Cave::from_str(TEST_DATA).unwrap();
        let freq = cave.find_tuning_frequency(20).unwrap();
        assert_eq!(freq, 56000011);
    }

    #[test]
    fn test_negative_x() {
        let cave = Cave::from_str("Sensor at x=20, y=-1: closest beacon is at x=15, y=3").unwrap();
        println!("{cave:?}");
        assert_eq!(cave.sensors[0].sensor_position.y, -1);
    }

    #[test]
    fn test_overlap() {
        let r0 = Range { start: 0, end: 2 };
        let r1 = Range { start: 1, end: 3 };
        assert_eq!(r0.range_overlap(r1), RangeOverlaps::Partial);
        let r1 = Range { start: 2, end: 3 };
        assert_eq!(r0.range_overlap(r1), RangeOverlaps::Partial);
        let r1 = Range { start: 3, end: 4 };
        assert_eq!(r0.range_overlap(r1), RangeOverlaps::None);
    }

    #[test]
    fn test_merge() {
        let r0 = Range { start: 0, end: 2 };
        let r1 = Range { start: 3, end: 4 };
        assert_eq!(r0.merge(r1), None);
        let r0 = Range { start: 0, end: 4 };
        let r1 = Range { start: 3, end: 4 };
        assert_eq!(r0.merge(r1), Some(r0));
        let r0 = Range { start: 0, end: 2 };
        let r1 = Range { start: 2, end: 4 };
        assert_eq!(r0.merge(r1), Some(Range { start: 0, end: 4 }));
        let r0 = Range { start: 0, end: 2 };
        let r1 = Range { start: 1, end: 3 };
        assert_eq!(r0.merge(r1), Some(Range { start: 0, end: 3 }));
    }

    #[test]
    fn test_insert() {
        let r0 = Range { start: -2, end: 14 };
        let r1 = Range { start: 16, end: 24 };
        let mut ov = NonOverlappingRanges::default();
        ov.insert(r0);
        ov.insert(r1);
        assert_eq!(ov.0.len(), 2);
    }
}
