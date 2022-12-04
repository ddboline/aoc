use anyhow::Error;
use clap::Parser;
use smallvec::SmallVec;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

#[derive(Parser)]
struct Input {
    input: PathBuf,
}

fn main() -> Result<(), Error> {
    let opts = Input::parse();

    let CalIndex { index, cals } = simple_iterator(&opts.input)?;
    println!("Max elf: {index}, Max calories {cals}");
    let CalIndex { index, cals } = use_bufreader(&opts.input)?;
    println!("Max elf: {index}, Max calories {cals}");

    let cals = simple_iterator3(&opts.input)?;
    println!("Max calories {cals}");
    let cals = use_bufreader3(&opts.input)?;
    println!("Max calories {cals}");
    Ok(())
}

#[derive(Default, Copy, Clone, Debug)]
struct CalIndex {
    index: u64,
    cals: u64,
}

#[derive(Default, Copy, Clone)]
struct Agg {
    current: CalIndex,
    max: CalIndex,
}

fn update_agg(mut agg: Agg, cals: Option<u64>) -> Agg {
    if let Some(cals) = cals {
        agg.current.cals += cals;
    } else {
        if agg.max.cals < agg.current.cals {
            agg.max.cals = agg.current.cals;
            agg.max.index = agg.current.index;
        }
        agg.current.cals = 0;
        agg.current.index += 1;
    }
    agg
}

fn simple_iterator(p: &Path) -> Result<CalIndex, Error> {
    let buf = fs::read_to_string(p)?;
    let agg = buf
        .split('\n')
        .map(|s| s.parse::<u64>().ok())
        .fold(Agg::default(), update_agg);
    let agg = update_agg(agg, None);
    Ok(agg.max)
}

fn use_bufreader(p: &Path) -> Result<CalIndex, Error> {
    let f = fs::File::open(p)?;
    let mut buf = BufReader::new(f);
    let mut line = String::new();
    let mut agg = Agg::default();
    while buf.read_line(&mut line)? > 0 {
        let cals: Option<u64> = line.trim().parse().ok();
        line.clear();
        agg = update_agg(agg, cals);
    }
    Ok(agg.max)
}

#[derive(Default, Clone, Debug)]
struct Agg3 {
    current: CalIndex,
    max_elfs: SmallVec<[CalIndex; 4]>,
}

fn update_agg3(mut agg: Agg3, cals: Option<u64>) -> Agg3 {
    if let Some(cals) = cals {
        agg.current.cals += cals;
    } else {
        agg.max_elfs.push(agg.current);
        agg.max_elfs.sort_by_key(|ci| ci.cals);
        agg.max_elfs.reverse();
        if agg.max_elfs.len() > 3 {
            agg.max_elfs.pop();
        }
        agg.current.cals = 0;
        agg.current.index += 1;
    }
    agg
}

fn simple_iterator3(p: &Path) -> Result<u64, Error> {
    let buf = fs::read_to_string(p)?;
    let agg = buf
        .split('\n')
        .map(|s| s.parse::<u64>().ok())
        .fold(Agg3::default(), update_agg3);
    let agg = update_agg3(agg, None);
    Ok(agg.max_elfs.into_iter().map(|x| x.cals).sum())
}

fn use_bufreader3(p: &Path) -> Result<u64, Error> {
    let f = fs::File::open(p)?;
    let mut buf = BufReader::new(f);
    let mut line = String::new();
    let mut agg = Agg3::default();
    while buf.read_line(&mut line)? > 0 {
        let cals: Option<u64> = line.trim().parse().ok();
        line.clear();
        agg = update_agg3(agg, cals);
    }
    Ok(agg.max_elfs.into_iter().map(|x| x.cals).sum())
}
