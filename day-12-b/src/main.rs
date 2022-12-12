use std::{
    cmp::{max, min},
    error::Error,
    fmt::Debug,
    io::{self, BufRead, BufReader},
};

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Point {
    x: u32,
    y: u32,
}

impl Point {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }

    pub fn x(&self) -> u32 {
        self.x
    }

    pub fn y(&self) -> u32 {
        self.y
    }

    pub fn distance(&self, other: Point) -> u32 {
        let xd = if self.x > other.x {
            self.x - other.x
        } else {
            other.x - self.x
        };
        let yd = if self.y > other.y {
            self.y - other.y
        } else {
            other.y - self.y
        };
        return xd + yd;
    }
}

impl Debug for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Size {
    width: u32,
    height: u32,
}

impl Size {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}

impl Debug for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} x {})", self.width, self.height)
    }
}

pub struct Grid<T> {
    size: Size,
    data: Vec<T>,
}

impl<T> Grid<T>
where
    T: Copy,
{
    pub fn new() -> Self {
        Self {
            size: Size::new(0, 0),
            data: Vec::new(),
        }
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn set_size(&mut self, size: Size, default_value: T) {
        if size.width != self.size.width || size.height != self.size.height {
            let mut new_data = Vec::new();
            new_data.reserve((size.width * size.height) as usize);
            let min_width = min(size.width, self.size.width);
            let max_width = max(size.width, self.size.height);
            let min_height = min(size.height, self.size.height);
            let max_height = max(size.height, self.size.height);
            for y in 0..min_height {
                for x in 0..min_width {
                    new_data.push(self.data[(x + y * self.size.width) as usize]);
                }
                for _ in min_width..max_width {
                    new_data.push(default_value);
                }
            }
            for _ in min_height..max_height {
                for _ in 0..size.width {
                    new_data.push(default_value);
                }
            }
            self.data = new_data;
            self.size = size;
        }
    }

    pub fn grow(&mut self, size: Size, default_value: T) {
        self.set_size(
            Size::new(
                max(self.size.width, size.width),
                max(self.size.height, size.height),
            ),
            default_value,
        )
    }

    pub fn get(&self, p: Point) -> Result<T, Box<dyn Error>> {
        if p.x < self.size.width && p.y < self.size.height {
            Ok(self.data[(p.x + p.y * self.size.width) as usize])
        } else {
            Err(format!(
                "point out of bounds {:?}, size = {:?}",
                p, self.size
            ))?
        }
    }

    pub fn set(&mut self, p: Point, value: T) -> Result<(), Box<dyn Error>> {
        if p.x < self.size.width && p.y < self.size.height {
            self.data[(p.x + p.y * self.size.width) as usize] = value;
            Ok(())
        } else {
            Err(format!(
                "point out of bounds {:?}, size = {:?}",
                p, self.size
            ))?
        }
    }

    pub fn get_neighbors(&self, p: Point) -> Vec<Point> {
        let mut results = Vec::new();
        results.reserve(4);
        if p.x >= 1 {
            results.push(Point::new(p.x() - 1, p.y()));
        }
        if p.x + 1 < self.size().width() {
            results.push(Point::new(p.x() + 1, p.y()));
        }
        if p.y >= 1 {
            results.push(Point::new(p.x(), p.y() - 1));
        }
        if p.y + 1 < self.size().height() {
            results.push(Point::new(p.x(), p.y() + 1));
        }
        results
    }
}

#[derive(Debug, Clone, Copy)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Clone, Copy)]
enum Solution {
    Goal,
    Direction(Direction, u32),
}

impl Solution {
    fn score(&self) -> u32 {
        match self {
            Solution::Goal => 0,
            Solution::Direction(_, score) => *score,
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Location {
    p: Point,
    height: u32,
    solution: Option<Solution>,
}

impl Location {
    pub fn new(p: Point, height: u32) -> Self {
        Self {
            p,
            height,
            solution: None,
        }
    }

    pub fn location(&self) -> Point {
        self.p
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn can_go_to(&self, other: &Location) -> bool {
        if self.height + 1 >= other.height {
            if self.p.x() + 1 == other.p.x() && self.p.y() == other.p.y() {
                true
            } else if other.p.x() + 1 == self.p.x() && self.p.y() == other.p.y() {
                true
            } else if self.p.x() == other.p.x() && self.p.y() + 1 == other.p.y() {
                true
            } else if self.p.x() == other.p.x() && other.p.y() + 1 == self.p.y() {
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut r = BufReader::new(io::stdin());
    do_it(&mut r)?;
    Ok(())
}

fn do_it(r: &mut impl std::io::Read) -> Result<usize, Box<dyn Error>> {
    let mut heightmap = Grid::new();
    let mut start = None;
    let mut goal = None;
    {
        let mut y = 0;
        for line in BufReader::new(r).lines() {
            let line = line?;
            let mut x = 0;
            for c in line.chars() {
                let p = Point::new(x, y);
                heightmap.grow(Size::new(x + 1, y + 1), Location::new(Point::new(0, 0), 0));
                match c {
                    'S' => {
                        heightmap.set(p, Location::new(p, 0))?;
                        start = Some(Point::new(x, y));
                    }
                    'E' => {
                        let mut location = Location::new(p, 25);
                        location.solution = Some(Solution::Goal);
                        heightmap.set(p, location)?;
                        goal = Some(Point::new(x, y));
                    }
                    'a'..='z' => {
                        heightmap.set(p, Location::new(p, c as u32 - 'a' as u32))?;
                    }
                    _ => Err(format!("unrecognized char: {}", c))?,
                };
                x += 1;
            }
            y += 1;
        }
    }
    let start = start.ok_or("no start")?;
    let goal = goal.ok_or("no goal")?;

    println!("heightmap size = {:?}", heightmap.size());
    for y in 0..heightmap.size().height() {
        for x in 0..heightmap.size().width() {
            let value = heightmap.get(Point::new(x, y))?;
            let c = (value.height as u8 + 'a' as u8) as char;
            print!("{}", c);
        }
        println!("");
    }
    println!("start = {:?}", start);
    println!("goal = {:?}", goal);
    println!("");

    let find_locations_that_can_reach =
        |heightmap: &Grid<Location>, goal: Point| -> Vec<(Direction, Point)> {
            [
                if goal.x() >= 1 {
                    Some((Direction::Right, Point::new(goal.x() - 1, goal.y())))
                } else {
                    None
                },
                if goal.x() + 1 < heightmap.size().width() {
                    Some((Direction::Left, Point::new(goal.x() + 1, goal.y())))
                } else {
                    None
                },
                if goal.y() >= 1 {
                    Some((Direction::Down, Point::new(goal.x(), goal.y() - 1)))
                } else {
                    None
                },
                if goal.y() + 1 < heightmap.size().height() {
                    Some((Direction::Up, Point::new(goal.x(), goal.y() + 1)))
                } else {
                    None
                },
            ]
            .iter()
            .filter(|x| x.is_some())
            .map(|x| x.unwrap())
            .filter(|(_, location)| {
                let location = heightmap.get(*location);
                let goal_location = heightmap.get(goal);
                if location.is_err() || goal_location.is_err() {
                    false
                } else {
                    location.unwrap().can_go_to(&goal_location.unwrap())
                }
            })
            .collect::<Vec<_>>()
        };

    // the places we can search are those locations that can go to the goal
    let mut locations_to_check = vec![goal];

    // while there are locations to search, go fill in one
    while !locations_to_check.is_empty() {
        let location = locations_to_check.pop().unwrap();
        let location = heightmap.get(location)?;
        let location_score = location.solution.unwrap().score();
        for (direction, other) in find_locations_that_can_reach(&heightmap, location.location()) {
            let mut other = heightmap.get(other)?;
            // if we accept this direction
            let proposed_other_score = location_score + 1;
            if match other.solution {
                Some(other_solution) => proposed_other_score < other_solution.score(),
                None => true,
            } {
                other.solution = Some(Solution::Direction(direction, proposed_other_score));
                heightmap.set(other.location(), other)?;
                locations_to_check.push(other.location());
            }
        }
    }

    for y in 0..heightmap.size().height() {
        for x in 0..heightmap.size().width() {
            let location = heightmap.get(Point::new(x, y))?;
            let c = match location.solution {
                Some(Solution::Direction(direction, _)) => match direction {
                    Direction::Left => '<',
                    Direction::Right => '>',
                    Direction::Up => '^',
                    Direction::Down => 'v',
                },
                Some(Solution::Goal) => 'E',
                None => '.',
            };
            print!("{}", c);
        }
        println!("");
    }

    let real_start = {
        let mut result = None;
        for y in 0..heightmap.size().height() {
            for x in 0..heightmap.size().width() {
                let location = heightmap.get(Point::new(x, y))?;
                if location.height() == 0 {
                    result = match result {
                        None => Some(location),
                        Some(current_best) => {
                            if current_best.solution.is_some() && location.solution.is_some() {
                                if current_best.solution.unwrap().score()
                                    < location.solution.unwrap().score()
                                {
                                    Some(current_best)
                                } else {
                                    Some(location)
                                }
                            } else {
                                Some(current_best)
                            }
                        }
                    };
                }
            }
        }
        result.unwrap()
    };
    println!("real start = {:?}", real_start);
    Ok(real_start.solution.unwrap().score().try_into().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample() -> Result<(), Box<dyn Error>> {
        assert_eq!(
            29,
            do_it(
                &mut r"Sabqponm
abcryxxl
accszExk
acctuvwj
abdefghi"
                    .as_bytes(),
            )?
        );
        Ok(())
    }
}
