use anyhow::{format_err, Error};
use clap::Parser;
use smallvec::SmallVec;
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
    let mut state = MonkeyState::from_str(&buf)?;
    for _ in 0..20 {
        state.run_single_round(3).unwrap();
    }
    let monkey_business = state.get_monkey_business();
    println!("monkey business {}", state.get_monkey_business());
    assert_eq!(monkey_business, 61503);
    let mut state = MonkeyState::from_str(&buf)?;
    for _ in 0..10_000 {
        state.run_single_round(1).unwrap();
    }
    let monkey_business = state.get_monkey_business();
    println!("monkey business overflowing worry {monkey_business}");
    assert_eq!(monkey_business, 14_081_365_540);
    Ok(())
}

struct MonkeyState(Vec<Monkey>);

impl MonkeyState {
    fn from_str(buf: &str) -> Result<Self, Error> {
        Monkey::monkeys_from_str(buf).map(Self)
    }

    fn run_single_round(&mut self, worry_divisor: isize) -> Option<()> {
        let number_monkeys = self.0.len();
        let product_of_divisors: isize = self.0.iter().map(|m| m.test_divisor).product();
        for idx in 0..number_monkeys {
            let mut items_to_move = Vec::new();

            let monkey = self.0.get_mut(idx)?;
            while let Some(item) = monkey.items.pop() {
                monkey.inspection_counter += 1;
                let operator = match &monkey.operation.1 {
                    Operator::Old => item,
                    Operator::Number(n) => *n,
                };
                let mut new = match &monkey.operation.0 {
                    Operation::Plus => item + operator,
                    Operation::Minus => item - operator,
                    Operation::Multiply => item * operator,
                };
                new /= worry_divisor;
                if worry_divisor == 1 && new > product_of_divisors {
                    new -= (product_of_divisors) * (new / product_of_divisors);
                }
                let monkey_index = if new % monkey.test_divisor == 0 {
                    monkey.true_monkey
                } else {
                    monkey.false_monkey
                };
                items_to_move.push((new, monkey_index));
            }
            for (new, monkey_index) in items_to_move {
                let monkey = self.0.get_mut(monkey_index)?;
                monkey.items.push(new);
            }
        }
        for monkey in self.0.iter_mut() {
            monkey.items.sort();
        }
        Some(())
    }

    fn get_monkey_business(&mut self) -> usize {
        self.0
            .sort_by_key(|monkey| -(monkey.inspection_counter as isize));
        self.0[0].inspection_counter * self.0[1].inspection_counter
    }
}

#[derive(Debug, Clone)]
struct Monkey {
    items: Vec<isize>,
    operation: (Operation, Operator),
    test_divisor: isize,
    true_monkey: usize,
    false_monkey: usize,
    inspection_counter: usize,
}

#[derive(Debug, Clone, Copy)]
enum Operator {
    Old,
    Number(isize),
}

#[derive(Debug, Clone, Copy)]
enum Operation {
    Plus,
    Minus,
    Multiply,
}

impl FromStr for Operation {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "+" => Ok(Self::Plus),
            "-" => Ok(Self::Minus),
            "*" => Ok(Self::Multiply),
            _ => Err(format_err!("Bad Operation")),
        }
    }
}

impl Monkey {
    fn monkey_from_str_vec(bufs: Vec<&str>) -> Option<Self> {
        let mut items: Vec<isize> = Vec::new();
        let mut operation: Option<Operation> = None;
        let mut operator: Option<Operator> = None;
        let mut test: Option<isize> = None;
        let mut true_monkey: Option<usize> = None;
        let mut false_monkey: Option<usize> = None;
        for buf in bufs {
            if buf.trim().starts_with("Starting items:") {
                if let Some(s) = buf.split("Starting items: ").nth(1) {
                    items = s.split(',').filter_map(|x| x.trim().parse().ok()).collect();
                }
            }
            if buf.trim().starts_with("Operation:") {
                if let Some(s) = buf.split("Operation: ").nth(1) {
                    let entries: SmallVec<[&str; 5]> = s.split(' ').collect();
                    if entries.len() == 5 {
                        operation = entries[3].parse().ok();
                        if entries[4].trim() == "old" {
                            operator.replace(Operator::Old);
                        } else {
                            let number: Option<isize> = entries[4].trim().parse().ok();
                            operator = number.map(Operator::Number);
                        }
                    }
                }
            }
            if buf.trim().starts_with("Test: divisible by") {
                if let Some(s) = buf.split("Test: divisible by ").nth(1) {
                    test = s.parse().ok();
                }
            }
            if buf.trim().starts_with("If true") {
                if let Some(s) = buf.split("If true: throw to monkey ").nth(1) {
                    true_monkey = s.trim().parse().ok();
                }
            }
            if buf.trim().starts_with("If false") {
                if let Some(s) = buf.split("If false: throw to monkey ").nth(1) {
                    false_monkey = s.trim().parse().ok();
                }
            }
        }
        Some(Self {
            items,
            operation: (operation?, operator?),
            test_divisor: test?,
            true_monkey: true_monkey?,
            false_monkey: false_monkey?,
            inspection_counter: 0,
        })
    }

    fn monkeys_from_str(buf: &str) -> Result<Vec<Self>, Error> {
        let mut monkey_buffers = Vec::new();
        let mut current_buffer = Vec::new();
        for line in buf.split('\n') {
            if line.starts_with("Monkey") && !current_buffer.is_empty() {
                monkey_buffers.push(current_buffer.clone());
                current_buffer.clear();
            }
            if !line.is_empty() {
                current_buffer.push(line);
            }
        }
        if !current_buffer.is_empty() {
            monkey_buffers.push(current_buffer.clone());
        }
        let monkeys: Vec<Self> = monkey_buffers
            .into_iter()
            .filter_map(Self::monkey_from_str_vec)
            .collect();
        Ok(monkeys)
    }
}

pub static TEST_DATA: &str = "
Monkey 0:
  Starting items: 79, 98
  Operation: new = old * 19
  Test: divisible by 23
    If true: throw to monkey 2
    If false: throw to monkey 3

Monkey 1:
  Starting items: 54, 65, 75, 74
  Operation: new = old + 6
  Test: divisible by 19
    If true: throw to monkey 2
    If false: throw to monkey 0

Monkey 2:
  Starting items: 79, 60, 97
  Operation: new = old * old
  Test: divisible by 13
    If true: throw to monkey 1
    If false: throw to monkey 3

Monkey 3:
  Starting items: 74
  Operation: new = old + 3
  Test: divisible by 17
    If true: throw to monkey 0
    If false: throw to monkey 1
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() -> Result<(), Error> {
        let monkeys = Monkey::monkeys_from_str(TEST_DATA)?;
        assert_eq!(monkeys.len(), 4);
        Ok(())
    }

    #[test]
    fn test_single_round() -> Result<(), Error> {
        let mut state = MonkeyState::from_str(TEST_DATA)?;
        state.run_single_round(3).unwrap();

        assert_eq!(state.0[0].items, vec![20, 23, 26, 27]);
        assert_eq!(state.0[1].items, vec![25, 167, 207, 401, 1046, 2080]);
        assert_eq!(state.0[2].items.len(), 0);
        assert_eq!(state.0[3].items.len(), 0);

        let mut state = MonkeyState::from_str(TEST_DATA)?;
        for _ in 0..20 {
            state.run_single_round(3).unwrap();
        }
        assert_eq!(state.get_monkey_business(), 10605);
        Ok(())
    }

    #[test]
    fn test_many_rounds_without_divisor() -> Result<(), Error> {
        let mut state = MonkeyState::from_str(TEST_DATA)?;
        for _ in 0..10_000 {
            state.run_single_round(1).unwrap();
        }
        assert_eq!(state.get_monkey_business(), 2_713_310_158);
        Ok(())
    }
}
