use itertools::Itertools;
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    io::{self, BufRead},
};
use substring::Substring;

fn main() -> Result<(), Box<dyn Error>> {
    // read file
    let lines: Vec<String> = lines().collect::<Result<Vec<String>, std::io::Error>>()?;
    let mut lines = lines.iter();

    // parse the first part of the file where the stacks are defined
    let mut stacks = {
        let re = regex::Regex::new(r"^(?:(?:   )|(?:\[[A-Z]\]))(?: (?:(?:   )|(?:\[[A-Z]\])))*$")?;
        let rows = lines
            .take_while_ref(|line| re.is_match(line))
            .map(|line| {
                let mut results = Vec::new();
                for i in (0..(line.chars().count())).step_by(4) {
                    results.push(line.substring(i, i + 3));
                }
                results
            })
            .collect::<Vec<Vec<&str>>>();
        let counts = rows.iter().map(|row| row.len()).collect::<HashSet<usize>>();
        if counts.len() == 1 {
            let count = *counts.iter().next().unwrap();
            let mut results = Box::new(Vec::new());
            for _ in 0..count {
                results.push(Vec::new());
            }
            for row in rows {
                let mut i = 0;
                for element in row {
                    if element.trim().len() > 0 {
                        results[i].push(element.chars().nth(1).unwrap());
                    }
                    i += 1;
                }
            }
            for result in results.iter_mut() {
                result.reverse();
            }
            Ok(results)
        } else {
            Err(format!(
                "expected all stacks to be of equal size, got different counts {:?}",
                counts
            ))
        }
    }?;
    for line in stacks.iter() {
        println!("stcks = {:?}", line);
    }

    // parse the number line below the stacks
    let number_line = {
        let re = regex::Regex::new(r"^ [0-9] (?:  [0-9] )*$")?;
        let lines = lines
            .take_while_ref(|line| re.is_match(line))
            .collect::<Vec<&String>>();
        let line = if lines.len() != 1 {
            Err(format!(
                "expected a single number line, got {}",
                lines.len()
            ))
        } else {
            Ok(lines[0])
        }?;
        // can't actually get data out of capture groups
        // but each useful element is a 3 character range separated by space
        let mut result = Box::new(HashMap::<i32, i32>::new());
        let mut stack_index = 0;
        for i in (0..(line.chars().count())).step_by(4) {
            let char_at = line.chars().nth(i + 1).unwrap();
            if char_at.is_numeric() {
                result.insert(
                    char_at.to_digit(10).unwrap().try_into().unwrap(),
                    stack_index,
                );
            }
            stack_index += 1;
        }
        result
    };
    for (a, b) in number_line.iter() {
        println!("number line: {} -> {}", a, b);
    }

    // parse the actual instructions
    let instructions = {
        let re = regex::Regex::new("^move ([0-9]+) from ([0-9]+) to ([0-9]+)$")?;
        lines
            .filter_map(|line| {
                let captures = re.captures(line);
                if let Some(capture) = captures {
                    let count = capture[1].parse::<i32>().unwrap();
                    let from_index = capture[2].parse::<i32>().unwrap();
                    let to_index = capture[3].parse::<i32>().unwrap();
                    Some((count, from_index, to_index))
                } else {
                    None
                }
            })
            .collect::<Vec<(i32, i32, i32)>>()
    };
    for (count, from_index, to_index) in instructions.iter() {
        println!(
            "instruction: from: {}, to: {}, count: {}",
            from_index, to_index, count
        );
    }

    // execute instructions
    for (count, from_index, to_index) in instructions.iter() {
        for _ in 0..(*count) {
            match {
                let from = &mut stacks[number_line[from_index] as usize];
                from.pop()
            } {
                Some(x) => {
                    let to = &mut stacks[number_line[to_index] as usize];
                    to.push(x)
                }
                None => break,
            }
        }
    }
    for line in stacks.iter() {
        println!("stcks = {:?}", line);
    }

    // print just the tops of each stack
    {
        let results = stacks.iter().map(|stack| stack.last().unwrap()).join("");
        println!("final answer: {}", results);
    }

    Ok(())
}

fn lines() -> impl Iterator<Item = Result<String, std::io::Error>> {
    io::BufReader::new(io::stdin())
        .lines()
        .into_iter()
        .map(|line| match line {
            Ok(s) => {
                if s.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(s))
                }
            }
            Err(e) => Err(e),
        })
        .filter(|line| match line {
            Ok(None) => false,
            _ => true,
        })
        .map(|line| match line {
            Ok(Some(s)) => Ok(s),
            Err(e) => Err(e),
            _ => panic!("oops"),
        })
}
