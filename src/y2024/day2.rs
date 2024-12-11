use crate::error::Error;
use crate::part_solver;
use crate::utils::{ires, ures};
use std::borrow::Borrow;
use std::str::FromStr;

part_solver!();

pub fn part1(input: &str) -> Result<ures, Error> {
    input.lines().map(parse_line).try_fold(0, |count, vec| {
        vec.map(|vec| {
            if verify_valid_1(&vec) {
                count + 1
            } else {
                count
            }
        })
    })
}

pub fn part2(input: &str) -> Result<ures, Error> {
    input.lines().map(parse_line).try_fold(0, |count, vec| {
        vec.map(|vec| {
            if verify_valid_2(&vec) {
                count + 1
            } else {
                count
            }
        })
    })
}

fn parse_line(line: &str) -> Result<Vec<ires>, Error> {
    line.split(' ')
        .map(|num| {
            ires::from_str(num)
                .map_err(|e| Error::ParseError(format!("Failed to parse {:?}: {}", line, e)))
        })
        .try_fold(Vec::new(), |mut vec, val| {
            val.map(|val| {
                vec.push(val);
                vec
            })
        })
}

fn verify_valid_1(line: &[ires]) -> bool {
    let mut iter = line.windows(2).map(|pair| pair[1] - pair[0]);

    if let Some(first_val) = iter.next() {
        if valid_inc(first_val) {
            iter.all(valid_inc)
        } else if valid_dec(first_val) {
            iter.all(valid_dec)
        } else {
            false
        }
    } else {
        true
    }
}

fn verify_valid_2(line: &[ires]) -> bool {
    if line.len() <= 2 {
        return true;
    }

    let diffs: Vec<_> = line.windows(2).map(|pair| pair[1] - pair[0]).collect();

    let mut valid = false;
    let not_inc_pos = diffs.iter().position(|v| !valid_inc(*v));

    if let Some(not_inc_pos) = not_inc_pos {
        valid = valid || (not_inc_pos == diffs.len() - 1);
        valid = valid
            || ((not_inc_pos == 0 || valid_inc(diffs[not_inc_pos] + diffs[not_inc_pos - 1]))
                && diffs[not_inc_pos + 1..].iter().all(valid_inc));
        valid = valid
            || (valid_inc(diffs[not_inc_pos] + diffs[not_inc_pos + 1])
                && diffs[not_inc_pos + 2..].iter().all(valid_inc));

        if valid {
            return valid;
        }

        let not_dec_pos = diffs.iter().position(|v| !valid_dec(*v));
        if let Some(not_dec_pos) = not_dec_pos {
            valid = valid || (not_dec_pos == diffs.len() - 1);
            valid = valid
                || ((not_dec_pos == 0 || valid_dec(diffs[not_dec_pos] + diffs[not_dec_pos - 1]))
                    && diffs[not_dec_pos + 1..].iter().all(valid_dec));
            valid = valid
                || (valid_dec(diffs[not_dec_pos] + diffs[not_dec_pos + 1])
                    && diffs[not_dec_pos + 2..].iter().all(valid_dec));
        } else {
            valid = true;
        }
    } else {
        valid = true
    }

    valid
}

fn valid_inc<T: Borrow<ires>>(val: T) -> bool {
    *val.borrow() >= 1 && *val.borrow() <= 3
}

fn valid_dec<T: Borrow<ires>>(val: T) -> bool {
    *val.borrow() >= -3 && *val.borrow() <= -1
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::utils::tests_utils::{get_input, human_text_duration};
    use chrono::Utc;

    #[test]
    pub fn part1() -> Result<(), Error> {
        let start = Utc::now();
        let input = get_input(2024, 2)?;
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
        let input = get_input(2024, 2)?;
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
