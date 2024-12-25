use crate::error::{Error, NomError};
use crate::nom::{fold_separated_many0, single_line, single_line_not_eof, FinalParse};
use crate::part_solver;
use crate::utils::ures;
use derive_more::{Deref, DerefMut, From, Into};
use nom::branch::alt;
use nom::bytes::complete::take_while_m_n;
use nom::character::complete::space0;
use nom::multi::fold_many_m_n;
use nom::{IResult, Parser};
use nom_supreme::tag::complete::tag;
use nom_supreme::ParserExt;

part_solver!();

pub fn part1(input: &str) -> Result<ures, Error> {
    let (locks, keys) = parse_input.final_parse(input)?;
    let mut count = 0;
    for lock in locks {
        for key in keys.iter() {
            if key.is_match(&lock) {
                count += 1;
            }
        }
    }

    Ok(count)
}

pub fn part2(_input: &str) -> Result<ures, Error> {
    Ok(0)
}

fn parse_key_or_lock_line(input: &str) -> IResult<&str, &str, NomError> {
    single_line_not_eof(take_while_m_n(5, 5, |c: char| c == '.' || c == '#')).parse(input)
}

#[derive(Copy, Clone, From, Into, Hash, Eq, PartialEq, Deref, DerefMut)]
struct Lock([u8; 5]);
#[derive(Copy, Clone, From, Into, Hash, Eq, PartialEq, Deref, DerefMut)]
struct Key([u8; 5]);
#[derive(Copy, Clone, From, Hash, Eq, PartialEq)]
enum LockOrKey {
    Lock(Lock),
    Key(Key),
}

impl Key {
    fn is_match(&self, lock: &Lock) -> bool {
        let lock = lock.deref();
        let key = self.deref();
        for i in 0..5 {
            if lock[i] + key[i] > 5 {
                return false;
            }
        }

        true
    }
}

fn parse_lock(input: &str) -> IResult<&str, LockOrKey, NomError> {
    single_line_not_eof(tag("#####"))
        .precedes(fold_many_m_n(
            5,
            5,
            parse_key_or_lock_line,
            || [0; 5],
            |mut res, s| {
                s.as_bytes()
                    .iter()
                    .enumerate()
                    .filter(|&(_, &v)| v == b'#')
                    .for_each(|(i, _)| res[i] += 1);

                res
            },
        ))
        .map(Lock::from)
        .map(LockOrKey::from)
        .terminated(single_line(tag(".....")))
        .parse(input)
}

fn parse_key(input: &str) -> IResult<&str, LockOrKey, NomError> {
    single_line_not_eof(tag("....."))
        .precedes(fold_many_m_n(
            5,
            5,
            parse_key_or_lock_line,
            || [0; 5],
            |mut res, s| {
                s.as_bytes()
                    .iter()
                    .enumerate()
                    .filter(|&(_, &v)| v == b'#')
                    .for_each(|(i, _)| res[i] += 1);

                res
            },
        ))
        .map(Key::from)
        .map(LockOrKey::from)
        .terminated(single_line(tag("#####")))
        .parse(input)
}

fn parse_input(input: &str) -> IResult<&str, (Vec<Lock>, Vec<Key>), NomError> {
    fold_separated_many0(
        single_line_not_eof(space0),
        alt((parse_lock, parse_key)),
        || (Vec::new(), Vec::new()),
        |(mut locks, mut keys), lock_or_key| {
            match lock_or_key {
                LockOrKey::Lock(l) => locks.push(l),
                LockOrKey::Key(k) => keys.push(k),
            }

            (locks, keys)
        },
    )
    .parse(input)
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::utils::tests_utils::{get_input, human_text_duration};
    use chrono::Utc;

    #[test]
    pub fn part1() -> Result<(), Error> {
        let start = Utc::now();
        let input = get_input(2024, 25)?;
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

    #[ignore]
    #[test]
    pub fn part2() -> Result<(), Error> {
        let start = Utc::now();
        let input = get_input(2024, 25)?;
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
