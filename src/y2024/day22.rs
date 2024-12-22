use crate::error::{Error, NomError};
use crate::nom::{single_line, ures, FinalParse};
use crate::part_solver;
use crate::utils::ures;
use nom::multi::many1;
use nom::{IResult, Parser};

part_solver!();

pub fn part1(input: &str) -> Result<ures, Error> {
    let res = parse_input
        .final_parse(input)?
        .into_iter()
        .map(|v| get_secret(v, 2000))
        .sum();

    Ok(res)
}

pub fn part2(input: &str) -> Result<ures, Error> {
    let mut bananas = vec![[[[0; 19]; 19]; 19]; 19];

    parse_input
        .final_parse(input)?
        .into_iter()
        .for_each(|v| compute_banana(v, 2000, bananas.as_mut_slice()));

    bananas
        .into_iter()
        .flatten()
        .flatten()
        .flatten()
        .max()
        .ok_or_else(|| Error::InvalidState("bananas is empty????".into()))
}

fn parse_input(input: &str) -> IResult<&str, Vec<ures>, NomError> {
    many1(single_line(ures)).parse(input)
}

fn gen_next_secret(secret: ures) -> ures {
    let secret = ((secret << 6) ^ secret) & 16777215;
    let secret = ((secret >> 5) ^ secret) & 16777215;
    
    ((secret << 11) ^ secret) & 16777215
}

fn get_secret(mut secret: ures, times: ures) -> ures {
    for _ in 0..times {
        secret = gen_next_secret(secret);
    }
    secret
}

fn compute_banana(
    mut secret: ures,
    times: ures,
    four_diffs_to_banana: &mut [[[[ures; 19]; 19]; 19]],
) {
    let mut diff_index_1;
    let mut diff_index_2 = 0;
    let mut diff_index_3 = 0;
    let mut diff_index_4 = 0;
    let mut prev_last_digit = 0;
    let mut visited = vec![[[[false; 19]; 19]; 19]; 19];
    for i in 0..times {
        diff_index_1 = diff_index_2;
        diff_index_2 = diff_index_3;
        diff_index_3 = diff_index_4;
        secret = gen_next_secret(secret);
        let next_last_digit = secret % 10;
        diff_index_4 = if next_last_digit >= prev_last_digit {
            next_last_digit - prev_last_digit
        } else {
            19 - prev_last_digit + next_last_digit
        };
        prev_last_digit = next_last_digit;
        if i >= 3 && !visited[diff_index_1][diff_index_2][diff_index_3][diff_index_4] {
            visited[diff_index_1][diff_index_2][diff_index_3][diff_index_4] = true;
            four_diffs_to_banana[diff_index_1][diff_index_2][diff_index_3][diff_index_4] +=
                next_last_digit;
        }
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
        let input = get_input(2024, 22)?;
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
        let input = get_input(2024, 22)?;
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
