use anyhow::Error;
use clap::Parser;
use serde_json::Value;
use std::cmp::Ordering;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
struct Input {
    input: PathBuf,
}

fn main() -> Result<(), Error> {
    let opts = Input::parse();

    let buf = fs::read_to_string(&opts.input)?;
    let packets = Packets::from_str(&buf)?;
    let total = packets.sum_of_indicies();
    println!("total {total}");
    assert_eq!(total, 6415);

    let buf = format!("{buf}\n\n[[2]]\n[[6]]");
    let mut packets = Packets::from_str(&buf)?;
    let decoder_key = packets.find_decoder_key();
    println!("decoder_key {decoder_key}");
    assert_eq!(decoder_key, 20056);
    Ok(())
}

#[derive(Debug, PartialEq)]
enum PacketElement {
    List(Vec<PacketElement>),
    Int(i64),
}

impl PacketElement {
    fn from_value(value: Value) -> Option<Self> {
        match value {
            Value::Number(n) => n.as_i64().map(Self::Int),
            Value::Array(array) => {
                let list: Vec<_> = array.into_iter().filter_map(Self::from_value).collect();
                Some(Self::List(list))
            }
            _ => None,
        }
    }

    fn compare_element(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Int(i0), Self::Int(i1)) => i0.cmp(i1),
            (Self::List(l0), Self::List(l1)) => {
                for (e0, e1) in l0.iter().zip(l1.iter()) {
                    match e0.compare_element(e1) {
                        c @ Ordering::Greater | c @ Ordering::Less => {
                            return c;
                        }
                        _ => {}
                    }
                }
                l0.len().cmp(&l1.len())
            }
            (Self::Int(i), Self::List(_)) => Self::List(vec![Self::Int(*i)]).compare_element(other),
            (Self::List(_), Self::Int(i)) => self.compare_element(&Self::List(vec![Self::Int(*i)])),
        }
    }
}

#[derive(Debug)]
struct Packets(Vec<PacketElement>);

impl Packets {
    fn from_str(buf: &str) -> Result<Self, Error> {
        let mut elements = Vec::new();
        for line in buf.split('\n') {
            if line.is_empty() {
                continue;
            }
            let value: Value = serde_json::from_str(line)?;
            if let Some(element) = PacketElement::from_value(value) {
                elements.push(element);
            }
        }
        Ok(Self(elements))
    }

    fn sum_of_indicies(&self) -> usize {
        let mut total = 0;
        for (index, v) in self.0.chunks(2).enumerate() {
            if v.len() != 2 {
                continue;
            }
            if v[0].compare_element(&v[1]) == Ordering::Less {
                total += index + 1;
            }
        }
        total
    }

    fn find_decoder_key(&mut self) -> usize {
        let mut decoder_key = 1;
        self.0.sort_by(|x, y| x.compare_element(y));
        for (i, p) in self.0.iter().enumerate() {
            if let PacketElement::List(v) = p {
                if v.len() == 1 {
                    if let PacketElement::List(x) = &v[0] {
                        if x.len() == 1 {
                            if let PacketElement::Int(y) = x[0] {
                                if y == 2 || y == 6 {
                                    decoder_key *= i + 1;
                                }
                            }
                        }
                    }
                }
            }
            if p == &PacketElement::List(vec![PacketElement::Int(2)])
                || p == &PacketElement::List(vec![PacketElement::Int(6)])
            {
                decoder_key *= i + 1;
            }
        }
        decoder_key
    }
}

pub static TEST_DATA: &str = "
[1,1,3,1,1]
[1,1,5,1,1]

[[1],[2,3,4]]
[[1],4]

[9]
[[8,7,6]]

[[4,4],4,4]
[[4,4],4,4,4]

[7,7,7,7]
[7,7,7]

[]
[3]

[[[]]]
[[]]

[1,[2,[3,[4,[5,6,7]]]],8,9]
[1,[2,[3,[4,[5,6,0]]]],8,9]
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() -> Result<(), Error> {
        let packets = Packets::from_str(TEST_DATA)?;
        let total = packets.sum_of_indicies();
        assert_eq!(total, 13);

        let new_buf = format!("{TEST_DATA}\n\n[[2]]\n[[6]]");
        let mut packets = Packets::from_str(&new_buf)?;
        let decoder_key = packets.find_decoder_key();
        assert_eq!(decoder_key, 140);
        Ok(())
    }
}
