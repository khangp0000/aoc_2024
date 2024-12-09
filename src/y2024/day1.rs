use crate::error::Error;
use crate::part_solver;
use crate::utils::ures;
use std::collections::{BinaryHeap, HashMap};
use std::str::FromStr;

part_solver!();
pub fn part1(input: &str) -> Result<ures, Error> {
    let (mut heap1, mut heap2) = input.lines().map(parse_line).try_fold(
        (BinaryHeap::new(), BinaryHeap::new()),
        |(mut heap1, mut heap2), line_pair| {
            line_pair.map(|(left, right)| {
                heap1.push(left);
                heap2.push(right);
                (heap1, heap2)
            })
        },
    )?;

    let mut sum = 0;
    while !heap1.is_empty() {
        let left = heap1
            .pop()
            .ok_or_else(|| Error::InvalidState("heap1 should not be empty".to_string()))?;
        let right = heap2
            .pop()
            .ok_or_else(|| Error::InvalidState("heap2 should not be empty".to_string()))?;
        sum += left.abs_diff(right);
    }

    Ok(sum)
}

pub fn part2(input: &str) -> Result<ures, Error> {
    let freqs =
        input
            .lines()
            .map(parse_line)
            .try_fold(HashMap::new(), |mut freqs, line_pair| {
                line_pair.map(|(left, right)| {
                    freqs
                        .entry(left)
                        .and_modify(|(left_freq, _)| *left_freq += 1)
                        .or_insert((1, 0));
                    freqs
                        .entry(right)
                        .and_modify(|(_, right_freq)| *right_freq += 1)
                        .or_insert((0, 1));
                    freqs
                })
            })?;

    let sum = freqs
        .into_iter()
        .fold(0, |sum, (value, (left_freq, right_freq))| {
            sum + (value * left_freq * right_freq)
        });

    Ok(sum)
}

fn parse_line(line: &str) -> Result<(ures, ures), Error> {
    let (left, right) = line.split_once("   ").ok_or_else(|| {
        Error::ParseError(format!("Failed to parse {:?}: no 3 spaces delimiter", line))
    })?;
    let left = ures::from_str(left)
        .map_err(|e| Error::ParseError(format!("Failed to parse: {:?}: {}", line, e)))?;
    let right = ures::from_str(right)
        .map_err(|e| Error::ParseError(format!("Failed to parse: {:?}: {}", line, e)))?;
    Ok((left, right))
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::utils::tests_utils::get_input;
    use chrono::Utc;

    #[test]
    pub fn part1() -> Result<(), Error> {
        let input = get_input(2024, 1)?;
        let start = Utc::now();
        println!("Result: {}", super::part1(input.as_str())?);
        let duration = Utc::now() - start;
        println!("Runtime: {}", duration);
        Ok(())
    }

    #[test]
    pub fn part2() -> Result<(), Error> {
        let input = get_input(2024, 1)?;
        let start = Utc::now();
        println!("Result: {}", super::part2(input.as_str())?);
        let duration = Utc::now() - start;
        println!("Runtime: {}", duration);
        Ok(())
    }
}
