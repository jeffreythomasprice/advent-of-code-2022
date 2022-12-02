use std::io::{self, BufRead};

struct Elf {
    food: Vec<u64>,
}

impl Elf {
    fn new() -> Elf {
        Elf { food: Vec::new() }
    }

    fn add(&mut self, s: &str) -> Result<(), String> {
        match s.parse::<u64>() {
            Ok(value) => Ok(self.food.push(value)),
            Err(e) => Err(format!("error parsing \"{s}\": ") + &e.to_string()),
        }
    }

    fn has_food(&self) -> bool {
        return !self.food.is_empty();
    }

    fn total(&self) -> u64 {
        self.food.iter().fold(0, |total, x| total + x)
        // self.food.iter().sum()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut elves: Vec<Box<Elf>> = Vec::new();
    let mut current = Box::new(Elf::new());

    fn push_grouping(elves: &mut Vec<Box<Elf>>, e: Box<Elf>) -> Box<Elf> {
        if e.has_food() {
            elves.push(e);
            Box::new(Elf::new())
        } else {
            e
        }
    }

    for line in io::BufReader::new(io::stdin()).lines() {
        match line {
            Ok(s) => match s {
                _ if s.is_empty() => {
                    current = push_grouping(&mut elves, current);
                    Ok(())
                }
                _ => current.add(&s),
            },
            Err(e) => Err(format!("line read error {e}")),
        }?;
    }
    push_grouping(&mut elves, current);

    for e in elves.iter() {
        println!("elf has {} values with {} total", e.food.len(), e.total());
        for value in e.food.iter() {
            println!("value = {}", value);
        }
        println!("");
    }
    println!("");

    let best = match elves
        .iter()
        .reduce(|a, b| if b.total() > a.total() { b } else { a })
    {
        Some(x) => Ok(x),
        None => Err("no input"),
    }?;
    println!("best elf had {} total", best.total());

    Ok(())
}
