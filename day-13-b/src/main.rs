use serde_json::json;
use std::{
    cmp::Ordering,
    error::Error,
    io::{self, BufRead, BufReader},
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut r = BufReader::new(io::stdin());
    do_it(&mut r)?;
    Ok(())
}

fn do_it(r: &mut impl std::io::Read) -> Result<usize, Box<dyn Error>> {
    #[derive(PartialEq, Debug)]
    enum LineMarker {
        Normal,
        Special,
    }

    let mut lines = BufReader::new(r)
        .lines()
        // get rid of any broken lines
        .collect::<Result<Vec<_>, _>>()?
        .iter()
        // get rid of empty lines
        .filter(|line| !line.trim().is_empty())
        // parse as json
        .map(|line| serde_json::from_str::<serde_json::Value>(line))
        // get rid of any parse errors
        .collect::<Result<Vec<_>, _>>()?
        .iter()
        // add line markers
        .map(|value| (LineMarker::Normal, value.clone()))
        // .map(|value| value.clone())
        .collect::<Vec<_>>();
    if lines.len() % 2 != 0 {
        Err(format!(
            "expected an even number of lines, got {}",
            lines.len()
        ))?;
    }

    lines.push((LineMarker::Special, json!([[2]])));
    lines.push((LineMarker::Special, json!([[6]])));

    lines.sort_by(|(_, a), (_, b)| compare_pair(a, b).unwrap());

    for line in lines.iter() {
        println!("{:?}", line);
    }

    let special_line_indices = lines
        .iter()
        .enumerate()
        .filter(|(i, (marker, _))| *marker == LineMarker::Special)
        .map(|(i, _)| i + 1)
        .collect::<Vec<_>>();
    println!("special line indices = {:?}", special_line_indices);
    let result = special_line_indices[0] * special_line_indices[1];
    println!("result = {}", result);
    Ok(result)
}

fn compare_pair(
    left: &serde_json::Value,
    right: &serde_json::Value,
) -> Result<std::cmp::Ordering, Box<dyn Error>> {
    if left.is_number() && right.is_number() {
        let left = assert_number(left)?;
        let right = assert_number(right)?;
        Ok(left.cmp(&right))
    } else if left.is_array() && right.is_array() {
        let left = assert_array(left)?;
        let right = assert_array(right)?;
        let mut i = 0;
        loop {
            if i >= left.len() && i >= right.len() {
                return Ok(Ordering::Equal);
            }
            if i >= left.len() {
                return Ok(Ordering::Less);
            }
            if i >= right.len() {
                return Ok(Ordering::Greater);
            }
            let result = compare_pair(&left[i], &right[i])?;
            if result == Ordering::Less || result == Ordering::Greater {
                return Ok(result);
            }
            i += 1;
        }
    } else if left.is_number() && right.is_array() {
        let left = &json!([assert_number(left)?]);
        compare_pair(left, right)
    } else if left.is_array() && right.is_number() {
        let right = &json!([assert_number(right)?]);
        compare_pair(left, right)
    } else {
        Err(format!(
            "types aren't numbers or arrays, left = {:?}, right = {:?}",
            left, right
        ))?
    }
}

fn assert_number(value: &serde_json::Value) -> Result<i64, Box<dyn Error>> {
    Ok(value
        .as_i64()
        .ok_or(format!("expected number: {}", value))?)
}

fn assert_array(value: &serde_json::Value) -> Result<&Vec<serde_json::Value>, Box<dyn Error>> {
    Ok(value
        .as_array()
        .ok_or(format!("expected array: {}", value))?)
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::*;

    #[test]
    fn sample() -> Result<(), Box<dyn Error>> {
        assert_eq!(
            140,
            do_it(
                &mut r"[1,1,3,1,1]
[1,1,5,1,1]

[[1],[2,3,4]]
[[1],4]

[9]
[[8,7,6]]

[[4,4],4,4]
[[4,4],4,4,4]

[7,7,7,7]
[7,7,7]

[]
[3]

[[[]]]
[[]]

[1,[2,[3,[4,[5,6,7]]]],8,9]
[1,[2,[3,[4,[5,6,0]]]],8,9]"
                    .as_bytes(),
            )?
        );
        Ok(())
    }
}
