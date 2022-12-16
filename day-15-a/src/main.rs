use lazy_static::lazy_static;
use std::{
    error::Error,
    io::{self, BufRead, BufReader},
};

#[derive(Debug, Clone, Copy)]
struct Point {
    x: i64,
    y: i64,
}

impl Point {
    fn distance(&self, other: &Point) -> i64 {
        return (self.x - other.x).abs() + (self.y - other.y).abs();
    }
}

#[derive(Debug, Clone, Copy)]
struct Range {
    y: i64,
    min_x: i64,
    max_x: i64,
}

impl Range {
    fn count(&self) -> i64 {
        self.max_x - self.min_x
    }

    fn contains_x(&self, x: i64) -> bool {
        x >= self.min_x && x <= self.max_x
    }

    fn intersects(&self, other: &Range) -> bool {
        self.contains_x(other.min_x)
            || self.contains_x(other.max_x)
            || other.contains_x(self.min_x)
            || other.contains_x(self.max_x)
    }

    fn union_all(ranges: Vec<Range>) -> Vec<Range> {
        let mut sorted = Vec::new();
        for r in ranges.iter() {
            sorted.push(*r);
        }
        sorted.sort_by(|a, b| a.min_x.cmp(&b.min_x));

        let mut results = Vec::new();
        while sorted.len() > 0 {
            if sorted.len() >= 2 {
                let a = sorted[0];
                let b = sorted[1];
                if a.intersects(&b) {
                    sorted[0].max_x = std::cmp::max(a.max_x, b.max_x);
                    sorted.remove(1);
                } else {
                    results.push(a);
                    sorted.remove(0);
                }
            } else {
                results.push(sorted[0]);
                sorted.remove(0);
            }
        }
        results
    }
}

#[derive(Debug)]
struct Line {
    sensor: Point,
    closest_beacon: Point,
}

impl Line {
    fn radius_at_y(&self, y: i64) -> i64 {
        let distance = self.sensor.distance(&self.closest_beacon);
        let distance_to_y = (y - self.sensor.y).abs();
        if distance_to_y > distance {
            0
        } else {
            distance - distance_to_y
        }
    }

    fn range_at_y(&self, y: i64) -> Option<Range> {
        let radius = self.radius_at_y(y);
        if radius <= 0 {
            None
        } else {
            Some(Range {
                y: y,
                min_x: self.sensor.x - radius,
                max_x: self.sensor.x + radius,
            })
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut r = BufReader::new(io::stdin());
    do_it(&mut r, 2000000)?;
    Ok(())
}

fn do_it(r: &mut impl std::io::Read, test_at_y: i64) -> Result<i64, Box<dyn Error>> {
    let mut ranges = Vec::new();
    for line in BufReader::new(r).lines() {
        let line = parse_line(&line?)?;
        println!("line = {:?}", line);

        let range = line.range_at_y(test_at_y);
        println!("testing at y = {}, coverage = {:?}", test_at_y, range);

        if let Some(range) = range {
            ranges.push(range)
        }
    }

    let ranges = Range::union_all(ranges);
    println!("final ranges at y={}", test_at_y);
    for r in ranges.iter() {
        println!("{:?}", r);
    }

    let result: i64 = ranges.iter().map(|r| r.count()).sum();
    println!("total count of all ranges = {}", result);
    Ok(result)
}

fn parse_line(s: &str) -> Result<Line, Box<dyn Error>> {
    lazy_static! {
        static ref LINE_REGEX: regex::Regex = regex::Regex::new(
            "^Sensor at x=(-?[0-9]+), y=(-?[0-9]+): closest beacon is at x=(-?[0-9]+), y=(-?[0-9]+)$"
        )
        .unwrap();
    }
    let captures = LINE_REGEX
        .captures(s)
        .ok_or(format!("failed to parse line: {}", s))?;
    Ok(Line {
        sensor: Point {
            x: captures[1].parse()?,
            y: captures[2].parse()?,
        },
        closest_beacon: Point {
            x: captures[3].parse()?,
            y: captures[4].parse()?,
        },
    })
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::*;

    #[test]
    fn sample() -> Result<(), Box<dyn Error>> {
        assert_eq!(
            26,
            do_it(
                &mut r"Sensor at x=2, y=18: closest beacon is at x=-2, y=15
Sensor at x=9, y=16: closest beacon is at x=10, y=16
Sensor at x=13, y=2: closest beacon is at x=15, y=3
Sensor at x=12, y=14: closest beacon is at x=10, y=16
Sensor at x=10, y=20: closest beacon is at x=10, y=16
Sensor at x=14, y=17: closest beacon is at x=10, y=16
Sensor at x=8, y=7: closest beacon is at x=2, y=10
Sensor at x=2, y=0: closest beacon is at x=2, y=10
Sensor at x=0, y=11: closest beacon is at x=2, y=10
Sensor at x=20, y=14: closest beacon is at x=25, y=17
Sensor at x=17, y=20: closest beacon is at x=21, y=22
Sensor at x=16, y=7: closest beacon is at x=15, y=3
Sensor at x=14, y=3: closest beacon is at x=15, y=3
Sensor at x=20, y=1: closest beacon is at x=15, y=3"
                    .as_bytes(),
                10
            )?
        );
        Ok(())
    }
}
