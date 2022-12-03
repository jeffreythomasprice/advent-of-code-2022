use std::{
    collections::HashSet,
    error::Error,
    io::{self, BufRead},
};
use substring::Substring;

fn split_line(s: &str) -> Result<(&str, &str), String> {
    let len = s.chars().count();
    if len % 2 != 0 {
        Err(format!("\"{}\" has len {}", s, len))
    } else {
        let first = s.substring(0, len / 2);
        let second = s.substring(len / 2, len);
        Ok((first, second))
    }
}

fn priority(c: char) -> Result<i32, String> {
    if c >= 'a' && c <= 'z' {
        Ok((c as i32) - ('a' as i32) + 1)
    } else if c >= 'A' && c <= 'Z' {
        Ok((c as i32) - ('A' as i32) + 27)
    } else {
        Err(format!("{} isn't in [a-zA-Z]", c))
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut total = 0;
    for line in io::BufReader::new(io::stdin()).lines() {
        match line {
            Ok(s) => {
                let (first, second) = split_line(s.as_str())?;
                let mut first_chars = HashSet::new();
                first.chars().for_each(|c| {
                    first_chars.insert(c);
                });
                let mut second_chars = HashSet::new();
                second.chars().for_each(|c| {
                    second_chars.insert(c);
                });
                let intersection = first_chars
                    .intersection(&second_chars)
                    .collect::<Vec<&char>>();
                if intersection.len() != 1 {
                    Err(format!("expected both to have exactly one char in common, first = {}, second = {}, intersection = {:?}", first, second ,intersection))
                } else {
                    let p = priority(*intersection[0])?;
                    println!(
                        "first = {}, second = {}, intersection = {:?}, priority = {}",
                        first, second, intersection, p
                    );
                    total += p;
                    Ok(())
                }
            }
            Err(e) => Err(format!("line read error {e}")),
        }?;
    }
    println!("total = {}", total);
    Ok(())
}
