use crate::error::Error;
use crate::part_solver;
use crate::space::space2d::{BitBoard2d, Board2d, RefBoard2d};
use crate::space::{IterSpace, Space};
use crate::utils::{cardinal, ures};
use std::borrow::Cow;

part_solver!();

pub fn part1(input: &str) -> Result<ures, Error> {
    let board = parse_input(input);
    let res = board
        .iter()
        .filter(|(_pos, &val)| val == b'0')
        .map(|(pos, &val)| {
            find_path_count(
                &board,
                &mut BitBoard2d::<usize>::with_height(board.height()),
                &pos,
                val,
            )
        })
        .sum();

    Ok(res)
}

pub fn part2(input: &str) -> Result<ures, Error> {
    let board = parse_input(input);
    let mut cache: Board2d<Option<ures>> = (0..board.height())
        .map(|row| {
            board
                .width(row)
                .ok_or_else(|| {
                    Error::InvalidState("board width should be valid for row 0 to height".into())
                })
                .map(|len| vec![None; len])
        })
        .try_fold(Vec::with_capacity(board.height()), |mut vec, val| {
            val.map(|v| {
                vec.push(v);
                vec
            })
        })?
        .into();
    let res = board
        .iter()
        .filter(|(_pos, &val)| val == b'0')
        .map(|(pos, &val)| find_path_count_2(&board, &mut cache, &pos, val))
        .sum();

    Ok(res)
}

fn parse_input(input: &str) -> RefBoard2d<u8> {
    input
        .lines()
        .map(|line| Cow::Borrowed(line.as_bytes()))
        .collect::<Vec<_>>()
        .into()
}

fn find_path_count(
    board: &RefBoard2d<u8>,
    visited_set: &mut BitBoard2d,
    pos: &[usize; 2],
    val: u8,
) -> ures {
    match visited_set.get(pos) {
        None => return 0,
        Some(visited) => {
            if *visited {
                return 0;
            }
        }
    }

    visited_set.set(pos, true);

    if val == b'9' {
        return 1;
    }

    let valid_next_val = val + 1;

    cardinal(pos)
        .filter_map(|pos| {
            board
                .get(&pos)
                .filter(|next_val| **next_val == valid_next_val)
                .map(|val| find_path_count(board, visited_set, &pos, *val))
        })
        .sum::<ures>()
}

fn find_path_count_2(
    board: &RefBoard2d<u8>,
    cache: &mut Board2d<Option<ures>>,
    pos: &[usize; 2],
    val: u8,
) -> ures {
    match cache.get(pos) {
        None => return 0,
        Some(cache_val) => {
            if let &Some(count) = cache_val {
                return count;
            }
        }
    }

    if val == b'9' {
        return 1;
    }

    let valid_next_val = val + 1;
    cardinal(pos)
        .filter_map(|pos| {
            board
                .get(&pos)
                .filter(|next_val| **next_val == valid_next_val)
                .map(|val| find_path_count_2(board, cache, &pos, *val))
        })
        .sum::<ures>()
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::utils::tests_utils::{get_input, human_text_duration};
    use chrono::Utc;

    #[test]
    pub fn part1() -> Result<(), Error> {
        let start = Utc::now();
        let input = get_input(2024, 10)?;
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
        let input = get_input(2024, 10)?;
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
