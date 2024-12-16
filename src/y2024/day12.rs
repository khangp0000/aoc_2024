use crate::error::Error;
use crate::part_solver;
use crate::space::space2d::{BitBoard2d, RefBoard2d};
use crate::space::{IterSpace, Pos, Space};
use crate::utils::{cardinal, ures};
use std::borrow::Cow;

part_solver!();

type Area = ures;
type Circumference = ures;

pub fn part1(input: &str) -> Result<ures, Error> {
    let board = parse_input(input);
    let visited_set = &mut BitBoard2d::<usize>::with_height(board.height());
    let mut cost = 0;
    for (pos, val) in board.iter() {
        let (area, circumference) = area_circumference(&board, visited_set, &pos, val)?;
        cost += area * circumference;
    }
    Ok(cost)
}

pub fn part2(input: &str) -> Result<ures, Error> {
    let board = parse_input(input);
    let visited_set = &mut BitBoard2d::<usize>::with_height(board.height());
    let mut cost = 0;
    for (pos, val) in board.iter() {
        let (area, side) = area_side(&board, visited_set, &pos, val)?;
        cost += area * side;
    }
    Ok(cost)
}

fn parse_input(input: &str) -> RefBoard2d<u8> {
    input
        .lines()
        .map(|line| Cow::Borrowed(line.as_bytes()))
        .collect::<Vec<_>>()
        .into()
}

fn count_corner(board: &RefBoard2d<u8>, pos: &[usize; 2]) -> Result<ures, Error> {
    let val = board
        .get(pos)
        .ok_or_else(|| Error::InvalidState("board out of bound".into()))?;
    let same_top = shift_equal(board, pos, &[0, -1], val);
    let same_right = shift_equal(board, pos, &[1, 0], val);
    let same_bottom = shift_equal(board, pos, &[0, 1], val);
    let same_left = shift_equal(board, pos, &[-1, 0], val);

    let mut corner_count = 0;

    if (!same_top && !same_left)
        || (same_top && same_left && !shift_equal(board, pos, &[-1, -1], val))
    {
        corner_count += 1;
    }

    if (!same_top && !same_right)
        || (same_top && same_right && !shift_equal(board, pos, &[1, -1], val))
    {
        corner_count += 1;
    }

    if (!same_bottom && !same_left)
        || (same_bottom && same_left && !shift_equal(board, pos, &[-1, 1], val))
    {
        corner_count += 1;
    }

    if (!same_bottom && !same_right)
        || (same_bottom && same_right && !shift_equal(board, pos, &[1, 1], val))
    {
        corner_count += 1;
    }

    Ok(corner_count)
}

#[inline]
fn shift_equal(board: &RefBoard2d<u8>, pos: &[usize; 2], diff: &[isize; 2], val: &u8) -> bool {
    pos.shift(diff).and_then(|pos| board.get(&pos)) == Some(val)
}

fn area_circumference(
    board: &RefBoard2d<u8>,
    visited_set: &mut BitBoard2d,
    pos: &[usize; 2],
    val: &u8,
) -> Result<(Area, Circumference), Error> {
    if Some(&true) == visited_set.get(pos) {
        Ok((0, 0))
    } else {
        visited_set
            .set(pos, true)
            .ok_or_else(|| Error::InvalidState("visited set board out of bound".into()))?;
        cardinal(pos)
            .filter_map(|next_pos| {
                board
                    .get(&next_pos)
                    .filter(|next_val| *next_val == val)
                    .map(|next_val| area_circumference(board, visited_set, &next_pos, next_val))
            })
            .try_fold((1, 4), |(area, circumference), res| {
                let (next_area, next_circumference) = res?;
                Ok((area + next_area, circumference + next_circumference - 1))
            })
    }
}

fn area_side(
    board: &RefBoard2d<u8>,
    visited_set: &mut BitBoard2d,
    pos: &[usize; 2],
    val: &u8,
) -> Result<(Area, Circumference), Error> {
    if Some(&true) == visited_set.get(pos) {
        Ok((0, 0))
    } else {
        visited_set
            .set(pos, true)
            .ok_or_else(|| Error::InvalidState("visited set board out of bound".into()))?;
        cardinal(pos)
            .filter_map(|next_pos| {
                board
                    .get(&next_pos)
                    .filter(|next_val| *next_val == val)
                    .map(|next_val| area_side(board, visited_set, &next_pos, next_val))
            })
            .try_fold((1, count_corner(board, pos)?), |(area, side), res| {
                let (next_area, next_side) = res?;
                Ok((area + next_area, side + next_side))
            })
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
        let input = get_input(2024, 12)?;
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
        let input = get_input(2024, 12)?;
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
