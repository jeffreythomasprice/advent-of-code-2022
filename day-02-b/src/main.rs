use regex::Regex;
use std::io::{self, BufRead};

#[derive(Debug)]
enum Choice {
    Rock,
    Paper,
    Scissors,
}

#[derive(Debug, PartialEq)]
enum GameOutcome {
    Win,
    Lose,
    Draw,
}

impl Choice {
    fn score(&self) -> i32 {
        match self {
            Choice::Rock => 1,
            Choice::Paper => 2,
            Choice::Scissors => 3,
        }
    }

    fn compare(&self, other: &Choice) -> GameOutcome {
        match self {
            Choice::Rock => match other {
                Choice::Rock => GameOutcome::Draw,
                Choice::Paper => GameOutcome::Lose,
                Choice::Scissors => GameOutcome::Win,
            },
            Choice::Paper => match other {
                Choice::Rock => GameOutcome::Win,
                Choice::Paper => GameOutcome::Draw,
                Choice::Scissors => GameOutcome::Lose,
            },
            Choice::Scissors => match other {
                Choice::Rock => GameOutcome::Lose,
                Choice::Paper => GameOutcome::Win,
                Choice::Scissors => GameOutcome::Draw,
            },
        }
    }
}

impl GameOutcome {
    fn score(&self) -> i32 {
        match self {
            GameOutcome::Win => 6,
            GameOutcome::Draw => 3,
            GameOutcome::Lose => 0,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let re = Regex::new("^([ABC]) ([XYZ])$")?;
    let mut total = 0;
    for line in io::BufReader::new(io::stdin()).lines() {
        match line {
            Ok(s) => {
                let captures = re
                    .captures(&s)
                    .ok_or_else(|| format!("line failed parsing: {}", s))?;
                let opponent_choice = match &captures[1] {
                    "A" => Ok(Choice::Rock),
                    "B" => Ok(Choice::Paper),
                    "C" => Ok(Choice::Scissors),
                    _ => Err(format!("unhandled opponent choice: {}", s)),
                }?;
                let desired_result = match &captures[2] {
                    "X" => Ok(GameOutcome::Lose),
                    "Y" => Ok(GameOutcome::Draw),
                    "Z" => Ok(GameOutcome::Win),
                    _ => Err(format!("unhandled my choice: {}", s)),
                }?;
                let my_choice = [Choice::Rock, Choice::Paper, Choice::Scissors]
                    .iter()
                    .find(|x| x.compare(&opponent_choice) == desired_result)
                    .unwrap();
                let round_result = my_choice.compare(&opponent_choice);
                let round_score = my_choice.score() + round_result.score();
                total += round_score;
                println!(
                    "line: opponent={:?}, desired outcome={:?}, me={:?}, my choice score = {}, result = {:?}, total score = {}",
                    opponent_choice,
                    desired_result,
                    my_choice,
                    my_choice.score(),
                    round_result,
                    round_score
                );
                Ok(())
            }
            Err(e) => Err(format!("line read error {e}")),
        }?;
    }
    println!("final score = {}", total);
    Ok(())
}
