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
    pub fn new<I>(points: I) -> Result<BoundingRectangle, String>
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

impl BoundingRectangle {
    fn contains_point(&self, p: &Point) -> bool {
        p.x >= self.min.x && p.x <= self.max.x && p.y >= self.min.y && p.y <= self.max.y
    }
}

struct Grid<T> {
    bounds: BoundingRectangle,
    width: usize,
    height: usize,
    data: Vec<T>,
}

impl<T> Grid<T>
where
    T: Copy,
{
    pub fn new(bounds: &BoundingRectangle, initial_value: T) -> Self {
        let mut result = Grid {
            bounds: bounds.clone(),
            width: (bounds.max.x - bounds.min.x + 1) as usize,
            height: (bounds.max.y - bounds.min.y + 1) as usize,
            data: Vec::new(),
        };
        result
            .data
            .resize(result.width * result.height, initial_value);
        result
    }

    pub fn get(&self, p: Point) -> Result<T, Box<dyn Error>> {
        if self.bounds.contains_point(&p) {
            Ok(self.data[(p.x - self.bounds.min.x) as usize
                + (p.y - self.bounds.min.y) as usize * self.width])
        } else {
            Err(format!(
                "out of bounds: {:?}, bounds = {:?}",
                p, self.bounds
            ))?
        }
    }

    pub fn set(&mut self, p: Point, value: T) -> Result<(), Box<dyn Error>> {
        if self.bounds.contains_point(&p) {
            self.data[(p.x - self.bounds.min.x) as usize
                + (p.y - self.bounds.min.y) as usize * self.width] = value;
            Ok(())
        } else {
            Err(format!(
                "out of bounds: {:?}, bounds = {:?}",
                p, self.bounds
            ))?
        }
    }

    pub fn set_line(&mut self, line: &Line, value: T) -> Result<(), Box<dyn Error>> {
        let mut p = line.a;
        while p != line.b {
            self.set(p, value)?;
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
        self.set(p, value)?;
        Ok(())
    }
}

impl Grid<char> {
    pub fn draw(&self) -> Result<(), Box<dyn Error>> {
        for y in self.bounds.min.y..self.bounds.max.y {
            for x in self.bounds.min.x..self.bounds.max.x {
                print!("{}", self.get(Point { x, y })?);
            }
            println!("");
        }
        Ok(())
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

    let lines = {
        let mut result = lines;
        let height = bounds.max.y - bounds.min.y;
        result.push(Line {
            a: Point {
                x: bounds.min.x - height,
                y: bounds.max.y + 2,
            },
            b: Point {
                x: bounds.max.x + height,
                y: bounds.max.y + 2,
            },
        });
        println!("added new infinite line at y = {}", bounds.max.y);
        result
    };

    let bounds = {
        let mut result = bounds;
        let height = bounds.max.y - bounds.min.y;
        result.min.x -= height;
        result.max.x += height;
        result.max.y += 3;
        result
    };
    println!("adjusted bounds = {:?}", bounds);

    let mut grid = Grid::new(&bounds, '.');
    for line in lines.iter() {
        grid.set_line(&line, '#')?;
    }

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
            // nothing at that location yet
            if grid.get(proposed_new_point)? == '.' {
                new_location = Some(proposed_new_point);
                break;
            }
        }

        // either we moved the sand, or we can't and it stops here
        if let Some(new_location) = new_location {
            // check against the bounds
            if bounds.contains_point(&new_location) {
                // still in bounds, we can move here
                sand_in_motion = Some(new_location);
            } else {
                // out of bounds, error
                Err("shouldn't reach this")?;
            }
        } else {
            // no new location found, we're done with this one
            sand_at_rest.push(sand_in_motion.unwrap());
            grid.set(sand_in_motion.unwrap(), 'o')?;
            if sand_in_motion.unwrap() == (Point { x: 500, y: 0 }) {
                // we've filled up to the origin point so done
                break;
            }
            sand_in_motion = None;
        }

        if tick % 1000 == 0 {
            println!("tick = {}, total so far = {}", tick, sand_at_rest.len());
            grid.draw()?;
            println!("");
        }
        tick += 1;
    }

    println!("final environment");
    println!("tick = {}", tick);
    grid.draw()?;
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

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::*;

    #[test]
    fn sample() -> Result<(), Box<dyn Error>> {
        assert_eq!(
            93,
            do_it(
                &mut r"498,4 -> 498,6 -> 496,6
503,4 -> 502,4 -> 502,9 -> 494,9"
                    .as_bytes(),
            )?
        );
        Ok(())
    }
}
