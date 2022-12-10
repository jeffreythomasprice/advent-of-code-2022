use std::{
    error::Error,
    io::{self, BufRead, BufReader},
};

enum Instruction {
    Noop,
    Addx(i32),
}

#[derive(Debug)]
struct State {
    cycle: usize,
    register_x: i32,
}

impl State {
    fn new() -> Self {
        Self {
            cycle: 0,
            register_x: 1,
        }
    }

    fn execute(&mut self, instruction: Instruction, mut f: impl FnMut(&State)) {
        match instruction {
            Instruction::Noop => {
                self.cycle += 1;
                f(&self);
            }
            Instruction::Addx(delta) => {
                self.cycle += 1;
                f(&self);
                self.cycle += 1;
                f(&self);
                self.register_x += delta;
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut r = BufReader::new(io::stdin());
    do_it(&mut r)?;
    Ok(())
}

fn do_it(r: &mut impl std::io::Read) -> Result<String, Box<dyn Error>> {
    let noop_re = regex::Regex::new("^noop$")?;
    let addx_re = regex::Regex::new("^addx (-?[0-9]+)$")?;

    let state = &mut State::new();
    let mut display = String::new();

    let mut screen_x = 0;
    for line in BufReader::new(r).lines() {
        let line = line?;
        println!("line = {}", line);

        let instruction = if noop_re.is_match(&line) {
            Ok(Instruction::Noop)
        } else if let Some(captures) = addx_re.captures(&line) {
            Ok(Instruction::Addx(captures[1].parse()?))
        } else {
            Err(format!("no match: {}", line))
        }?;
        state.execute(instruction, |updated_state| {
            println!("{:?}", updated_state);
            let color = if (screen_x - updated_state.register_x).abs() <= 1 {
                '#'
            } else {
                '.'
            };
            display += color.to_string().as_str();
            if screen_x == 39 {
                display += "\n";
            }
            screen_x = (screen_x + 1) % 40;
        });
    }
    println!("final state = {:?}", state);
    println!("final display =\n{}", display);

    Ok(display)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample() -> Result<(), Box<dyn Error>> {
        assert_eq!(
            r"##..##..##..##..##..##..##..##..##..##..
###...###...###...###...###...###...###.
####....####....####....####....####....
#####.....#####.....#####.....#####.....
######......######......######......####
#######.......#######.......#######.....
",
            do_it(
                &mut r"addx 15
addx -11
addx 6
addx -3
addx 5
addx -1
addx -8
addx 13
addx 4
noop
addx -1
addx 5
addx -1
addx 5
addx -1
addx 5
addx -1
addx 5
addx -1
addx -35
addx 1
addx 24
addx -19
addx 1
addx 16
addx -11
noop
noop
addx 21
addx -15
noop
noop
addx -3
addx 9
addx 1
addx -3
addx 8
addx 1
addx 5
noop
noop
noop
noop
noop
addx -36
noop
addx 1
addx 7
noop
noop
noop
addx 2
addx 6
noop
noop
noop
noop
noop
addx 1
noop
noop
addx 7
addx 1
noop
addx -13
addx 13
addx 7
noop
addx 1
addx -33
noop
noop
noop
addx 2
noop
noop
noop
addx 8
noop
addx -1
addx 2
addx 1
noop
addx 17
addx -9
addx 1
addx 1
addx -3
addx 11
noop
noop
addx 1
noop
addx 1
noop
noop
addx -13
addx -19
addx 1
addx 3
addx 26
addx -30
addx 12
addx -1
addx 3
addx 1
noop
noop
noop
addx -9
addx 18
addx 1
addx 2
noop
noop
addx 9
noop
noop
noop
addx -1
addx 2
addx -37
addx 1
addx 3
noop
addx 15
addx -21
addx 22
addx -6
addx 1
noop
addx 2
addx 1
noop
addx -10
noop
noop
addx 20
addx 1
addx 2
addx 2
addx -6
addx -11
noop
noop
noop"
                    .as_bytes(),
            )?
        );
        Ok(())
    }
}
