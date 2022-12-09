use anyhow::{format_err, Error};
use clap::{Parser};
use smallvec::SmallVec;
use std::collections::BTreeMap;
use std::fmt::Write;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
struct Input {
    input: PathBuf,
}

fn main() -> Result<(), Error> {
    let opts = Input::parse();

    let buf = fs::read_to_string(&opts.input)?;
    let node_buffer = process_buf(&buf)?;
    let seq10000 = node_buffer.sum_dir_leq_100000(0);
    println!("seq10000 {seq10000}");
    assert_eq!(seq10000, 1367870);

    let total_size_root = node_buffer.total_size(0);
    let threshold = 30_000_000 + total_size_root - 70_000_000;
    let (_, size) = node_buffer.smallest_directory_geq(0, threshold).unwrap();
    println!("size {size}");
    assert_eq!(size, 549173);

    let _ = node_buffer.debug_output(0, 0);
    Ok(())
}

fn process_buf(buf: &str) -> Result<NodeBuffer, Error> {
    let mut node_buffer = NodeBuffer::new();
    for line in buf.split('\n') {
        if line.is_empty() {
            continue;
        }
        match CommandLine::from_str(line)? {
            CommandLine::Cd("/") => {
                node_buffer.current_directory = 0;
            }
            CommandLine::Cd("..") => {
                let current_directory = node_buffer
                    .get_current_directory()
                    .ok_or_else(|| format_err!("Cannot find current_directory"))?;
                let parent_directory = current_directory
                    .get_parent_directory()
                    .ok_or_else(|| format_err!("No parent directory"))?;
                node_buffer.current_directory = parent_directory;
            }
            CommandLine::Cd(d) => {
                node_buffer
                    .change_current_directory(d)
                    .ok_or_else(|| format_err!("Directory does not exist {d} {line}"))?;
            }
            CommandLine::Ls => {}
            CommandLine::Dir(d) => {
                node_buffer
                    .insert_directory(d)
                    .ok_or_else(|| format_err!("Failed to insert directory"))?;
            }
            CommandLine::File { name, size } => {
                node_buffer
                    .insert_file(name, size)
                    .ok_or_else(|| format_err!("Failed to insert file"))?;
            }
        }
    }
    Ok(node_buffer)
}

enum CommandLine<'a> {
    Cd(&'a str),
    Ls,
    Dir(&'a str),
    File { name: &'a str, size: usize },
}

impl<'a> CommandLine<'a> {
    fn from_str(s: &'a str) -> Result<Self, Error> {
        let v: SmallVec<[&str; 3]> = s.split(' ').take(3).collect();
        if v[0] == "$" {
            if v[1] == "cd" && v.len() == 3 {
                Ok(Self::Cd(v[2]))
            } else if v[1] == "ls" {
                Ok(Self::Ls)
            } else {
                Err(format_err!("Command does not parse"))
            }
        } else if v[0] == "dir" {
            Ok(Self::Dir(v[1]))
        } else if v.len() == 2 {
            let size = v[0].parse()?;
            Ok(Self::File { name: v[1], size })
        } else {
            Err(format_err!("File does not parse {v:?}"))
        }
    }
}

#[derive(Debug)]
enum Inode<'a> {
    Directory {
        name: &'a str,
        parent_directory: Option<usize>,
        child_directories: BTreeMap<&'a str, usize>,
        child_files: BTreeMap<&'a str, usize>,
    },
    File {
        parent_directory: usize,
        name: &'a str,
        size: usize,
    },
}

impl Default for Inode<'_> {
    fn default() -> Self {
        Self::Directory {
            name: "/",
            parent_directory: None,
            child_directories: BTreeMap::new(),
            child_files: BTreeMap::new(),
        }
    }
}

impl<'a> Inode<'a> {
    fn get_parent_directory(&self) -> Option<usize> {
        match self {
            Self::Directory {
                parent_directory, ..
            } => *parent_directory,
            Self::File {
                parent_directory, ..
            } => Some(*parent_directory),
        }
    }

    fn new_directory(name: &'a str, parent_directory: Option<usize>) -> Self {
        Self::Directory {
            name,
            parent_directory,
            child_directories: BTreeMap::new(),
            child_files: BTreeMap::new(),
        }
    }

    fn get_directory(&self, name: &'a str) -> Option<usize> {
        if let Inode::Directory {
            child_directories, ..
        } = self
        {
            child_directories.get(name).copied()
        } else {
            None
        }
    }

    fn insert_child_file(&mut self, child_name: &'a str, child_index: usize) -> Option<()> {
        if let Inode::Directory { child_files, .. } = self {
            child_files.insert(child_name, child_index);
            Some(())
        } else {
            None
        }
    }

    fn insert_child_directory(&mut self, child_name: &'a str, child_index: usize) -> Option<()> {
        if let Inode::Directory {
            child_directories, ..
        } = self
        {
            child_directories.insert(child_name, child_index);
            Some(())
        } else {
            None
        }
    }

    fn debug_output(&self) -> String {
        match self {
            Inode::File { name, size, .. } => {
                format!("- {name} (file, size={size})")
            }
            Inode::Directory {
                name,
                parent_directory,
                ..
            } => {
                if parent_directory.is_some() {
                    format!("- {name} (dir)")
                } else {
                    "- / (dir)".to_string()
                }
            }
        }
    }
}

#[derive(Default)]
struct NodeBuffer<'a> {
    buffer: Vec<Inode<'a>>,
    current_directory: usize,
}

impl<'a> NodeBuffer<'a> {
    fn new() -> Self {
        let mut node_buffer = NodeBuffer::default();
        node_buffer.buffer.push(Inode::default());
        node_buffer.current_directory = 0;
        node_buffer
    }

    fn get_current_directory(&mut self) -> Option<&mut Inode<'a>> {
        let current_directory = self.current_directory;
        self.buffer.get_mut(current_directory)
    }

    fn insert_file(&mut self, name: &'a str, size: usize) -> Option<()> {
        let file_index = self.buffer.len();
        self.buffer.push(Inode::File {
            parent_directory: self.current_directory,
            name,
            size,
        });
        self.get_current_directory()?
            .insert_child_file(name, file_index)
    }

    fn insert_directory(&mut self, name: &'a str) -> Option<()> {
        let parent_directory = Some(self.current_directory);
        let index = self.buffer.len();
        let directory_inode = Inode::new_directory(name, parent_directory);
        self.buffer.push(directory_inode);
        self.get_current_directory()?
            .insert_child_directory(name, index)
    }

    fn change_current_directory(&mut self, name: &'a str) -> Option<()> {
        let index = self.get_current_directory()?.get_directory(name)?;
        self.current_directory = index;
        Some(())
    }

    fn debug_output(&self, index: usize, indent: usize) -> String {
        let current_node = self.buffer.get(index).unwrap();
        let mut output = String::new();
        let indent_str: String = (0..indent).map(|_| "  ").collect();
        writeln!(&mut output, "{indent_str}{}", current_node.debug_output()).unwrap();
        if let Inode::Directory {
            child_directories,
            child_files,
            ..
        } = current_node
        {
            for d in child_directories.values() {
                let buf = self.debug_output(*d, indent + 1);
                if !buf.is_empty() {
                    write!(&mut output, "{buf}").unwrap();
                }
            }
            for f in child_files.values() {
                if let Some(node) = self.buffer.get(*f) {
                    writeln!(&mut output, "{indent_str}  {}", node.debug_output()).unwrap();
                }
            }
        }
        output
    }

    fn total_size(&self, index: usize) -> usize {
        let current_node = self.buffer.get(index).unwrap();
        let mut total_size = 0;
        if let Inode::Directory {
            child_directories,
            child_files,
            ..
        } = current_node
        {
            for d in child_directories.values() {
                total_size += self.total_size(*d);
            }
            for f in child_files.values() {
                if let Some(Inode::File { size, .. }) = self.buffer.get(*f) {
                    total_size += *size;
                }
            }
        }
        total_size
    }

    fn sum_dir_leq_100000(&self, index: usize) -> usize {
        let current_node = self.buffer.get(index).unwrap();
        let mut total_size = 0;
        let size = self.total_size(0);
        if size <= 100_000 {
            total_size += size;
        }
        if let Inode::Directory {
            child_directories, ..
        } = current_node
        {
            for d in child_directories.values() {
                let size = self.total_size(*d);
                if size <= 100_000 {
                    total_size += size
                }
                total_size += self.sum_dir_leq_100000(*d);
            }
        }
        total_size
    }

    fn smallest_directory_geq(&self, index: usize, threshold: usize) -> Option<(usize, usize)> {
        let current_node = self.buffer.get(index).unwrap();
        let mut smallest_directory = None;

        let size = self.total_size(index);
        if size >= threshold {
            smallest_directory.replace((0, size));
        }
        if let Inode::Directory {
            child_directories, ..
        } = current_node
        {
            for d in child_directories.values() {
                if let Some((i, s)) = self.smallest_directory_geq(*d, threshold) {
                    if let Some((_, size)) = smallest_directory {
                        if s < size {
                            smallest_directory.replace((i, s));
                        }
                    } else {
                        smallest_directory.replace((i, s));
                    }
                }
            }
        }
        smallest_directory
    }
}

pub static TEST_BUF: &str = "
$ cd /
$ ls
dir a
14848514 b.txt
8504156 c.dat
dir d
$ cd a
$ ls
dir e
29116 f
2557 g
62596 h.lst
$ cd e
$ ls
584 i
$ cd ..
$ cd ..
$ cd d
$ ls
4060174 j
8033020 d.log
5626152 d.ext
7214296 k
";

pub static TEST_OUTPUT: &str = "\
- / (dir)
  - a (dir)
    - e (dir)
      - i (file, size=584)
    - f (file, size=29116)
    - g (file, size=2557)
    - h.lst (file, size=62596)
  - d (dir)
    - d.ext (file, size=5626152)
    - d.log (file, size=8033020)
    - j (file, size=4060174)
    - k (file, size=7214296)
  - b.txt (file, size=14848514)
  - c.dat (file, size=8504156)
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_buf() -> Result<(), Error> {
        let node_buffer = process_buf(TEST_BUF)?;
        let output = node_buffer.debug_output(0, 0);
        assert_eq!(&output, TEST_OUTPUT);
        assert_eq!(node_buffer.sum_dir_leq_100000(0), 95437);

        let total_size_root = node_buffer.total_size(0);
        let threshold = 30_000_000 + total_size_root - 70_000_000;
        let (_, size) = node_buffer.smallest_directory_geq(0, threshold).unwrap();
        assert_eq!(size, 24933642);
        Ok(())
    }
}
