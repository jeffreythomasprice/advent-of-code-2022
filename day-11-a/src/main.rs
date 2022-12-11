use std::{
    cell::RefCell,
    collections::HashMap,
    error::Error,
    io::{self, BufRead, BufReader},
    rc::Rc,
};

#[derive(Debug, Clone, Copy)]
struct Item(u32);

#[derive(Debug, Clone, Copy)]
enum Operand {
    Constant(u32),
    Old,
}

#[derive(Debug, Clone, Copy)]
enum Operation {
    Add(Operand),
    Multiply(Operand),
}

#[derive(Debug, Clone, Copy)]
struct Test(u32);

#[derive(Debug, Clone, Copy)]
struct Target(u32);

#[derive(Debug)]
struct Entity {
    items: Rc<RefCell<Vec<Item>>>,
    operation: Operation,
    test: Test,
    if_true: Target,
    if_false: Target,
}

impl Entity {
    fn check_items(&mut self, send_to: impl Fn(Target, Item)) {
        let mut items = self.items.borrow_mut();
        for Item(item) in items.iter() {
            println!("handling {:?}", Item(*item));
            let result = match self.operation {
                Operation::Add(operand) => {
                    item + match operand {
                        Operand::Constant(value) => value,
                        Operand::Old => *item,
                    }
                }
                Operation::Multiply(operand) => {
                    item * match operand {
                        Operand::Constant(value) => value,
                        Operand::Old => *item,
                    }
                }
            } / 3;
            let Test(test) = self.test;
            let test_result = result % test == 0;
            println!("{} % {} == 0 ? {}", result, test, test_result);
            let target = if test_result {
                self.if_true
            } else {
                self.if_false
            };
            let result = Item(result);
            println!("sending {:?} to {:?}", result, target);
            send_to(target, result);
        }
        items.clear();
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut r = BufReader::new(io::stdin());
    do_it(&mut r)?;
    Ok(())
}

fn do_it(r: &mut impl std::io::Read) -> Result<usize, Box<dyn Error>> {
    let entities = Rc::new(RefCell::new(parse_entities(r)?));

    let indices = Rc::new(RefCell::new(
        entities.borrow().keys().map(|i| *i).collect::<Vec<_>>(),
    ));
    indices.borrow_mut().sort();

    let debug_entities = {
        let entities = entities.clone();
        let indices = indices.clone();
        move || {
            for index in indices.borrow().iter() {
                let entity = entities.borrow().get(index).unwrap().clone();
                let entity = entity.borrow();
                println!("entitiy[{}] = {:?}", index, entity);
            }
        }
    };
    debug_entities();
    println!("");

    let mut counts = HashMap::new();
    for index in indices.borrow().iter() {
        counts.insert(*index, 0);
    }

    for round in 0..20 {
        println!("round {}", round);
        for index in indices.borrow().iter() {
            println!("entity {}", index);
            let entity = entities.borrow().get(index).unwrap().clone();
            let count = entity.borrow().items.borrow().len();
            *counts.get_mut(index).unwrap() += count;
            entity.borrow_mut().check_items(|target, item| {
                let Target(target) = target;
                let target = entities.borrow().get(&target).unwrap().clone();
                target.borrow().items.borrow_mut().push(item);
            });
        }
        debug_entities();
        println!("counts = {:?}", counts);
        println!("");
    }

    let result = {
        let mut counts = counts.iter().collect::<Vec<_>>();
        counts.sort_by(|(_, a), (_, b)| b.cmp(a));
        counts
            .iter()
            .take(2)
            .map(|(_, count)| **count)
            .reduce(|result, x| result * x)
            .unwrap()
    };
    println!("result = {}", result);

    Ok(result)
}

fn parse_entities(
    r: &mut impl std::io::Read,
) -> Result<HashMap<u32, Rc<RefCell<Entity>>>, Box<dyn Error>> {
    let header_re = regex::Regex::new(r"^\s*Monkey\s+([0-9]+):\s*$")?;
    let items_re = regex::Regex::new(r"^\s+Starting items:\s*([0-9]+(?:\s*,\s*[0-9]+)*)?\s*$")?;
    let operation_re =
        regex::Regex::new(r"^\s*Operation:\s*new\s*=\s*old\s*([+*])\s*([0-9]+|old)\s*$")?;
    let test_re = regex::Regex::new(r"^\s*Test:\s*divisible\s*by\s*([0-9]+)\s*$")?;
    let if_re = regex::Regex::new(r"^\s*If\s*(true|false):\s*throw\s*to\s*monkey\s*([0-9]+)\s*$")?;

    let current_index = RefCell::new(None);
    let current_items = RefCell::<Option<Rc<RefCell<Vec<Item>>>>>::new(None);
    let current_operation = RefCell::new(None);
    let current_test = RefCell::new(None);
    let current_if_true = RefCell::new(None);
    let current_if_false = RefCell::new(None);
    let mut results = HashMap::new();
    let mut handle_current_entity = || {
        let current_index = current_index.borrow();
        let current_items = current_items.borrow();
        let current_operation = current_operation.borrow();
        let current_test = current_test.borrow();
        let current_if_true = current_if_true.borrow();
        let current_if_false = current_if_false.borrow();
        if current_index.is_some()
            && current_items.is_some()
            && current_operation.is_some()
            && current_test.is_some()
            && current_if_true.is_some()
            && current_if_false.is_some()
        {
            results.insert(
                current_index.unwrap(),
                Rc::new(RefCell::new(Entity {
                    items: current_items.clone().unwrap(),
                    operation: current_operation.unwrap(),
                    test: current_test.unwrap(),
                    if_true: current_if_true.unwrap(),
                    if_false: current_if_false.unwrap(),
                })),
            );
            Ok(())
        } else if current_index.is_none()
            && current_items.is_none()
            && current_operation.is_none()
            && current_test.is_none()
            && current_if_true.is_none()
            && current_if_false.is_none()
        {
            // nothing to do, no item
            Ok(())
        } else {
            Err("partial item")
        }
    };
    for line in BufReader::new(r).lines() {
        let line = line?;
        println!("line = {}", line);

        if line.trim().is_empty() {
            continue;
        } else if let Some(captures) = header_re.captures(line.as_str()) {
            // process any previous entitiy that might now be finished
            handle_current_entity()?;
            let index = captures[1].parse::<u32>()?;
            println!("header, index = {}", index);
            current_index.replace(Some(index));
        } else if let Some(captures) = items_re.captures(line.as_str()) {
            let items = &captures[1];
            let items = Rc::new(RefCell::new(
                items
                    .split(",")
                    .map(|item| -> Result<Item, Box<dyn Error>> {
                        Ok(Item(item.trim().parse::<u32>()?))
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            ));
            println!("items = {:?}", items);
            current_items.replace(Some(items));
        } else if let Some(captures) = operation_re.captures(line.as_str()) {
            let operator = &captures[1];
            let operand = if &captures[2] == "old" {
                Operand::Old
            } else {
                Operand::Constant(captures[2].parse()?)
            };
            let operation = match operator {
                "+" => Operation::Add(operand),
                "*" => Operation::Multiply(operand),
                _ => Err(format!("unrecognized operand: {:?}", operand))?,
            };
            println!("operation = {:?}", operation);
            current_operation.replace(Some(operation));
        } else if let Some(captures) = test_re.captures(line.as_str()) {
            let test = Test(captures[1].parse::<u32>()?);
            println!("test = {:?}", test);
            current_test.replace(Some(test));
        } else if let Some(captures) = if_re.captures(line.as_str()) {
            let condition = &captures[1];
            let target = Target(captures[2].parse::<u32>()?);
            println!("if, condition = {}, target = {:?}", condition, target);
            match condition {
                "true" => current_if_true.replace(Some(target)),
                "false" => current_if_false.replace(Some(target)),
                _ => Err(format!("unrecognized condition: {}", condition))?,
            };
        } else {
            Err(format!("unrecognized line = {}", line))?;
        }
    }
    // done with all lines, process any entitiy at the end
    handle_current_entity()?;
    println!("");
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample() -> Result<(), Box<dyn Error>> {
        assert_eq!(
            10605,
            do_it(
                &mut r"Monkey 0:
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
    If false: throw to monkey 1"
                    .as_bytes(),
            )?
        );
        Ok(())
    }
}
