use std::{
    collections::HashSet,
    error::Error,
    io::{self, Read},
};

use substring::Substring;

fn main() -> Result<(), Box<dyn Error>> {
    let mut s = String::new();
    io::BufReader::new(io::stdin()).read_to_string(&mut s)?;
    println!("input = {}", s);

    match find_first_unique_pattern(&s, 4) {
        Some(solution) => println!("solution = {:?}", solution),
        None => println!("no solution"),
    }

    Ok(())
}

fn find_first_unique_pattern(s: &str, pattern_len: usize) -> Option<usize> {
    for i in 0..(s.len() - pattern_len) {
        let possible = s
            .substring(i, i + pattern_len)
            .chars()
            .collect::<HashSet<char>>();
        if possible.len() == pattern_len {
            return Some(i + pattern_len);
        }
    }
    return None;
}
