use std::{
    collections::HashSet,
    error::Error,
    io::{self, BufRead},
};

fn intersection(s1: &String, s2: &String) -> Vec<char> {
    let mut chars1 = HashSet::new();
    s1.chars().for_each(|c| {
        chars1.insert(c);
    });
    let mut chars2 = HashSet::new();
    s2.chars().for_each(|c| {
        chars2.insert(c);
    });
    chars1
        .intersection(&chars2)
        .map(|c| *c)
        .collect::<Vec<char>>()
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
    let mut grouping = Vec::<String>::new();
    for line in io::BufReader::new(io::stdin()).lines() {
        match line {
            Ok(s) => {
                grouping.push(s);

                if grouping.len() == 3 {
                    let intersection01 = intersection(&grouping[0], &grouping[1]);
                    let complete_intersection =
                        intersection(&intersection01.iter().collect(), &grouping[2]);
                    println!(
                        "lines:\n{}\n{}\n{}\nintersection = {:?}\n\n",
                        grouping[0], grouping[1], grouping[2], complete_intersection
                    );

                    if complete_intersection.len() != 1 {
                        Err(format!(
                            "expected exactly one character from that intersection"
                        ))?;
                    }

                    total += priority(complete_intersection[0])?;

                    grouping.clear();
                }
                Ok(())
            }
            Err(e) => Err(format!("line read error {e}")),
        }?;
    }
    println!("total = {}", total);
    Ok(())
}
