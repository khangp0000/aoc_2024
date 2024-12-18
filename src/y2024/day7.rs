use crate::error::Error;
use crate::part_solver;
use crate::utils::ures;
use std::str::FromStr;

part_solver!();

pub fn part1(input: &str) -> Result<ures, Error> {
    input
        .lines()
        .map(parse_line)
        .filter_map(|res| {
            res.and_then(|(target, vec)| check_1(target, &vec))
                .transpose()
        })
        .try_fold(0, |mut sum, val| {
            val.map(|val| {
                sum += val;
                sum
            })
        })
}

pub fn part2(input: &str) -> Result<ures, Error> {
    input
        .lines()
        .map(parse_line)
        .filter_map(|res| {
            res.and_then(|(target, vec)| check_2(target, &vec))
                .transpose()
        })
        .try_fold(0, |mut sum, val| {
            val.map(|val| {
                sum += val;
                sum
            })
        })
}

fn parse_line(line: &str) -> Result<(ures, Vec<ures>), Error> {
    let (left, right) = line.split_once(":").ok_or_else(|| {
        Error::ParseError(format!("failed to parse {:?}: no 3 spaces delimiter", line).into())
    })?;

    let target_val = ures::from_str(left)
        .map_err(|e| Error::ParseError(format!("failed to parse: {:?}: {}", line, e).into()))?;

    let vec = right
        .trim()
        .split(" ")
        .map(|val| {
            ures::from_str(val).map_err(|e| {
                Error::ParseError(format!("failed to parse: {:?}: {}", line, e).into())
            })
        })
        .try_fold(Vec::new(), |mut vec, val| {
            val.map(|val| {
                vec.push(val);
                vec
            })
        })?;
    Ok((target_val, vec))
}

fn check_1(target_val: ures, vec: &[ures]) -> Result<Option<ures>, Error> {
    let &first = vec
        .first()
        .ok_or_else(|| Error::ParseError("right hand side of colon missing".into()))?;
    if check_1_inner(target_val, first, vec[1..].iter()) {
        Ok(Some(target_val))
    } else {
        Ok(None)
    }
}

fn check_1_inner<
    'a,
    I: Iterator<Item = &'a ures> + Clone,
    T: IntoIterator<Item = &'a ures, IntoIter = I>,
>(
    target: ures,
    current: ures,
    iter: T,
) -> bool {
    if current > target {
        return false;
    }
    let mut iter = iter.into_iter();
    if let Some(val) = iter.next() {
        check_1_inner(target, current + *val, iter.clone())
            || check_1_inner(target, current * *val, iter)
    } else {
        current == target
    }
}

fn check_2(target_val: ures, vec: &[ures]) -> Result<Option<ures>, Error> {
    let &first = vec
        .first()
        .ok_or_else(|| Error::ParseError("right hand side of colon missing".into()))?;
    if check_2_inner(target_val, first, vec[1..].iter()) {
        Ok(Some(target_val))
    } else {
        Ok(None)
    }
}
fn check_2_inner<
    'a,
    I: Iterator<Item = &'a ures> + Clone,
    T: IntoIterator<Item = &'a ures, IntoIter = I>,
>(
    target: ures,
    current: ures,
    iter: T,
) -> bool {
    if current > target {
        return false;
    }
    let mut iter = iter.into_iter();
    if let Some(val) = iter.next() {
        check_2_inner(target, current + *val, iter.clone())
            || check_2_inner(target, current * *val, iter.clone())
            || check_2_inner(target, concat(current, *val), iter)
    } else {
        current == target
    }
}

fn concat(a: ures, b: ures) -> ures {
    if b == 0 {
        a * 10
    } else {
        a * (10 as ures).pow(b.ilog10() + 1) + b
    }
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::utils::tests_utils::{get_input, human_text_duration};
    use chrono::Utc;

    #[test]
    pub fn part1() -> Result<(), Error> {
        let start = Utc::now();
        let input = get_input(2024, 7)?;
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
        let input = get_input(2024, 7)?;
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
