use anyhow::Error;
use clap::Parser;
use smallvec::SmallVec;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
struct Input {
    input: PathBuf,
}

fn main() -> Result<(), Error> {
    let opts = Input::parse();

    let buf = fs::read(opts.input)?;
    let index = find_marker::<4>(&buf);
    println!("start packet {index} / {}", buf.len());
    assert_eq!(index, 1707);
    let index = find_marker::<14>(&buf);
    println!("start message {index} / {}", buf.len());
    assert_eq!(index, 3697);
    Ok(())
}

fn find_marker<const L: usize>(buf: &[u8]) -> usize {
    let buf_len = buf.len();
    buf.windows(L)
        .enumerate()
        .find(|(_, c)| check_uniqueness::<L>(c))
        .map_or(buf_len, |(i, _)| i + L)
}

fn check_uniqueness<const L: usize>(v: &[u8]) -> bool {
    let mut v: SmallVec<[u8; L]> = v.iter().copied().collect();
    v.sort();
    if v.len() != L {
        false
    } else {
        for i in 0..L - 1 {
            if v[i] == v[i + 1] {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_start_packet() -> Result<(), Error> {
        assert_eq!(find_marker::<4>(b"mjqjpqmgbljsphdztnvjfqwrcgsmlb"), 7);
        assert_eq!(find_marker::<4>(b"bvwbjplbgvbhsrlpgdmjqwftvncz"), 5);
        assert_eq!(find_marker::<4>(b"nppdvjthqldpwncqszvftbrmjlhg"), 6);
        assert_eq!(find_marker::<4>(b"nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg"), 10);
        assert_eq!(find_marker::<4>(b"zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw"), 11);
        Ok(())
    }

    #[test]
    fn test_find_start_message() -> Result<(), Error> {
        assert_eq!(find_marker::<14>(b"mjqjpqmgbljsphdztnvjfqwrcgsmlb"), 19);
        assert_eq!(find_marker::<14>(b"bvwbjplbgvbhsrlpgdmjqwftvncz"), 23);
        assert_eq!(find_marker::<14>(b"nppdvjthqldpwncqszvftbrmjlhg"), 23);
        assert_eq!(find_marker::<14>(b"nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg"), 29);
        assert_eq!(find_marker::<14>(b"zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw"), 26);
        Ok(())
    }
}
