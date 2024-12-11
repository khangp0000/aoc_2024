use crate::error::Error;
use crate::part_solver;
use crate::utils::ures;
use std::str::FromStr;

part_solver!();

pub fn part1(input: &str) -> Result<ures, Error> {
    let mut cache = vec![[None; 25]; 100];
    parse_input(input.trim())
        .map(|v| v.and_then(|v| blink_count_with_cache(v, 25, &mut cache)))
        .try_fold(0, |sum, res| res.map(|v| v + sum))
}

pub fn part2(input: &str) -> Result<ures, Error> {
    let mut cache = vec![[None; 75]; 1000];
    parse_input(input.trim())
        .map(|v| v.and_then(|v| blink_count_with_cache(v, 75, &mut cache)))
        .try_fold(0, |sum, res| res.map(|v| v + sum))
}

fn parse_input<'a>(input: &'a str) -> impl Iterator<Item = Result<u64, Error>> + use<'a> {
    input.split(" ").map(|s| {
        u64::from_str(s).map_err(|e| {
            Error::ParseError(format!("Failed to parse unsigned integer: {:?}: {}", s, e))
        })
    })
}

fn blink(val: u64) -> Result<Vec<u64>, Error> {
    let res = if val == 0 {
        vec![1]
    } else {
        let count_digit_minus_1 = val.ilog10();
        if count_digit_minus_1 % 2 == 1 {
            let mult = 10u64.pow((count_digit_minus_1 + 1) / 2);
            vec![val / mult, val % mult]
        } else {
            vec![val.checked_mul(2024).ok_or_else(|| {
                Error::InvalidState("number is too large, not fitting in u64".to_string())
            })?]
        }
    };

    Ok(res)
}

fn blink_count_with_cache<const N: usize>(
    val: u64,
    target_count: u8,
    cache: &mut Vec<[Option<ures>; N]>,
) -> Result<ures, Error> {
    if target_count == 0 {
        Ok(1)
    } else {
        if let Some(Some(count)) = cache
            .get(val as usize)
            .and_then(|v| v.get(target_count as usize))
        {
            return Ok(*count);
        }
        let mut sum = 0;
        for val in blink(val)?.into_iter() {
            sum += blink_count_with_cache(val, target_count - 1, cache)?;
        }

        cache.get_mut(val as usize).and_then(|inner_array| {
            inner_array
                .get_mut(target_count as usize)
                .map(|count| count.replace(sum))
        });
        Ok(sum)
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
        let input = get_input(2024, 11)?;
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
        let input = get_input(2024, 11)?;
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
