use crate::error::{Error, NomError};
use crate::nom::{fold_separated_list0, single_line, single_line_not_eof, trim_space, FinalParse};
use crate::part_solver;
use crate::utils::ures;
use gcd::Gcd;
use nom::character::complete::{space0, u64};
use nom::sequence::{pair, tuple};
use nom::{IResult, Parser};
use nom_supreme::tag::complete::tag;
use nom_supreme::ParserExt;
use std::num::NonZeroU64;

part_solver!();

pub fn part1(input: &str) -> Result<ures, Error> {
    let res = solve_input_parser(prize_parser).final_parse(input)?;
    Ok(res)
}

pub fn part2(input: &str) -> Result<ures, Error> {
    let res = solve_input_parser(prize_parser_2).final_parse(input)?;
    Ok(res)
}

fn non_zero_u64(input: &str) -> IResult<&str, NonZeroU64, NomError<'_>> {
    u64.map_res(|v| {
        NonZeroU64::new(v).ok_or_else(|| Error::ParseError("got 0 for non-zero".into()))
    })
    .context("parse non-zero 64")
    .parse(input)
}

fn button_a_parser(input: &str) -> IResult<&str, (u64, u64), NomError> {
    pair(
        tag("Button A: X+")
            .precedes(non_zero_u64)
            .map(NonZeroU64::get),
        tag(", Y+").precedes(non_zero_u64).map(NonZeroU64::get),
    )
    .context("parse A line")
    .parse(input)
}

fn button_b_parser(input: &str) -> IResult<&str, (u64, u64), NomError> {
    pair(
        tag("Button B: X+")
            .precedes(non_zero_u64)
            .map(NonZeroU64::get),
        tag(", Y+").precedes(non_zero_u64).map(NonZeroU64::get),
    )
    .context("parse B line")
    .parse(input)
}

fn prize_parser(input: &str) -> IResult<&str, (u64, u64), NomError> {
    pair(
        tag("Prize: X=").precedes(non_zero_u64).map(NonZeroU64::get),
        tag(", Y=").precedes(non_zero_u64).map(NonZeroU64::get),
    )
    .context("parse prize line")
    .parse(input)
}

fn prize_parser_2(input: &str) -> IResult<&str, (u64, u64), NomError<'_>> {
    prize_parser
        .map(|(c1, c2)| (c1 + 10000000000000, c2 + 10000000000000))
        .parse(input)
}

fn solve_block_parser<'a, F>(
    prize_parser: F,
) -> impl Parser<&'a str, Option<(u64, u64)>, NomError<'a>>
where
    F: Parser<&'a str, (u64, u64), NomError<'a>> + 'a,
{
    tuple((
        single_line_not_eof(trim_space(button_a_parser)),
        single_line_not_eof(trim_space(button_b_parser)),
        single_line(trim_space(prize_parser)),
    ))
    .cut()
    .context("parse block")
    .map_res_cut(|((a1, a2), (b1, b2), (c1, c2))| solve_day13_int_matrix(a1, a2, b1, b2, c1, c2))
    .context("solving matrix")
}

fn solve_input_parser<'a, F>(prize_parser: F) -> impl Parser<&'a str, ures, NomError<'a>>
where
    F: Parser<&'a str, (u64, u64), NomError<'a>> + 'a,
{
    fold_separated_list0(
        single_line_not_eof(space0),
        solve_block_parser(prize_parser),
        || 0,
        |mut sum, val| {
            if let Some((a, b)) = val {
                sum += a as ures * 3 + b as ures;
            }

            sum
        },
    )
}

fn solve_day13_int_matrix(
    a1: u64,
    a2: u64,
    b1: u64,
    b2: u64,
    c1: u64,
    c2: u64,
) -> Result<Option<(u64, u64)>, Error> {
    let gcd = a1.gcd(a2);
    let mult_1 = a2 / gcd;
    let mult_2 = a1 / gcd;
    let multiplied_b1 = b1
        .checked_mul(mult_1)
        .ok_or_else(|| Error::Unsolvable("overflow while matching coefficient".into()))?;
    let multiplied_b2 = b2
        .checked_mul(mult_2)
        .ok_or_else(|| Error::Unsolvable("overflow while matching coefficient".into()))?;
    let multiplied_c1 = c1
        .checked_mul(mult_1)
        .ok_or_else(|| Error::Unsolvable("overflow while matching coefficient".into()))?;
    let multiplied_c2 = c2
        .checked_mul(mult_2)
        .ok_or_else(|| Error::Unsolvable("overflow while matching coefficient".into()))?;
    if multiplied_b1 == multiplied_b2 {
        if multiplied_c1 == multiplied_c2 {
            return Err(Error::Unsolvable("infinite solution".into()));
        } else {
            return Ok(None);
        }
    }

    let b;

    if multiplied_c1 == multiplied_c2 {
        b = 0;
    } else {
        let ord = multiplied_b2.cmp(&multiplied_b1);
        if ord != multiplied_c2.cmp(&multiplied_c1) {
            return Ok(None);
        }

        let c_diff = multiplied_c1.abs_diff(multiplied_c2);
        let b_diff = multiplied_b1.abs_diff(multiplied_b2);
        if c_diff % b_diff != 0 {
            return Ok(None);
        } else {
            b = c_diff / b_diff
        }
    }

    let res = b1
        .checked_mul(b)
        .and_then(|b1_final| c1.checked_sub(b1_final))
        .filter(|c1_final| *c1_final % a1 == 0)
        .map(|c1_final| c1_final / a1)
        .map(|a| (a, b));

    Ok(res)
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::utils::tests_utils::{get_input, human_text_duration};
    use chrono::Utc;

    #[test]
    pub fn part1() -> Result<(), Error> {
        let start = Utc::now();
        let input = get_input(2024, 13)?;
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
        let input = get_input(2024, 13)?;
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
