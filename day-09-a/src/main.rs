use std::{
    cmp::{max, min},
    collections::HashSet,
    error::Error,
    fmt,
    io::{self, BufRead, BufReader},
};

#[derive(Debug)]
enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

#[derive(Copy, Clone, PartialEq, Hash, Eq, Debug)]
struct Point {
    x: i32,
    y: i32,
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut r = BufReader::new(io::stdin());
    do_it(&mut r)?;
    Ok(())
}

fn do_it(r: &mut impl std::io::Read) -> Result<usize, Box<dyn Error>> {
    let line_re = regex::Regex::new("^([UDLR]) ([0-9]+)$")?;

    let mut head_position = Point { x: 0, y: 0 };
    let mut tail_positions = vec![head_position];

    for line in BufReader::new(r).lines() {
        let line = line?;
        let c = line_re
            .captures(line.as_str())
            .ok_or(format!("unhandled line: {}", line))?;
        let direction = match &c[1] {
            "U" => Direction::UP,
            "D" => Direction::DOWN,
            "L" => Direction::LEFT,
            // "R"
            _ => Direction::RIGHT,
        };
        let distance = c[2].parse::<i32>()?;
        println!("move direction={:?}, distance={}", direction, distance);

        for _ in 0..distance {
            let delta = match direction {
                Direction::UP => Point { x: 0, y: -1 },
                Direction::DOWN => Point { x: 0, y: 1 },
                Direction::LEFT => Point { x: -1, y: 0 },
                Direction::RIGHT => Point { x: 1, y: 0 },
            };
            head_position.x += delta.x;
            head_position.y += delta.y;
            println!("new head = {:?}", head_position);

            let current_tail = *tail_positions.last().unwrap();
            let diff = Point {
                x: head_position.x - current_tail.x,
                y: head_position.y - current_tail.y,
            };
            let touching = diff.x.abs() <= 1 && diff.y.abs() <= 1;
            let tail_delta = if touching {
                Point { x: 0, y: 0 }
            } else if diff.x == 0 && diff.y < 0 {
                Point { x: 0, y: -1 }
            } else if diff.x == 0 && diff.y > 0 {
                Point { x: 0, y: 1 }
            } else if diff.x < 0 && diff.y == 0 {
                Point { x: -1, y: 0 }
            } else if diff.x > 0 && diff.y == 0 {
                Point { x: 1, y: 0 }
            } else if diff.x < 0 && diff.y < 0 {
                Point { x: -1, y: -1 }
            } else if diff.x > 0 && diff.y < 0 {
                Point { x: 1, y: -1 }
            } else if diff.x < 0 && diff.y > 0 {
                Point { x: -1, y: 1 }
            } else {
                // if diff.x > 0 && diff.y > 0
                Point { x: 1, y: 1 }
            };
            tail_positions.push(Point {
                x: current_tail.x + tail_delta.x,
                y: current_tail.y + tail_delta.y,
            });
            println!("new tail = {:?}", tail_positions.last().unwrap());
        }
    }

    let result = tail_positions.iter().collect::<HashSet<_>>().len();
    println!("result = {}", result);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample() -> Result<(), Box<dyn Error>> {
        assert_eq!(
            13,
            do_it(
                &mut r"R 4
U 4
L 3
D 1
R 4
D 1
L 5
R 2"
                .as_bytes(),
            )?
        );
        Ok(())
    }
}
