use crate::error::{Error, NomError};
use crate::nom::{fold_res_many1, single_line_not_eof, FinalParse};
use crate::part_solver;
use crate::space::space2d::{Board2d, RefBoard2d};
use crate::space::{IterSpace, Pos, Space};
use crate::utils::ures;
use indexmap::IndexSet;
use nom::character::complete::space0;
use nom::error::{FromExternalError, ParseError};
use nom::{IResult, Parser, Slice};
use nom_supreme::ParserExt;
use std::borrow::Cow;
use std::collections::VecDeque;
use std::num::NonZero;
use std::ops::DerefMut;

part_solver!();

type BoardAndStartPosCow<'a> = (Cow<'a, [u8]>, Option<(usize, &'a str)>);
type BoardAndStartPosVec<'a> = (Vec<u8>, Option<(usize, &'a str)>);

pub fn part1(input: &str) -> Result<ures, Error> {
    Ok(parse_board_and_solve_1.final_parse(input)?)
}

pub fn part2(input: &str) -> Result<ures, Error> {
    Ok(parse_board_and_solve_2.final_parse(input)?)
}

fn parse_board_line_1<'a>(
    input: &'a str,
) -> IResult<&'a str, BoardAndStartPosCow<'a>, NomError<'a>> {
    let bytes = input.as_bytes();
    let mut index_start = None;
    let mut idx = 0;
    for val in bytes.iter() {
        match *val {
            b'#' | b'.' | b'O' => {}
            b'@' => {
                if index_start.replace(idx).is_some() {
                    return Err(nom::Err::Failure(NomError::from_external_error(
                        input.slice(idx..),
                        nom::error::ErrorKind::Char,
                        Error::ParseError("multiple starting pos".into()),
                    )));
                }
            }
            _ => break,
        }

        idx += 1;
    }

    if idx == 0 {
        return Err(nom::Err::Error(NomError::from_error_kind(
            input,
            nom::error::ErrorKind::Many1,
        )));
    }

    let res_slice: Cow<'a, [u8]> = bytes.slice(0..idx).into();
    let remaining = input.slice(idx..);

    Ok((
        remaining,
        (res_slice, index_start.map(|i| (i, input.slice(i..)))),
    ))
}

fn parse_board<'a, B, O, P>(
    parse_board_line: P,
) -> impl Parser<&'a str, (B, [usize; 2]), NomError<'a>>
where
    B: From<Vec<O>>,
    P: Parser<&'a str, (O, Option<(usize, &'a str)>), NomError<'a>> + 'a,
{
    fold_res_many1(
        single_line_not_eof(parse_board_line.context("parse board line")),
        || (Vec::new(), None),
        |(mut lines, mut start), (line, found_start)| {
            if let Some((val, error_loc)) = found_start {
                if let Some(old_val) = start.replace([val, lines.len()]) {
                    return Err((
                        (lines, Some(old_val)),
                        Some(error_loc),
                        nom::Err::Failure(Error::ParseError("multiple starting pos".into())),
                    ));
                }
            }
            lines.push(line);
            Ok((lines, start))
        },
    )
    .map_res(|(board, start_pos)| {
        if let Some(start_pos) = start_pos {
            Ok((B::from(board), start_pos))
        } else {
            Err(Error::ParseError("cannot find starting pos".into()))
        }
    })
    .context("parse full board")
}

fn parse_board_1(input: &str) -> IResult<&str, (RefBoard2d<u8>, [usize; 2]), NomError> {
    parse_board(parse_board_line_1).parse(input)
}

fn parse_board_and_solve_1(input: &str) -> IResult<&str, ures, NomError> {
    let (remaining, (mut board, mut pos)) = parse_board_1(input)?;
    let (remaining, _) = single_line_not_eof(space0)
        .context("parse empty line")
        .parse(remaining)?;

    for (index, &input) in remaining.as_bytes().iter().enumerate() {
        match input {
            b'<' => step_1(&mut board, &mut pos, [-1, 0]),
            b'^' => step_1(&mut board, &mut pos, [0, -1]),
            b'>' => step_1(&mut board, &mut pos, [1, 0]),
            b'v' => step_1(&mut board, &mut pos, [0, 1]),
            b if b.is_ascii_whitespace() => continue,
            _ => {
                return Err(nom::Err::Failure(NomError::from_external_error(
                    remaining.slice(index..),
                    nom::error::ErrorKind::Char,
                    Error::ParseError("invalid direction character".into()),
                )));
            }
        }
    }

    let remaining = remaining.slice(remaining.len()..);
    Ok((remaining, point_1(&board)))
}

#[inline]
fn step_1(board: &mut RefBoard2d<u8>, pos: &mut [usize; 2], diff: [isize; 2]) {
    let next_pos = pos.shift(&diff);
    let next_pos = match next_pos {
        None => return,
        Some(pos) => pos,
    };

    let mut last_pos_val = board.get(&next_pos).map(|&v| (next_pos, v));
    loop {
        match last_pos_val {
            None => return,
            Some((_, b'#')) => return,
            Some((p, b'O')) => {
                last_pos_val = p.shift(&diff).and_then(|p| board.get(&p).map(|&v| (p, v)));
            }
            Some((p, _)) => {
                board.swap(&p, &next_pos);
                break;
            }
        }
    }

    board.swap(pos, &next_pos);
    *pos = next_pos;
}

fn parse_board_line_2(input: &str) -> IResult<&str, BoardAndStartPosVec, NomError> {
    let bytes = input.as_bytes();
    let mut index_start = None;
    let mut idx = 0;
    let mut v = Vec::new();

    for val in bytes.iter() {
        match *val {
            b'O' => {
                v.push(b'[');
                v.push(b']');
            }
            b'#' | b'.' => {
                v.push(*val);
                v.push(*val);
            }
            b'@' => {
                if index_start.replace(idx * 2).is_some() {
                    return Err(nom::Err::Failure(NomError::from_external_error(
                        input.slice(idx..),
                        nom::error::ErrorKind::Char,
                        Error::ParseError("multiple starting pos".into()),
                    )));
                } else {
                    v.push(b'@');
                    v.push(b'.');
                }
            }
            _ => break,
        }

        idx += 1;
    }

    if idx == 0 {
        return Err(nom::Err::Error(NomError::from_error_kind(
            input,
            nom::error::ErrorKind::Many1,
        )));
    }

    let remaining = input.slice(idx..);
    Ok((remaining, (v, index_start.map(|i| (i, input.slice(i..))))))
}

fn parse_board_2(input: &str) -> IResult<&str, (Board2d<u8>, [usize; 2]), NomError> {
    parse_board(parse_board_line_2).parse(input)
}

fn parse_board_and_solve_2(input: &str) -> IResult<&str, ures, NomError> {
    let (remaining, (mut board, mut pos)) = parse_board_2(input)?;
    let (remaining, _) = single_line_not_eof(space0)
        .context("parse empty line")
        .parse(remaining)?;
    let mut visited = IndexSet::new();
    for (index, &input) in remaining.as_bytes().iter().enumerate() {
        match input {
            b'<' => {
                let [x, y] = &mut pos;
                if let Some(line) = board.deref_mut().get_mut(*y) {
                    if step_horizontal_2(line.as_mut_slice(), *x, NonZero::new(-1).unwrap()) {
                        *x -= 1;
                    }
                }
            }
            b'^' => step_vertical_2(
                &mut board,
                &mut pos,
                NonZero::new(-1).unwrap(),
                &mut visited,
            ),
            b'>' => {
                let [x, y] = &mut pos;
                if let Some(line) = board.deref_mut().get_mut(*y) {
                    if step_horizontal_2(line.as_mut_slice(), *x, NonZero::new(1).unwrap()) {
                        *x += 1;
                    }
                }
            }
            b'v' => step_vertical_2(&mut board, &mut pos, NonZero::new(1).unwrap(), &mut visited),
            b if b.is_ascii_whitespace() => continue,
            _ => {
                return Err(nom::Err::Failure(NomError::from_external_error(
                    remaining.slice(index..),
                    nom::error::ErrorKind::Char,
                    Error::ParseError("invalid direction character".into()),
                )));
            }
        }
    }

    let remaining = remaining.slice(remaining.len()..);
    Ok((remaining, point_2(&board)))
}

#[inline]
fn check_vertical_2(
    board: &Board2d<u8>,
    mut queue: VecDeque<[usize; 2]>,
    y_diff: NonZero<isize>,
    visited: &mut IndexSet<[usize; 2]>,
) -> bool {
    while let Some(pos) = queue.pop_front() {
        let [x, y] = pos;

        let (left_pos, right_pos) = match y
            .checked_add_signed(y_diff.get())
            .and_then(|next_y| board.get(&[x, next_y]).map(|val| ([x, next_y], val)))
        {
            Some((_, &b'#')) | None => return false,
            Some((next_pos, &b']')) => {
                let [x, y] = &next_pos;
                ([x.checked_sub(1).unwrap(), *y], next_pos)
            }
            Some((next_pos, b'[')) => {
                let [x, y] = &next_pos;
                (next_pos, [x.checked_add(1).unwrap(), *y])
            }
            Some(_) => {
                continue;
            }
        };

        if !visited.insert(left_pos) {
            continue;
        }

        queue.push_back(left_pos);
        queue.push_back(right_pos);
    }

    true
}

#[inline]
fn step_vertical_2(
    board: &mut Board2d<u8>,
    pos: &mut [usize; 2],
    y_diff: NonZero<isize>,
    visited: &mut IndexSet<[usize; 2]>,
) {
    let mut queue = VecDeque::new();
    queue.push_back(*pos);
    if check_vertical_2(board, queue, y_diff, visited) {
        let y_diff = y_diff.get();
        for box_left in visited.drain(..).rev() {
            board.swap(&box_left, &box_left.shift_dimension(1, y_diff).unwrap());
            board.swap(
                &box_left.shift_dimension(0, 1).unwrap(),
                &box_left.shift(&[1, y_diff]).unwrap(),
            );
        }

        let next_pos = pos.shift_dimension(1, y_diff).unwrap();
        board.swap(pos, &next_pos);
        *pos = next_pos;
    } else {
        visited.clear()
    }
}

#[inline]
fn step_horizontal_2(line: &mut [u8], x_pos: usize, x_diff: NonZero<isize>) -> bool {
    let next = x_pos
        .checked_add_signed(x_diff.get())
        .and_then(|next_x| line.get(next_x).map(|&v| (next_x, v)));
    if let Some((next_x, next_val)) = next {
        match next_val {
            b'#' => false,
            b'[' | b']' => {
                if step_horizontal_2(line, next_x, x_diff) {
                    line.swap(x_pos, next_x);
                    true
                } else {
                    false
                }
            }
            _ => {
                line.swap(x_pos, next_x);
                true
            }
        }
    } else {
        false
    }
}

fn point_1<T: IterSpace<u8, usize, 2>>(board: &T) -> ures {
    point_with_filter(board, |val| val == b'O')
}

fn point_2<T: IterSpace<u8, usize, 2>>(board: &T) -> ures {
    point_with_filter(board, |val| val == b'[')
}

fn point_with_filter<T: IterSpace<u8, usize, 2>, F: Fn(u8) -> bool>(board: &T, filter: F) -> ures {
    board
        .iter()
        .filter(|(_, &val)| filter(val))
        .map(|([x, y], _)| 100 * y as ures + x as ures)
        .sum()
}

#[allow(dead_code)]
fn check_board(board: &Board2d<u8>) -> bool {
    if let Some((bad_pos, _)) = board.iter().find(|(pos, &v)| {
        v == b'['
            && pos
                .shift_dimension(0, 1)
                .and_then(|p| board.get(&p))
                .cloned()
                != Some(b']')
    }) {
        println!("{:?}", bad_pos);
        return false;
    };
    true
}

#[cfg(test)]
mod tests {
    use crate::error::Error;

    use crate::utils::tests_utils::{get_input, human_text_duration};

    use chrono::Utc;

    #[test]
    pub fn part1() -> Result<(), Error> {
        let start = Utc::now();
        let input = get_input(2024, 15)?;
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
        let input = get_input(2024, 15)?;
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
