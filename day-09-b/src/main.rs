use std::{
    collections::HashSet,
    error::Error,
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

    let mut points = Vec::new();
    for _ in 0..10 {
        points.push(Point { x: 0, y: 0 })
    }
    let mut tail_positions = HashSet::new();
    tail_positions.insert(*points.last().unwrap());

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
            points[0].x += delta.x;
            points[0].y += delta.y;

            for i in 1..points.len() {
                let target = points[i - 1];
                move_towards(&mut points[i], target)
            }

            tail_positions.insert(*points.last().unwrap());

            println!("new points = {:?}", points);
        }
    }

    let result = tail_positions.len();
    println!("result = {}", result);
    Ok(result)
}

fn move_towards(point: &mut Point, target: Point) {
    let diff = Point {
        x: target.x - point.x,
        y: target.y - point.y,
    };
    let touching = diff.x.abs() <= 1 && diff.y.abs() <= 1;
    let delta = if touching {
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
    *point = Point {
        x: point.x + delta.x,
        y: point.y + delta.y,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample() -> Result<(), Box<dyn Error>> {
        assert_eq!(
            1,
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

    #[test]
    fn sample2() -> Result<(), Box<dyn Error>> {
        assert_eq!(
            36,
            do_it(
                &mut r"R 5
U 8
L 8
D 3
R 17
D 10
L 25
U 20"
                    .as_bytes(),
            )?
        );
        Ok(())
    }
}
