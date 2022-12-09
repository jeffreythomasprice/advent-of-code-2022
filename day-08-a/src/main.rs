use std::{
    cmp::{max, min},
    error::Error,
    fmt,
    io::{self, BufRead, BufReader},
};

#[derive(Copy, Clone)]
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn new(x: i32, y: i32) -> Point {
        Point { x, y }
    }

    fn x(&self) -> i32 {
        self.x
    }

    fn y(&self) -> i32 {
        self.y
    }
}

impl fmt::Debug for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[derive(Copy, Clone)]
struct Size {
    width: i32,
    height: i32,
}

impl Size {
    fn new(width: i32, height: i32) -> Size {
        Size { width, height }
    }

    fn width(self) -> i32 {
        self.width
    }

    fn height(self) -> i32 {
        self.height
    }
}

impl fmt::Debug for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({} x {})", self.width, self.height)
    }
}

struct Map {
    size: Size,
    data: Vec<i32>,
}

impl Map {
    fn new() -> Map {
        Map {
            size: Size::new(0, 0),
            data: Vec::new(),
        }
    }

    fn size(&self) -> Size {
        self.size
    }

    fn get_at(&self, p: Point) -> Result<i32, String> {
        let size = self.size();
        if p.x() < 0 || p.y() < 0 || p.x() >= size.width() || p.y() >= size.height() {
            Err(format!("out of bounds {:?}, size = {:?}", p, self))
        } else {
            Ok(self.data[(p.x() + p.y() * size.width()) as usize])
        }
    }

    fn set_at(&mut self, p: Point, value: i32) -> Result<(), String> {
        let size = self.size();
        if p.x() < 0 || p.y() < 0 || p.x() >= size.width() || p.y() >= size.height() {
            Err(format!("out of bounds {:?}, size = {:?}", p, self.size))
        } else {
            self.data[(p.x() + p.y() * size.width()) as usize] = value;
            Ok(())
        }
    }

    fn set_size(&mut self, size: Size) -> Result<(), String> {
        if size.width() < 0 || size.height() < 0 {
            Err(format!("size must be positive {:?}", size))?
        }
        if size.width() == 0 || size.height() == 0 {
            self.size = Size::new(0, 0);
            self.data.clear();
        } else {
            let mut new_data = Vec::new();
            let copy_width = min(self.size().width(), size.width());
            let extra_width = if size.width() > copy_width {
                size.width() - copy_width
            } else {
                0
            };
            let copy_height = min(self.size().height(), size.height());
            let extra_height = if size.height() > copy_height {
                size.height() - copy_height
            } else {
                0
            };
            for y in 0..copy_height {
                for x in 0..copy_width {
                    new_data.push(self.get_at(Point::new(x, y))?);
                }
                for _ in 0..extra_width {
                    new_data.push(0);
                }
            }
            for _ in 0..extra_height {
                for _ in 0..size.width() {
                    new_data.push(0);
                }
            }
            self.size = size;
            self.data = new_data;
        }
        Ok(())
    }

    fn grow(&mut self, size: Size) -> Result<(), String> {
        let desired_width = max(self.size().width(), size.width());
        let desired_height = max(self.size().height(), size.height());
        if desired_width > self.size().width() || desired_height > self.size().height() {
            self.set_size(Size::new(desired_width, desired_height))?;
        }
        Ok(())
    }

    fn set_and_grow(&mut self, p: Point, value: i32) -> Result<(), String> {
        self.grow(Size::new(p.x() + 1, p.y() + 1))?;
        self.set_at(p, value)?;
        Ok(())
    }
}

impl fmt::Debug for Map {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data = self
            .data
            .chunks(self.size().width() as usize)
            .map(|row| {
                row.iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join("")
            })
            .collect::<Vec<String>>()
            .join("\n");
        write!(f, "Map({:?}\n{})", self.size, data)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut r = BufReader::new(io::stdin());
    do_it(&mut r)?;
    Ok(())
}

fn do_it(r: &mut impl std::io::Read) -> Result<i32, Box<dyn Error>> {
    let mut map = Map::new();
    let mut y = 0;
    for line in BufReader::new(r).lines() {
        let line = line?;
        let mut x = 0;
        for c in line.chars() {
            let value = c
                .to_digit(10)
                .ok_or(format!("not a digit at {}, {}, c = {}", x, y, c))?
                as i32;
            map.set_and_grow(Point::new(x, y), value)?;
            x += 1;
        }
        y += 1;
    }
    println!("map = {:?}", map);

    let mut count = 0;
    for y in 0..map.size().height() {
        for x in 0..map.size().width() {
            let p = Point::new(x, y);
            let v = is_visible(&map, p)?;
            println!("{:?} is visible? {}", p, v);
            if v {
                count += 1;
            }
        }
    }
    println!("count = {}", count);
    Ok(count)
}

fn is_visible(map: &Map, p: Point) -> Result<bool, Box<dyn Error>> {
    let value = map.get_at(p)?;
    if p.x() == 0
        || p.y() == 0
        || p.x() == map.size().width() - 1
        || p.y() == map.size().height() - 1
    {
        Ok(true)
    } else {
        for r in [
            (0..p.x())
                .map(|x| Point::new(x, p.y()))
                .collect::<Vec<_>>()
                .iter(),
            ((p.x() + 1)..map.size().width())
                .map(|x| Point::new(x, p.y()))
                .collect::<Vec<_>>()
                .iter(),
            (0..p.y())
                .map(|y| Point::new(p.x(), y))
                .collect::<Vec<_>>()
                .iter(),
            ((p.y() + 1)..map.size().height())
                .map(|y| Point::new(p.x(), y))
                .collect::<Vec<_>>()
                .iter(),
        ] {
            if r.map(|p| map.get_at(*p))
                .collect::<Result<Vec<_>, _>>()?
                .iter()
                .all(|other_value| *other_value < value)
            {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::error::Error;

    #[test]
    fn sample() -> Result<(), Box<dyn Error>> {
        assert_eq!(
            21,
            do_it(
                &mut r"30373
25512
65332
33549
35390"
                    .as_bytes(),
            )?
        );
        Ok(())
    }
}
