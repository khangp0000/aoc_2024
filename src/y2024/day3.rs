use crate::error::Error;
use crate::part_solver;
use crate::utils::ures;
use regex::{Captures, Regex};
use std::borrow::Borrow;
use std::str::FromStr;
use std::sync::OnceLock;

part_solver!();

pub fn part1(input: &str) -> Result<ures, Error> {
    static RE: OnceLock<Result<Regex, Error>> = OnceLock::new();
    let regex = RE
        .get_or_init(|| {
            Regex::new(r"mul\((\d+),(\d+)\)").map_err(|e| {
                Error::InitError(format!(
                    "failed to init regex `{}`: {}",
                    r"mul\((\d+),(\d+)\)", e
                ))
            })
        })
        .as_ref()
        .map_err(Clone::clone)?;

    regex
        .captures_iter(input)
        .map(mul_capture)
        .try_fold(0, |sum, mul| mul.map(|v| sum + v))
}

pub fn part2(mut input: &str) -> Result<ures, Error> {
    let mut sum = 0;
    while !input.is_empty() {
        let end = input.find("don't()").unwrap_or(input.len());
        sum += part1(&input[..end])?;
        let end = end + 7;
        if end >= input.len() {
            break;
        }
        input = &input[end..];
        let start = input.find("do()").map(|i| i + 4).unwrap_or(input.len());
        input = &input[start..];
    }
    Ok(sum)
}

fn mul_capture<'a, C: Borrow<Captures<'a>>>(captures: C) -> Result<ures, Error> {
    let val1 = captures
        .borrow()
        .get(1)
        .ok_or_else(|| Error::InvalidState("Capture should have group 1".to_string()))?
        .as_str();
    let val2 = captures
        .borrow()
        .get(2)
        .ok_or_else(|| Error::InvalidState("Capture should have group 2".to_string()))?
        .as_str();
    let val1 = ures::from_str(val1)
        .map_err(|e| Error::ParseError(format!("Failed to parse {:?}: {}", val1, e)))?;
    let val2 = ures::from_str(val2)
        .map_err(|e| Error::ParseError(format!("Failed to parse {:?}: {}", val2, e)))?;
    Ok(val1 * val2)
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::utils::tests_utils::{get_input, human_text_duration};
    use chrono::Utc;

    #[test]
    pub fn part1() -> Result<(), Error> {
        let start = Utc::now();
        let input = get_input(2024, 3)?;
        let input_finish = Utc::now();
        let res = super::part1(input.as_str())?;
        let run_finish = Utc::now();
        println!("Result: {}", res);
        println!(
            "Input runtime: {}",
            human_text_duration(input_finish - start)
        );
        println!(
            "Solve runtime: {}",
            human_text_duration(run_finish - input_finish)
        );
        println!("Total runtime: {}", human_text_duration(run_finish - start));
        Ok(())
    }

    #[test]
    pub fn part2() -> Result<(), Error> {
        let start = Utc::now();
        let input = get_input(2024, 3)?;
        let input_finish = Utc::now();
        let res = super::part2(input.as_str())?;
        let run_finish = Utc::now();
        println!("Result: {}", res);
        println!(
            "Input runtime: {}",
            human_text_duration(input_finish - start)
        );
        println!(
            "Solve runtime: {}",
            human_text_duration(run_finish - input_finish)
        );
        println!("Total runtime: {}", human_text_duration(run_finish - start));
        Ok(())
    }
}
