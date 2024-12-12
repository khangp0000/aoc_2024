use crate::error::Error;
use crate::part_solver;
use crate::space::{BitBoard2d, Board2d, IterSpace, Pos, RefBoard2d, Space};
use crate::utils::{cardinal, ures};
use std::borrow::Cow;

part_solver!();

type Area = ures;
type Circumference = ures;

#[derive(Copy, Clone)]
enum Wall {
    Left = 1,
    Top = 2,
    Right = 4,
    Bottom = 8,
}

impl Wall {
    fn has_wall(&self, val: u8) -> bool {
        (*self as u8 & val) != 0
    }
}

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
    let wall_board = board.map_ref(|pos, val| place_wall(&board, &pos, val));
    let mut cost = 0;
    for (pos, val) in board.iter() {
        let (area, side) = area_side(&board, &wall_board, visited_set, &pos, val)?;
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

fn place_wall(board: &RefBoard2d<u8>, pos: &[usize; 2], val: &u8) -> u8 {
    let mut output = 0;
    if Some(val) != pos.shift_dimension(0, -1).and_then(|pos| board.get(&pos)) {
        output |= Wall::Left as u8;
    }
    if Some(val) != pos.shift_dimension(1, -1).and_then(|pos| board.get(&pos)) {
        output |= Wall::Top as u8;
    }
    if Some(val) != pos.shift_dimension(0, 1).and_then(|pos| board.get(&pos)) {
        output |= Wall::Right as u8;
    }
    if Some(val) != pos.shift_dimension(1, 1).and_then(|pos| board.get(&pos)) {
        output |= Wall::Bottom as u8;
    }
    output
}

fn count_corner(wall_board: &Board2d<u8>, pos: &[usize; 2]) -> Result<ures, Error> {
    let val = wall_board
        .get(pos)
        .ok_or_else(|| Error::InvalidState("wall board out of bound".into()))?;
    let has_top = Wall::Top.has_wall(*val);
    let has_right = Wall::Right.has_wall(*val);
    let has_bottom = Wall::Bottom.has_wall(*val);
    let has_left = Wall::Left.has_wall(*val);

    let mut corner_count = 0;

    if (has_top && has_left)
        || (!has_top && !has_left && {
            let val = pos
                .shift(&[-1, -1])
                .and_then(|p| wall_board.get(&p))
                .cloned()
                .unwrap_or(15);
            Wall::Right.has_wall(val) && Wall::Bottom.has_wall(val)
        })
    {
        corner_count += 1;
    }

    if (has_top && has_right)
        || (!has_top && !has_right && {
            let val = pos
                .shift(&[1, -1])
                .and_then(|p| wall_board.get(&p))
                .cloned()
                .unwrap_or(15);
            Wall::Left.has_wall(val) && Wall::Bottom.has_wall(val)
        })
    {
        corner_count += 1;
    }

    if (has_bottom && has_left)
        || (!has_bottom && !has_left && {
            let val = pos
                .shift(&[-1, 1])
                .and_then(|p| wall_board.get(&p))
                .cloned()
                .unwrap_or(15);
            Wall::Right.has_wall(val) && Wall::Top.has_wall(val)
        })
    {
        corner_count += 1;
    }

    if (has_bottom && has_right)
        || (!has_bottom && !has_right && {
            let val = pos
                .shift(&[1, 1])
                .and_then(|p| wall_board.get(&p))
                .cloned()
                .unwrap_or(15);
            Wall::Left.has_wall(val) && Wall::Top.has_wall(val)
        })
    {
        corner_count += 1;
    }

    Ok(corner_count)
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
    wall_board: &Board2d<u8>,
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
                    .map(|next_val| area_side(board, wall_board, visited_set, &next_pos, next_val))
            })
            .try_fold((1, count_corner(wall_board, pos)?), |(area, side), res| {
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
