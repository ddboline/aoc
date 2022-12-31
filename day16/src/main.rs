use anyhow::{format_err, Error};
use clap::Parser;
use itertools::Itertools;
use log::debug;
use maplit::{hashmap, hashset};
use smallvec::SmallVec;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Parser)]
struct Input {
    input: PathBuf,
}

fn main() -> Result<(), Error> {
    let opts = Input::parse();

    let buf = fs::read_to_string(&opts.input)?;
    let valves = ValveMap::from_str(&buf).unwrap();
    let maximum_pressure = valves.maximum_pressure_2(7, 30);
    println!("maximum pressure {maximum_pressure}");
    assert_eq!(maximum_pressure, 1754);
    let maximum_pressure_with_elephant = valves.maximum_pressure_with_elephant_2(7, 26);
    println!("maximum pressure with elephant_2 7 {maximum_pressure_with_elephant}");
    assert_eq!(maximum_pressure_with_elephant, 2474);
    Ok(())
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Label(u16);

impl Label {
    fn from_str(s: &str) -> Option<Self> {
        let v: SmallVec<[u8; 2]> = s.bytes().take(2).collect();
        if v.len() != 2 {
            None
        } else {
            match (v[0], v[1]) {
                (b'A'..=b'Z', b'A'..=b'Z') => {
                    let index: u16 = (v[0] - b'A') as u16 + (v[1] - b'A') as u16 * 26;
                    Some(Self(index))
                }
                _ => None,
            }
        }
    }

    fn as_u8s(self) -> (u8, u8) {
        let a = (self.0 % 26) as u8 + b'A';
        let b = (self.0 / 26) as u8 + b'A';
        (a, b)
    }
}

impl FromStr for Label {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str(s).ok_or_else(|| format_err!("Failed parse"))
    }
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (a, b) = self.as_u8s();
        write!(f, "{}{}", a as char, b as char)
    }
}

#[derive(Debug, Default, Clone)]
struct Valve {
    label: Label,
    flow_rate: usize,
    children: BTreeSet<Label>,
}

impl Valve {
    fn from_str(buf: &str) -> Option<Self> {
        let mut iter = buf.split("Valve ");
        let line = iter.nth(1)?;
        let label: Label = line.split(' ').next()?.parse().ok()?;
        let line = line.split(" has flow rate=").nth(1)?;
        let flow_rate = line.split(';').next()?.parse().ok()?;
        let mut children: BTreeSet<Label> = BTreeSet::new();
        if line.contains("; tunnels lead to valves ") {
            let line = line.split("; tunnels lead to valves ").nth(1)?;
            children.extend(
                line.split(',')
                    .filter_map(|s| s.trim().parse::<Label>().ok()),
            );
        } else if line.contains("; tunnel leads to valve ") {
            children.insert(
                line.split("; tunnel leads to valve ")
                    .nth(1)?
                    .parse()
                    .ok()?,
            );
        }
        Some(Valve {
            label,
            flow_rate,
            children,
        })
    }
}

#[derive(Debug, Default, Clone)]
struct ValveMap {
    valves: BTreeMap<Label, Valve>,
    distance_map: HashMap<(Label, Label), usize>,
}

impl ValveMap {
    fn from_str(buf: &str) -> Option<Self> {
        let mut valves = BTreeMap::new();
        for line in buf.split('\n') {
            if line.is_empty() {
                continue;
            }
            if let Some(valve) = Valve::from_str(line) {
                valves.insert(valve.label, valve);
            }
        }
        let start_node = Label::from_str("AA").unwrap();
        let mut label_list: Vec<Label> = valves
            .iter()
            .filter_map(|(l, v)| if v.flow_rate > 0 { Some(*l) } else { None })
            .collect();
        label_list.push(start_node);
        let mut valve_map = Self {
            valves,
            ..Self::default()
        };
        valve_map.distance_map = valve_map.get_distance_map();
        Some(valve_map)
    }

    fn pressure_for_order(&self, order: &[Label], max_time: usize) -> Option<(usize, usize)> {
        if order.is_empty() {
            Some((0, 0))
        } else if order.len() == 1 {
            let dist = *self.distance_map.get(&(Label::default(), order[0]))?;
            self.valves
                .get(&order[0])
                .map(|v| (v.flow_rate * (max_time - dist - 1), dist + 1))
        } else {
            let mut total_pressure = 0;
            let mut iter = order.iter();
            let mut current_label = iter.next().unwrap();
            let dist = *self.distance_map.get(&(Label::default(), *current_label))?;
            let mut current_time = dist + 1;
            total_pressure +=
                (self.valves.get(current_label)?.flow_rate) * (max_time - current_time);
            for next_label in iter {
                let (l0, l1) = if current_label < next_label {
                    (*current_label, *next_label)
                } else {
                    (*next_label, *current_label)
                };
                let dist = self.distance_map.get(&(l0, l1))?;
                if current_time + *dist + 1 > max_time {
                    break;
                }
                current_time += *dist + 1;
                total_pressure +=
                    (self.valves.get(next_label)?.flow_rate) * (max_time - current_time);
                current_label = next_label;
            }
            Some((total_pressure, current_time))
        }
    }

    fn maximum_pressure(&self, window_size: usize) -> (usize, Vec<Label>) {
        let mut maximum_pressure = 0;
        let mut max_order: Vec<Label> = Vec::new();
        let mut max_time = 0;
        let nonzero: Vec<Label> = self
            .valves
            .values()
            .sorted_by_key(|v| v.flow_rate)
            .rev()
            .filter_map(|v| if v.flow_rate > 0 { Some(v.label) } else { None })
            .collect();
        let number_permutations = nonzero.iter().permutations(window_size).count();
        for (idx, order) in nonzero.into_iter().permutations(window_size).enumerate() {
            if idx % 10_000_000 == 0 {
                debug!(
                    "idx {idx} / {number_permutations} - {maximum_pressure} -- {}",
                    max_order.iter().map(|x| x.to_string()).join(",")
                );
            }
            if let Some((pressure, time)) = self.pressure_for_order(&order, 30) {
                if pressure > maximum_pressure {
                    maximum_pressure = pressure;
                    max_order = order.clone();
                    max_time = time;
                }
            }
        }
        debug!(
            "maximum pressure {window_size} {maximum_pressure} max_order {} time {}",
            max_order.iter().map(|x| x.to_string()).join(","),
            max_time,
        );
        (maximum_pressure, max_order)
    }

    fn maximum_pressure_2(&self, window_size: usize, max_time: usize) -> usize {
        let (mut max_pressure, mut max_order) = self.maximum_pressure(window_size);
        loop {
            let remaining: Vec<Label> = self
                .valves
                .values()
                .sorted_by_key(|v| v.flow_rate)
                .rev()
                .filter_map(|v| {
                    if v.flow_rate > 0 && !max_order.contains(&v.label) {
                        Some(v.label)
                    } else {
                        None
                    }
                })
                .collect();
            if remaining.is_empty() {
                break;
            }
            if let Some((new_max, new_order)) =
                self.add_one_element(&max_order, &remaining, max_time)
            {
                if new_max > max_pressure {
                    max_pressure = new_max;
                    max_order = new_order;
                    debug!(
                        "max_pressure {max_pressure} max_order {}",
                        max_order.iter().map(|x| x.to_string()).join(",")
                    );
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        debug!(
            "max_pressure {max_pressure} max_order {}",
            max_order.iter().map(|x| x.to_string()).join(",")
        );
        max_pressure
    }

    fn add_one_element(
        &self,
        current: &[Label],
        remaining: &[Label],
        max_time: usize,
    ) -> Option<(usize, Vec<Label>)> {
        let (mut max_pressure, _) = self.pressure_for_order(current, max_time)?;
        let mut max_order = current.to_vec();
        for new_element in remaining {
            for i in 0..current.len() + 1 {
                let mut new = current.to_vec();
                new.insert(i, *new_element);
                if let Some((pressure, _)) = self.pressure_for_order(&new, max_time) {
                    if pressure > max_pressure {
                        max_pressure = pressure;
                        max_order = new;
                    }
                }
            }
        }
        Some((max_pressure, max_order))
    }

    fn maximum_pressure_with_elephant_2(&self, window_size: usize, max_time: usize) -> usize {
        let (
            mut max_pressure,
            mut max_pressure0,
            mut max_pressure1,
            mut max_order0,
            mut max_order1,
        ) = self.maximum_pressure_with_elephant(window_size);
        loop {
            let mut empty0 = false;
            let mut empty1 = false;
            let remaining: Vec<Label> = self
                .valves
                .values()
                .sorted_by_key(|v| v.flow_rate)
                .rev()
                .filter_map(|v| {
                    if v.flow_rate > 0
                        && !max_order0.contains(&v.label)
                        && !max_order1.contains(&v.label)
                    {
                        Some(v.label)
                    } else {
                        None
                    }
                })
                .collect();
            if remaining.is_empty() {
                break;
            }
            if let Some((new_pressure0, new_order0)) =
                self.add_one_element(&max_order0, &remaining, max_time)
            {
                if new_pressure0 + max_pressure1 > max_pressure {
                    max_pressure0 = new_pressure0;
                    max_pressure = max_pressure0 + max_pressure1;
                    max_order0 = new_order0;
                } else {
                    empty0 = true;
                }
            }
            let remaining: Vec<Label> = self
                .valves
                .values()
                .sorted_by_key(|v| v.flow_rate)
                .rev()
                .filter_map(|v| {
                    if v.flow_rate > 0
                        && !max_order0.contains(&v.label)
                        && !max_order1.contains(&v.label)
                    {
                        Some(v.label)
                    } else {
                        None
                    }
                })
                .collect();
            if remaining.is_empty() {
                break;
            }
            if let Some((new_pressure1, new_order1)) =
                self.add_one_element(&max_order1, &remaining, max_time)
            {
                if max_pressure0 + new_pressure1 > max_pressure {
                    max_pressure1 = new_pressure1;
                    max_pressure = max_pressure0 + max_pressure1;
                    max_order1 = new_order1;
                } else {
                    empty1 = true;
                }
            } else {
                break;
            }
            debug!(
                "maximum pressure2 {window_size} {max_pressure} max_order {} {}",
                max_order0.iter().map(|x| x.to_string()).join(","),
                max_order1.iter().map(|x| x.to_string()).join(","),
            );
            if empty0 && empty1 {
                break;
            }
        }
        let (pressure0, time0) = self.pressure_for_order(&max_order0, max_time).unwrap();
        let (pressure1, time1) = self.pressure_for_order(&max_order1, max_time).unwrap();
        debug!("maximum pressure0 {pressure0} {time0} pressure1 {pressure1} {time1}");
        max_pressure
    }

    fn maximum_pressure_with_elephant(
        &self,
        window_size: usize,
    ) -> (usize, usize, usize, Vec<Label>, Vec<Label>) {
        let mut maximum_pressure = 0;
        let mut max_pressure0 = 0;
        let mut max_pressure1 = 0;
        let mut max_order0: Vec<Label> = Vec::new();
        let mut max_order1: Vec<Label> = Vec::new();
        let mut max_time0 = 0;
        let mut max_time1 = 0;
        let nonzero: Vec<Label> = self
            .valves
            .values()
            .sorted_by_key(|v| v.flow_rate)
            .rev()
            .filter_map(|v| if v.flow_rate > 0 { Some(v.label) } else { None })
            .collect();
        let number_permutations = nonzero.iter().permutations(window_size).count();
        for (idx, order) in nonzero.into_iter().permutations(window_size).enumerate() {
            let order0: Vec<Label> = order
                .iter()
                .enumerate()
                .filter_map(|(i, l)| if i % 2 == 0 { Some(*l) } else { None })
                .collect();
            let order1: Vec<Label> = order
                .iter()
                .enumerate()
                .filter_map(|(i, l)| if i % 2 == 1 { Some(*l) } else { None })
                .collect();

            if idx % 10_000_000 == 0 {
                debug!(
                    "idx {idx} / {number_permutations} - {maximum_pressure} -- {} {} {} {}",
                    max_order0.iter().map(|x| x.to_string()).join(","),
                    max_order1.iter().map(|x| x.to_string()).join(","),
                    max_time0,
                    max_time1,
                );
            }
            if let Some(((pressure0, time0), (pressure1, time1))) = self
                .pressure_for_order(&order0, 26)
                .and_then(|(pressure0, time0)| {
                    self.pressure_for_order(&order1, 26)
                        .map(|(pressure1, time1)| ((pressure0, time0), (pressure1, time1)))
                })
            {
                let pressure = pressure0 + pressure1;
                if pressure > maximum_pressure {
                    maximum_pressure = pressure;
                    max_pressure0 = pressure0;
                    max_pressure1 = pressure1;
                    max_order0 = order0;
                    max_order1 = order1;
                    max_time0 = time0;
                    max_time1 = time1;
                }
            }
        }
        debug!(
            "maximum pressure {window_size} {maximum_pressure} max_order {} {} time {} {}",
            max_order0.iter().map(|x| x.to_string()).join(","),
            max_order1.iter().map(|x| x.to_string()).join(","),
            max_time0,
            max_time1,
        );
        (
            maximum_pressure,
            max_pressure0,
            max_pressure1,
            max_order0,
            max_order1,
        )
    }

    fn get_distance_map(&self) -> HashMap<(Label, Label), usize> {
        let start_node = Label::from_str("AA").unwrap();
        let mut label_list: Vec<Label> = self
            .valves
            .iter()
            .filter_map(|(l, v)| if v.flow_rate > 0 { Some(*l) } else { None })
            .collect();
        label_list.push(start_node);
        let mut distance_map = HashMap::new();
        for label0 in &label_list {
            for label1 in &label_list {
                if label0 >= label1 {
                    continue;
                }
                if distance_map.contains_key(&(*label0, *label1)) {
                    continue;
                }
                if let Some(minimum_steps) = self.minimum_steps(*label0, *label1) {
                    distance_map.insert((*label0, *label1), minimum_steps);
                }
            }
        }
        distance_map
    }

    fn minimum_steps(&self, start: Label, end: Label) -> Option<usize> {
        let mut nodes: HashMap<Label, usize> = hashmap! {start => 0};
        let mut tentative = hashset! {start};
        let mut visited: HashSet<Label> = HashSet::new();
        loop {
            if visited.contains(&end) {
                break;
            }
            let mut next_label: Option<(Label, usize)> = None;
            for t in &tentative {
                let dist = nodes.get(t)?;
                if let Some((_, d)) = next_label {
                    if dist < &d {
                        next_label.replace((*t, *dist));
                    }
                } else {
                    next_label.replace((*t, *dist));
                }
            }
            if let Some((label, dist)) = next_label {
                tentative.remove(&label);
                let new_dist = dist + 1;
                for child in &self.valves.get(&label)?.children {
                    if visited.contains(child) {
                        continue;
                    }
                    if let Some(min_dist) = nodes.get_mut(child) {
                        if new_dist < *min_dist {
                            *min_dist = new_dist;
                        }
                    } else {
                        nodes.insert(*child, new_dist);
                    }
                    tentative.insert(*child);
                }
                visited.insert(label);
            } else {
                break;
            }
        }
        nodes.get(&end).copied()
    }
}

pub static TEST_DATA: &str = "
Valve AA has flow rate=0; tunnels lead to valves DD, II, BB
Valve BB has flow rate=13; tunnels lead to valves CC, AA
Valve CC has flow rate=2; tunnels lead to valves DD, BB
Valve DD has flow rate=20; tunnels lead to valves CC, AA, EE
Valve EE has flow rate=3; tunnels lead to valves FF, DD
Valve FF has flow rate=0; tunnels lead to valves EE, GG
Valve GG has flow rate=0; tunnels lead to valves FF, HH
Valve HH has flow rate=22; tunnel leads to valve GG
Valve II has flow rate=0; tunnels lead to valves AA, JJ
Valve JJ has flow rate=21; tunnel leads to valve II
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_labels() {
        let a = Label::from_str("AA").unwrap();
        assert_eq!(a.as_u8s(), (b'A', b'A'));
        let a = Label::from_str("ZZ").unwrap();
        assert_eq!(a.as_u8s(), (b'Z', b'Z'));
    }

    #[test]
    fn test() {
        let valves = ValveMap::from_str(TEST_DATA).unwrap();
        for v in valves.valves.values() {
            println!("{} {}", v.label, v.flow_rate);
        }
        assert_eq!(valves.valves.len(), 10);
        let nonzero: Vec<_> = valves.valves.values().filter(|v| v.flow_rate > 0).collect();
        assert_eq!(nonzero.len(), 6);
        let permutations = nonzero.iter().permutations(6).count();
        assert_eq!(permutations, 720);
    }

    #[test]
    fn test_minimum_steps() {
        let valves = ValveMap::from_str(TEST_DATA).unwrap();
        let aa = Label::from_str("AA").unwrap();
        let bb = Label::from_str("BB").unwrap();
        let cc = Label::from_str("CC").unwrap();
        let dd = Label::from_str("DD").unwrap();
        println!("aa {aa} bb {bb}");
        let min_steps = valves.minimum_steps(aa, bb).unwrap();
        assert_eq!(min_steps, 1);
        let min_steps = valves.minimum_steps(aa, cc).unwrap();
        assert_eq!(min_steps, 2);
        let min_steps = valves.minimum_steps(aa, dd).unwrap();
        assert_eq!(min_steps, 1);
    }

    #[test]
    fn test_get_distance_map() {
        let valves = ValveMap::from_str(TEST_DATA).unwrap();
        let distance_map = valves.get_distance_map();
        assert_eq!(distance_map.len(), 21);

        let buf = include_str!("../input.txt");
        let valves = ValveMap::from_str(buf).unwrap();
        let nonzero: Vec<_> = valves.valves.values().filter(|v| v.flow_rate > 0).collect();
        assert_eq!(nonzero.len(), 15);
        let permutations = nonzero.iter().permutations(nonzero.len()).count();
        assert_eq!(permutations, 1_307_674_368_000);
        let permutations = nonzero.iter().permutations(6).count();
        assert_eq!(permutations, 3_603_600);
        let permutations = nonzero.iter().permutations(7).count();
        assert_eq!(permutations, 32_432_400);
        let permutations = nonzero.iter().permutations(8).count();
        assert_eq!(permutations, 259_459_200);

        let distance_map = valves.get_distance_map();
        for ((l0, l1), v) in &distance_map {
            println!("({l0},{l1}) {v}");
        }
        println!("distance_map {}", distance_map.len());
    }

    #[test]
    fn test_maximum_pressure() {
        let valves = ValveMap::from_str(TEST_DATA).unwrap();
        let (maximum_pressure, _) = valves.maximum_pressure(6);
        assert_eq!(maximum_pressure, 1651);
        let maximum_pressure = valves.maximum_pressure_2(4, 30);
        assert_eq!(maximum_pressure, 1651);
    }

    #[test]
    fn test_pressure_for_order() {
        let valves = ValveMap::from_str(TEST_DATA).unwrap();
        let bb = Label::from_str("BB").unwrap();
        let cc = Label::from_str("CC").unwrap();
        let dd = Label::from_str("DD").unwrap();
        let ee = Label::from_str("EE").unwrap();
        let hh = Label::from_str("HH").unwrap();
        let jj = Label::from_str("JJ").unwrap();
        let order = [dd, bb, jj, hh, ee, cc];
        let (pressure, time) = valves.pressure_for_order(&order, 30).unwrap();
        assert_eq!(pressure, 1651);
        let permutations = order.iter().permutations(6).count();
        println!("permutations {permutations}");
        assert_eq!(permutations, 720);
    }

    #[test]
    fn test_maximum_pressure_with_elephant() {
        let valves = ValveMap::from_str(TEST_DATA).unwrap();
        let (maximum_pressure, ..) = valves.maximum_pressure_with_elephant(6);
        assert_eq!(maximum_pressure, 1707);
    }
}
