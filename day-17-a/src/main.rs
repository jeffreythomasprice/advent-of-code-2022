use std::{
    collections::HashSet,
    error::Error,
    fmt::Debug,
    io::{self, BufReader},
    rc::Rc,
};

#[derive(Debug, Clone, Copy)]
enum Direction {
    Left,
    Right,
}

#[derive(Clone, Copy)]
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    fn add(&self, other: Point) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }

    fn subtract(&self, other: Point) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Debug for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[derive(Clone, Copy)]
struct Size {
    width: i32,
    height: i32,
}

impl Size {
    fn new(width: i32, height: i32) -> Self {
        Self { width, height }
    }
}

impl Debug for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} x {})", self.width, self.height)
    }
}

#[derive(Clone, Copy)]
struct Rectangle {
    min: Point,
    max: Point,
}

impl Rectangle {
    fn new_with_min_max(min: Point, max: Point) -> Rectangle {
        Self {
            min: Point::new(std::cmp::min(min.x, max.x), std::cmp::min(min.y, max.y)),
            max: Point::new(std::cmp::max(min.x, max.x), std::cmp::max(min.y, max.y)),
        }
    }

    fn new_with_points<'a, I>(mut points: I) -> Result<Self, String>
    where
        I: Iterator<Item = &'a Point>,
    {
        let min = points.next();
        if min.is_none() {
            Err("must provide at least one point")?;
        }
        let mut min = *min.unwrap();
        let mut max = min;
        for p in points {
            min.x = std::cmp::min(min.x, p.x);
            min.y = std::cmp::min(min.y, p.y);
            max.x = std::cmp::max(max.x, p.x);
            max.y = std::cmp::max(max.y, p.y);
        }
        Ok(Self { min, max })
    }

    fn size(&self) -> Size {
        Size {
            width: self.max.x - self.min.x + 1,
            height: self.max.y - self.min.y + 1,
        }
    }

    fn offset(&self, offset: Point) -> Rectangle {
        Rectangle::new_with_min_max(self.min.add(offset), self.max.add(offset))
    }

    fn intersects(&self, other: &Rectangle) -> bool {
        !(other.min.x > self.max.x
            || other.max.x < self.min.x
            || other.min.y > self.max.y
            || other.max.y < self.min.y)
    }
}

impl Debug for Rectangle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(min={:?}, max={:?}, size={:?})",
            self.min,
            self.max,
            self.size()
        )
    }
}

struct Shape {
    size: Size,
    data: Vec<Vec<bool>>,
}

impl Shape {
    fn new(input: &str) -> Result<Self, String> {
        let data = input
            .trim()
            .split("\n")
            .map(|line| {
                line.trim()
                    .chars()
                    .map(|c| match c {
                        '#' => Ok(true),
                        '.' => Ok(false),
                        _ => Err(format!("unrecognized character: {}", c)),
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
            .collect::<Result<Vec<_>, _>>()?;
        let widths = data.iter().map(|line| line.len()).collect::<HashSet<_>>();
        if widths.len() != 1 {
            Err("not all lines are the same length")?;
        }
        Ok(Self {
            size: Size::new(*widths.iter().next().unwrap() as i32, data.len() as i32),
            data,
        })
    }

    fn bounds(&self) -> Rectangle {
        Rectangle::new_with_min_max(
            Point::new(0, 0),
            Point::new(self.size.width - 1, self.size.height - 1),
        )
    }

    fn contains(&self, point: &Point) -> bool {
        if point.x < 0 || point.x >= self.size.width || point.y < 0 || point.y >= self.size.height {
            false
        } else {
            // y is flipped, 0 is on the bottom for everything else but the shape is given in input top-down
            self.data[(self.size.height - point.y - 1) as usize][point.x as usize]
        }
    }
}

impl Debug for Shape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "shape(size = {:?}\n{})",
            self.size,
            self.data
                .iter()
                .map(|line| {
                    line.iter()
                        .map(|value| if *value { "#" } else { "." })
                        .collect::<Vec<_>>()
                        .join("")
                })
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

#[derive(Debug)]
struct PlacedShape<'a> {
    shape: &'a Shape,
    location: Point,
}

impl<'a> PlacedShape<'a> {
    fn new(shape: &'a Shape, location: Point) -> Self {
        Self { shape, location }
    }

    fn bounds(&self) -> Rectangle {
        self.shape.bounds().offset(self.location)
    }

    fn contains(&self, point: &Point) -> bool {
        self.shape.contains(&point.subtract(self.location))
    }

    fn intersects(&self, other: &PlacedShape) -> Result<bool, String> {
        if !self.bounds().intersects(&other.bounds()) {
            return Ok(false);
        }
        let combined_bounds = Rectangle::new_with_points(
            vec![
                self.bounds().min,
                self.bounds().max,
                other.bounds().min,
                other.bounds().max,
            ]
            .iter(),
        )?;
        for x in combined_bounds.min.x..=combined_bounds.max.x {
            for y in combined_bounds.min.y..=combined_bounds.max.y {
                let p = Point::new(x, y);
                if self.contains(&p) && other.contains(&p) {
                    return Ok(true);
                }
            }
        }
        return Ok(false);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut r = BufReader::new(io::stdin());
    do_it(&mut r)?;
    Ok(())
}

fn do_it(r: &mut impl std::io::Read) -> Result<i32, Box<dyn Error>> {
    let shapes = vec![
        Shape::new(
            r"
            ####
            ",
        )?,
        Shape::new(
            r"
            .#.
            ###
            .#.
            ",
        )?,
        Shape::new(
            r"
            ..#
            ..#
            ###
            ",
        )?,
        Shape::new(
            r"
            #
            #
            #
            #
            ",
        )?,
        Shape::new(
            r"
            ##
            ##
            ",
        )?,
    ];
    for shape in shapes.iter() {
        println!("possible shape\n{:?}", shape);
    }
    println!("");

    let mut input = String::new();
    r.read_to_string(&mut input)?;

    let input = input
        .trim()
        .chars()
        .map(|c| match c {
            '<' => Ok(Direction::Left),
            '>' => Ok(Direction::Right),
            _ => Err(format!("not a valid characte: {}", c)),
        })
        .collect::<Result<Vec<_>, _>>()?;
    println!("input = {:?}", input);
    println!("");

    let mut placed_shapes: Vec<Rc<PlacedShape>> = Vec::new();
    let mut placed_shapes_bounds: Option<Rectangle> = None;
    let mut current_shape: Option<Rc<PlacedShape>> = None;
    let mut direction_index = 0;
    let mut shape_index = 0;
    while placed_shapes.len() < 2022 {
        let mut cs = if let Some(cs) = &current_shape {
            cs.clone()
        } else {
            let shape = &shapes[shape_index];
            println!("spawning shape {}, {:?}", shape_index, shape);
            shape_index = (shape_index + 1) % shapes.len();
            let y = if let Some(bounds) = placed_shapes_bounds {
                bounds.max.y + 1
            } else {
                0
            };
            let result = Rc::new(PlacedShape::new(shape, Point::new(2, y + 3)));
            current_shape = Some(result.clone());
            result
        };

        let direction = input[direction_index];
        direction_index = (direction_index + 1) % input.len();
        println!("direction = {:?}", direction);

        for (offset, can_place_here) in [
            (
                match direction {
                    Direction::Left => Point::new(-1, 0),
                    Direction::Right => Point::new(1, 0),
                },
                false,
            ),
            (Point::new(0, -1), true),
        ] {
            let proposed_cs_bounds = cs.bounds().offset(offset);
            println!("moving {:?} to {:?}", offset, proposed_cs_bounds);
            let collides = if proposed_cs_bounds.min.x < 0 {
                println!("hit left wall");
                true
            } else if proposed_cs_bounds.max.x > 6 {
                println!("hit right wall");
                true
            } else if proposed_cs_bounds.min.y < 0 {
                println!("hit floor");
                true
            } else {
                // can fall some, check against the existing shapes
                // stop here (i.e true) when we hit anything trying to move down
                let mut found_one = false;
                // only check existing shape intersections if we're inside the global bounding box
                if let Some(psb) = placed_shapes_bounds {
                    let proposed = PlacedShape::new(cs.shape, cs.location.add(offset));
                    if psb.intersects(&proposed.bounds()) {
                        for ps in placed_shapes.iter() {
                            if proposed.intersects(ps)? {
                                found_one = true;
                                println!("hit another shape");
                                break;
                            }
                        }
                    }
                }
                found_one
            };

            if collides {
                if can_place_here {
                    println!("placing shape {:?}", cs);
                    placed_shapes_bounds = if let Some(psb) = placed_shapes_bounds {
                        Some(Rectangle::new_with_points(
                            vec![psb.min, psb.max, cs.bounds().min, cs.bounds().max].iter(),
                        )?)
                    } else {
                        Some(cs.bounds())
                    };
                    placed_shapes.push(cs.clone());
                    current_shape = None;
                    println!("there are now {} shapes", placed_shapes.len());
                    println!(
                        "new total placed bounds = {:?}",
                        placed_shapes_bounds.unwrap()
                    );
                    println!("");
                    break;
                } else {
                    println!(
                        "not moving because hit something, but can't place place on this move"
                    );
                }
            } else {
                let new_shape = Rc::new(PlacedShape::new(cs.shape, cs.location.add(offset)));
                cs = new_shape.clone();
                println!("moving to {:?}", new_shape.bounds());
                current_shape = Some(new_shape);
            }
        }
    }
    println!("");

    let result = placed_shapes_bounds.unwrap().size().height;
    println!("total height = {}", result);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::*;

    #[test]
    fn sample() -> Result<(), Box<dyn Error>> {
        assert_eq!(
            3068,
            do_it(&mut r">>><<><>><<<>><>>><<<>>><<<><<<>><>><<>>".as_bytes())?
        );
        Ok(())
    }
}
