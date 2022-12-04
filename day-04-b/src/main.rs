use std::{
    error::Error,
    io::{self, BufRead},
};

use regex::Regex;

#[derive(Debug)]
struct Range {
    min: i32,
    max: i32,
}

impl Range {
    fn new(min: i32, max: i32) -> Result<Range, String> {
        if min <= max {
            Ok(Range { min, max })
        } else {
            Err(format!("range out of order, min: {}, max: {}", min, max))
        }
    }

    fn contains_range(&self, other: &Range) -> bool {
        other.min >= self.min && other.max <= self.max
    }

    fn contains_single(&self, other: i32) -> bool {
        other >= self.min && other <= self.max
    }

    fn overlaps(&self, other: &Range) -> bool {
        self.contains_single(other.min)
            || self.contains_single(other.max)
            || other.contains_single(self.min)
            || other.contains_single(self.max)
    }
}

#[derive(Debug)]
struct Pair {
    left: Range,
    right: Range,
}

impl Pair {
    fn new(line: &str) -> Result<Pair, Box<dyn Error>> {
        let re = regex::Regex::new("^([0-9]+)-([0-9]+),([0-9]+)-([0-9]+)$")?;
        let captures = re
            .captures(line)
            .ok_or(format!("failed to match line: {}", line))?;
        Ok(Pair {
            left: Range::new(captures[1].parse()?, captures[2].parse()?)?,
            right: Range::new(captures[3].parse()?, captures[4].parse()?)?,
        })
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut count = 0;
    for line in io::BufReader::new(io::stdin()).lines() {
        match line {
            Ok(s) => {
                let pair = Pair::new(&s)?;
                println!("pair = {:?}", pair);
                if pair.left.overlaps(&pair.right) {
                    count += 1;
                }
                Ok(())
            }
            Err(e) => Err(format!("line read error {e}")),
        }?;
    }
    println!("count = {}", count);
    Ok(())
}
