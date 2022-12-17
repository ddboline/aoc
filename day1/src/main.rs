use anyhow::Error;
use clap::Parser;
use smallvec::SmallVec;
use std::fs;
use std::io::{BufRead, BufReader, Read};
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
    assert_eq!(cals, 74394);

    let cals = simple_iterator3(&opts.input)?;
    println!("Max calories {cals}");
    let cals = use_bufreader3(&opts.input)?;
    println!("Max calories {cals}");
    assert_eq!(cals, 212836);
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
    Ok(_simple_iterator(&buf))
}

fn _simple_iterator(buf: &str) -> CalIndex {
    let agg = buf
        .split('\n')
        .map(|s| s.parse::<u64>().ok())
        .fold(Agg::default(), update_agg);
    update_agg(agg, None).max
}

fn use_bufreader(p: &Path) -> Result<CalIndex, Error> {
    let f = fs::File::open(p)?;
    let mut it = AggIterator::new(f);
    let agg = it.try_fold(Agg::default(), |agg, result| {
        result.map(|cals| update_agg(agg, cals))
    })?;
    Ok(agg.max)
}

struct AggIterator<T: Read> {
    reader: BufReader<T>,
    line: String,
}

impl<T: Read> AggIterator<T> {
    fn new(read: T) -> Self {
        Self {
            reader: BufReader::new(read),
            line: String::new(),
        }
    }
}

impl<T: Read> Iterator for AggIterator<T> {
    type Item = Result<Option<u64>, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.reader.read_line(&mut self.line) {
            Ok(0) => None,
            Ok(_) => {
                let cals: Option<u64> = self.line.trim().parse().ok();
                self.line.clear();
                Some(Ok(cals))
            }
            Err(e) => Some(Err(e.into())),
        }
    }
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
    Ok(_simple_iterator3(&buf))
}

fn _simple_iterator3(buf: &str) -> u64 {
    let agg = buf
        .split('\n')
        .map(|s| s.parse::<u64>().ok())
        .fold(Agg3::default(), update_agg3);
    let agg = update_agg3(agg, None);
    agg.max_elfs.into_iter().map(|x| x.cals).sum()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_iterator() -> Result<(), Error> {
        let data = include_str!("../input.txt");
        let CalIndex { index: _, cals } = _simple_iterator(data);
        assert_eq!(cals, 74394);
        Ok(())
    }

    #[test]
    fn test_simple_iterator3() -> Result<(), Error> {
        let data = include_str!("../input.txt");
        let cals = _simple_iterator3(data);
        assert_eq!(cals, 212836);
        Ok(())
    }
}
