use std::{
    error::Error,
    io::{self, BufRead, BufReader},
};

#[derive(Debug, Clone, Copy, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Debug, Clone, Copy)]
struct Line {
    a: Point,
    b: Point,
}

#[derive(Debug, Clone, Copy)]
struct BoundingRectangle {
    min: Point,
    max: Point,
}

impl BoundingRectangle {
    fn new<I>(points: I) -> Result<BoundingRectangle, String>
    where
        I: Iterator<Item = Point>,
    {
        let mut min = None;
        let mut max = None;
        for p in points {
            min = match min {
                None => Some(p),
                Some(min) => Some(Point {
                    x: std::cmp::min(min.x, p.x),
                    y: std::cmp::min(min.y, p.y),
                }),
            };
            max = match max {
                None => Some(p),
                Some(max) => Some(Point {
                    x: std::cmp::max(max.x, p.x),
                    y: std::cmp::max(max.y, p.y),
                }),
            };
        }
        Ok(BoundingRectangle {
            min: min.ok_or("expected at least one point")?,
            max: max.ok_or("expected at least one point")?,
        })
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut r = BufReader::new(io::stdin());
    do_it(&mut r)?;
    Ok(())
}

fn do_it(r: &mut impl std::io::Read) -> Result<usize, Box<dyn Error>> {
    let lines = BufReader::new(r)
        .lines()
        .collect::<Result<Vec<_>, _>>()?
        .iter()
        .map(|line| parse_line(line))
        .collect::<Result<Vec<_>, _>>()?;
    let lines = lines
        .iter()
        .flatten()
        .map(|line| line.clone())
        .collect::<Vec<_>>();
    for line in lines.iter() {
        println!("line = {:?}", line);
    }

    let bounds = {
        let mut result =
            BoundingRectangle::new(lines.iter().flat_map(|line| vec![line.a, line.b]))?;
        result.min.y = 0;
        result
    };
    println!("bounds = {:?}", bounds);

    let mut sand_at_rest = Vec::new();
    let mut sand_in_motion = None;

    let mut tick = 0;
    loop {
        // generate new sand
        if sand_in_motion.is_none() {
            sand_in_motion = Some(Point { x: 500, y: 0 });
        }

        // look at the locations we can move sand to and find the next location, or none if no new location is possible
        let mut new_location = None;
        for proposed_new_point in [
            Point {
                x: sand_in_motion.unwrap().x,
                y: sand_in_motion.unwrap().y + 1,
            },
            Point {
                x: sand_in_motion.unwrap().x - 1,
                y: sand_in_motion.unwrap().y + 1,
            },
            Point {
                x: sand_in_motion.unwrap().x + 1,
                y: sand_in_motion.unwrap().y + 1,
            },
        ] {
            if !point_collide_environment(&proposed_new_point, lines.iter(), sand_at_rest.iter())? {
                new_location = Some(proposed_new_point);
                break;
            }
        }

        // either we moved the sand, or we can't and it stops here
        if let Some(new_location) = new_location {
            // check against the bounds
            if bounds_contains_point(&bounds, &new_location) {
                // still in bounds, we can move here
                sand_in_motion = Some(new_location);
            } else {
                // out of bounds, we're done
                break;
            }
        } else {
            // no new location found, we're done with this one
            sand_at_rest.push(sand_in_motion.unwrap());
            sand_in_motion = None;
        }

        if tick % 100 == 0 {
            println!("tick = {}", tick);
            draw_environment(&bounds, sand_in_motion, lines.iter(), sand_at_rest.iter());
            println!("");
        }
        tick += 1;
    }

    println!("final environment");
    println!("tick = {}", tick);
    draw_environment(&bounds, sand_in_motion, lines.iter(), sand_at_rest.iter());
    println!("");

    println!("total sand dropped = {}", sand_at_rest.len());
    Ok(sand_at_rest.len())
}

fn parse_line(s: &str) -> Result<Vec<Line>, Box<dyn Error>> {
    let points = s
        .split("->")
        .map(parse_point)
        .collect::<Result<Vec<_>, _>>()?;
    if points.len() < 2 {
        Err(format!("expected at least two points, got {}", s))?
    }
    let mut results = Vec::new();
    for i in 0..(points.len() - 1) {
        results.push(Line {
            a: points[i],
            b: points[i + 1],
        });
    }
    Ok(results)
}

fn parse_point(s: &str) -> Result<Point, Box<dyn Error>> {
    let parts = s.trim().split(",").collect::<Vec<_>>();
    if parts.len() == 2 {
        Ok(Point {
            x: parts[0].parse()?,
            y: parts[1].parse()?,
        })
    } else {
        Err(format!(
            "expected exactly two components separated by a comma: {}",
            s
        ))?
    }
}

fn draw_environment<'a, LineI, PointI>(
    bounds: &BoundingRectangle,
    point: Option<Point>,
    lines: LineI,
    other_points: PointI,
) where
    LineI: Iterator<Item = &'a Line>,
    PointI: Iterator<Item = &'a Point>,
{
    let width = bounds.max.x - bounds.min.x + 1;
    let height = bounds.max.y - bounds.min.y + 1;
    let mut data = Vec::new();
    data.resize((width * height) as usize, '.');
    for line in lines {
        let mut p = line.a;
        while p != line.b {
            data[((p.x - bounds.min.x) + (p.y - bounds.min.y) * width) as usize] = '#';
            if p.x < line.b.x {
                p.x += 1;
            } else if p.x > line.b.x {
                p.x -= 1;
            }
            if p.y < line.b.y {
                p.y += 1;
            } else if p.y > line.b.y {
                p.y -= 1;
            }
        }
    }
    for p in other_points {
        data[((p.x - bounds.min.x) + (p.y - bounds.min.y) * width) as usize] = 'o';
    }
    if let Some(point) = point {
        data[((point.x - bounds.min.x) + (point.y - bounds.min.y) * width) as usize] = 'O';
    }
    for y in 0..height {
        for x in 0..width {
            print!("{}", data[(x + y * width) as usize]);
        }
        println!("");
    }
}

fn point_collide_environment<'a, LineI, PointI>(
    point: &Point,
    lines: LineI,
    other_points: PointI,
) -> Result<bool, String>
where
    LineI: Iterator<Item = &'a Line>,
    PointI: Iterator<Item = &'a Point>,
{
    for line in lines {
        let result = point_collide_line(point, &line)?;
        if result {
            return Ok(true);
        }
    }
    for other_point in other_points {
        let result = point_collide_point(point, &other_point);
        if result {
            return Ok(true);
        }
    }
    Ok(false)
}

fn point_collide_point(point: &Point, other: &Point) -> bool {
    point.x == other.x && point.y == other.y
}

fn point_collide_line(point: &Point, line: &Line) -> Result<bool, String> {
    if line.a.x == line.b.x {
        let min = std::cmp::min(line.a.y, line.b.y);
        let max = std::cmp::max(line.a.y, line.b.y);
        Ok(point.x == line.a.x && point.y >= min && point.y <= max)
    } else if line.a.y == line.b.y {
        let min = std::cmp::min(line.a.x, line.b.x);
        let max = std::cmp::max(line.a.x, line.b.x);
        Ok(point.y == line.a.y && point.x >= min && point.x <= max)
    } else {
        Err(format!(
            "line is not perfectly horizontal or vertical: {:?}",
            line
        ))
    }
}

fn bounds_contains_point(bounds: &BoundingRectangle, point: &Point) -> bool {
    point.x >= bounds.min.x
        && point.y <= bounds.max.x
        && point.y >= bounds.min.y
        && point.y <= bounds.max.y
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::*;

    #[test]
    fn sample() -> Result<(), Box<dyn Error>> {
        assert_eq!(
            24,
            do_it(
                &mut r"498,4 -> 498,6 -> 496,6
503,4 -> 502,4 -> 502,9 -> 494,9"
                    .as_bytes(),
            )?
        );
        Ok(())
    }
}
