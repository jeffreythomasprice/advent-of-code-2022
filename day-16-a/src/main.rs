use std::{
    collections::{HashMap, HashSet},
    error::Error,
    io::{self, BufRead, BufReader},
};

use rand::seq::{IteratorRandom, SliceRandom};

#[derive(Debug, Clone)]
struct Entry {
    name: String,
    rate: usize,
    leads_to: Vec<String>,
}

struct Map {
    entries: Vec<Entry>,
    name_to_index: HashMap<String, usize>,
    reachable: Vec<Vec<Option<usize>>>,
}

impl Map {
    fn new(entries: &Vec<Entry>) -> Result<Map, Box<dyn Error>> {
        let mut name_to_index = HashMap::new();
        let mut reachable = Vec::new();
        reachable.resize_with(entries.len(), || {
            let mut v = Vec::new();
            v.resize(entries.len(), None);
            v
        });
        // start out with distance 1 to all the ones marked as reachable
        for (index, entry) in entries.iter().enumerate() {
            name_to_index.insert(entry.name.clone(), index);
            for other in entry.leads_to.iter() {
                let other_index = entries
                    .iter()
                    .position(|e| e.name == *other)
                    .ok_or(format!("no such entry: {}", other))?;
                reachable[index][other_index] = Some(1);
            }
        }
        // need to iterate enough times that we're sure we have the shortest route
        for _ in 0..entries.len() {
            // iterate over all possible connections, A to B
            for index_a in 0..entries.len() {
                for index_b in 0..entries.len() {
                    if index_a == index_b {
                        continue;
                    }
                    if let Some(current_distance_ab) = reachable[index_a][index_b] {
                        // for all current connections B to C
                        for index_c in 0..entries.len() {
                            if index_a == index_c {
                                continue;
                            }
                            if let Some(current_distance_bc) = reachable[index_b][index_c] {
                                let proposed_distance_ac =
                                    current_distance_ab + current_distance_bc;
                                if let Some(current_distance_ac) = reachable[index_a][index_c] {
                                    if proposed_distance_ac < current_distance_ac {
                                        reachable[index_a][index_c] = Some(proposed_distance_ac);
                                    }
                                } else {
                                    reachable[index_a][index_c] = Some(proposed_distance_ac);
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(Map {
            entries: entries.to_vec(),
            name_to_index,
            reachable,
        })
    }

    fn index_of(&self, name: &String) -> Option<&usize> {
        self.name_to_index.get(name)
    }

    fn distance_between(&self, a: &String, b: &String) -> Result<Option<usize>, Box<dyn Error>> {
        let index_a = self.index_of(a).ok_or(format!("no such entry: {}", a))?;
        let index_b = self.index_of(b).ok_or(format!("no such entry: {}", b))?;
        Ok(self.reachable[*index_a][*index_b])
    }

    fn value_for_route(&self, first: String, route: &Vec<String>) -> Result<usize, Box<dyn Error>> {
        if route.len() == 0 {
            Ok(0)
        } else {
            let mut current = first.clone();
            println!("starting at {}", current);
            let mut total = 0;
            let mut rate = 0;
            let mut time_remaining = 30;
            for next in route.iter() {
                println!("moving to {}", next);
                let distance = self
                    .distance_between(&current, next)?
                    .ok_or(format!("can't move from {} to {}", current, next))?;
                let time_taken = distance + 1;
                if time_taken > time_remaining {
                    println!("out of time, can't move here");
                    break;
                }
                time_remaining -= time_taken;
                println!("time remaining = {}", time_remaining);
                total += rate * time_taken;
                println!("new total = {}", total);
                rate += self
                    .entries
                    .iter()
                    .find(|e| e.name == *next)
                    .ok_or(format!("no such entity: {}", next))?
                    .rate;
                println!("new rate = {}", rate);
                current = next.clone();
                println!("new position = {}", current);
            }
            println!("time remaining = {}", time_remaining);
            total += rate * time_remaining;
            println!("final total = {}", total);
            Ok(total)
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut r = BufReader::new(io::stdin());
    do_it(&mut r)?;
    Ok(())
}

fn do_it(r: &mut impl std::io::Read) -> Result<usize, Box<dyn Error>> {
    let re = regex::Regex::new("^Valve ([a-zA-Z]+) has flow rate=([0-9]+); tunnels? leads? to valves? ([a-zA-Z]+(?:, [a-zA-Z]+)*)$")?;

    let mut entries = Vec::new();
    for line in BufReader::new(r).lines() {
        let line = line?;
        let captures = re
            .captures(line.as_str())
            .ok_or(format!("failed to parse line: {}", line))?;
        let entry = Entry {
            name: captures[1].to_string(),
            rate: captures[2].parse()?,
            leads_to: captures[3]
                .split(", ")
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
        };
        entries.push(entry);
    }
    for e in entries.iter() {
        println!("{:?}", e);
    }
    println!("");

    let map = Map::new(&entries)?;
    println!("distances");
    for (i, e) in map.entries.iter().enumerate() {
        print!("{} ", e.name);
        for j in 0..map.entries.len() {
            let c = match map.reachable[i][j] {
                None => "-".to_string(),
                Some(distance) => format!("{}", distance),
            };
            print!("{} ", c);
        }
        println!("");
    }
    println!("");

    let valid_locations = entries
        .iter()
        .filter(|e| e.rate > 0)
        .map(|e| e.name.clone())
        .collect::<HashSet<_>>();
    println!("these are the valid nodes: {:?}", valid_locations);

    let mut rng = rand::thread_rng();

    let mut current_solution = Box::new(
        valid_locations
            .iter()
            .map(|s| s.clone())
            .collect::<Vec<_>>(),
    );
    current_solution.shuffle(&mut rng);
    println!("potential solution = {:?}", current_solution);
    let start = "AA".to_string();
    let mut current_total = map.value_for_route(start.clone(), &current_solution)?;

    for i in 0..(current_solution.len().pow(2)) {
        println!("permutation {}", i);
        let picks = (0..current_solution.len()).choose_multiple(&mut rng, 2);
        let mut new_solution = current_solution.clone();
        new_solution[picks[0]] = current_solution[picks[1]].clone();
        new_solution[picks[1]] = current_solution[picks[0]].clone();
        println!("new solution = {:?}", new_solution);
        let new_total = map.value_for_route(start.clone(), &new_solution)?;
        if new_total > current_total {
            println!("keeping new, {} > {}", new_total, current_total);
            current_solution = new_solution;
            current_total = new_total;
        } else {
            println!(
                "not a better solution, {} is not > {}",
                new_total, current_total
            );
        }
        println!("");
    }
    println!("solution = {:?}", current_solution);
    Ok(current_total)
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::*;

    #[test]
    fn sample() -> Result<(), Box<dyn Error>> {
        assert_eq!(
            1651,
            do_it(
                &mut r"Valve AA has flow rate=0; tunnels lead to valves DD, II, BB
Valve BB has flow rate=13; tunnels lead to valves CC, AA
Valve CC has flow rate=2; tunnels lead to valves DD, BB
Valve DD has flow rate=20; tunnels lead to valves CC, AA, EE
Valve EE has flow rate=3; tunnels lead to valves FF, DD
Valve FF has flow rate=0; tunnels lead to valves EE, GG
Valve GG has flow rate=0; tunnels lead to valves FF, HH
Valve HH has flow rate=22; tunnel leads to valve GG
Valve II has flow rate=0; tunnels lead to valves AA, JJ
Valve JJ has flow rate=21; tunnel leads to valve II"
                    .as_bytes()
            )?
        );
        Ok(())
    }
}
