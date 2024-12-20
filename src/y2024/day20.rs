use crate::error::{Error, NomError};
use crate::graph::MaybeProcessed::Processed;
use crate::graph::{Bfs, NeighborFn};
use crate::nom::{fold_res_many1, single_line, FinalParse};
use crate::part_solver;
use crate::set::OptionSpace;
use crate::space::space2d::{Board2d, Direction, RefBoard2d};
use crate::space::Space;
use crate::space::{IterSpace, Pos};
use crate::utils::ures;
use derive_more::{Deref, From};
use nom::error::FromExternalError;
use nom::error::ParseError;
use nom::{IResult, Parser, Slice};
use nom_supreme::ParserExt;
use std::borrow::Cow;
use std::collections::VecDeque;
use std::convert::identity;
use std::ops::ControlFlow::{Break, Continue};

part_solver!();

type LineAndStartPosAndEndPosCow<'a> = (
    Cow<'a, [u8]>,
    Option<(usize, &'a str)>,
    Option<(usize, &'a str)>,
);

type BoardAndStartPosAndEndPos<'a> = (RefBoard2d<'a, u8>, [usize; 2], [usize; 2]);

struct WalkableNeighbor<'a> {
    board: &'a RefBoard2d<'a, u8>,
    start: [usize; 2],
    end: [usize; 2],
    error: Option<Error>,
}

type State = [usize; 2];
type Metadata = ();

impl NeighborFn<(State, Metadata)> for WalkableNeighbor<'_> {
    fn get_neighbors(
        &mut self,
        sm: &(State, Metadata),
    ) -> impl IntoIterator<Item = (State, Metadata)> {
        if self.error.is_some() {
            return Vec::default();
        }
        let (state, _) = &sm;
        let board = self.board;
        let pos = state;
        let res = Direction::cardinal()
            .iter()
            .filter_map(|new_direction| {
                pos.shift(new_direction.get_movement_vec())
                    .filter(|new_pos| board.get(new_pos) != Some(&b'#'))
            })
            .map(|new_state| (new_state, ()))
            .try_fold(Vec::with_capacity(2), |mut v, s| {
                if v.len() == 2 {
                    self.error.replace(Error::InvalidState(
                        format!(
                            "there are path intersection which is not supported: {:?}",
                            pos
                        )
                        .into(),
                    ));
                    Break(Vec::default())
                } else {
                    v.push(s);
                    Continue(v)
                }
            });

        let v = match res {
            Continue(v) => v,
            Break(v) => v,
        };

        if self.error.is_none() {
            if pos == &self.start || pos == &self.end {
                if v.len() != 1 {
                    self.error.replace(Error::InvalidState(
                        "starting or ending position is not terminated".into(),
                    ));
                }
            } else if v.len() != 2 {
                unreachable!("every middle position should have 2 neighbor")
            }
        }
        v
    }
}

pub fn part1(input: &str) -> Result<ures, Error> {
    inner_solver::<2>(input)
}

pub fn part2(input: &str) -> Result<ures, Error> {
    inner_solver::<20>(input)
}

fn inner_solver<const N: usize>(input: &str) -> Result<ures, Error> {
    let (board, start, end) = parse_input.final_parse(input)?;

    let distance_from_start = Board2d::from(
        board
            .deref()
            .iter()
            .map(|line| vec![None; line.len()])
            .collect::<Vec<_>>(),
    );
    let mut bfs = Bfs {
        queue: VecDeque::new(),
        neighbor_fn: WalkableNeighbor {
            board: &board,
            start,
            end,
            error: None,
        },
        visited: OptionSpace::from(distance_from_start),
    };
    bfs.queue.push_back((start, ()));

    let mut distance = 0;
    loop {
        match bfs.next() {
            None => break,
            Some(Err(e)) => return Err(e),
            Some(Ok(Processed((pos, _)))) => {
                if let Some(error) = bfs.neighbor_fn.error.take() {
                    return Err(error);
                }
                bfs.visited.set(&pos, Some(distance));
                distance += 1;
            }
            _ => continue,
        }
    }

    Ok(count_cheat(bfs.visited.deref(), N, |saved| saved >= 100))
}

fn parse_board_line<'a>(
    input: &'a str,
) -> IResult<&'a str, LineAndStartPosAndEndPosCow<'a>, NomError<'a>> {
    let bytes = input.as_bytes();
    let mut index_start = None;
    let mut index_end = None;
    let mut idx = 0;
    for val in bytes.iter() {
        match *val {
            b'#' | b'.' => {}
            b'S' => {
                if index_start.replace(idx).is_some() {
                    return Err(nom::Err::Failure(NomError::from_external_error(
                        input.slice(idx..),
                        nom::error::ErrorKind::Char,
                        Error::ParseError("multiple starting pos".into()),
                    )));
                }
            }
            b'E' => {
                if index_end.replace(idx).is_some() {
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
        (
            res_slice,
            index_start.map(|i| (i, input.slice(i..))),
            index_end.map(|i| (i, input.slice(i..))),
        ),
    ))
}

fn parse_board<'a, B, O, P>(
    parse_board_line: P,
) -> impl Parser<&'a str, (B, [usize; 2], [usize; 2]), NomError<'a>>
where
    B: From<Vec<O>>,
    P: Parser<&'a str, (O, Option<(usize, &'a str)>, Option<(usize, &'a str)>), NomError<'a>> + 'a,
{
    fold_res_many1(
        single_line(parse_board_line.context("parse board line")),
        || (Vec::new(), None, None),
        |(mut lines, mut start, mut end), (line, found_start, found_end)| {
            if let Some((val, error_loc)) = found_start {
                if let Some(old_val) = start.replace([val, lines.len()]) {
                    return Err((
                        (lines, Some(old_val), end),
                        Some(error_loc),
                        nom::Err::Failure(Error::ParseError("multiple starting pos".into())),
                    ));
                }
            }
            if let Some((val, error_loc)) = found_end {
                if let Some(old_val) = end.replace([val, lines.len()]) {
                    return Err((
                        (lines, start, Some(old_val)),
                        Some(error_loc),
                        nom::Err::Failure(Error::ParseError("multiple starting pos".into())),
                    ));
                }
            }
            lines.push(line);
            Ok((lines, start, end))
        },
    )
    .map_res(|(board, start_pos, end_pos)| {
        if let Some(start_pos) = start_pos {
            if let Some(end_pos) = end_pos {
                return Ok((B::from(board), start_pos, end_pos));
            }
        }
        Err(Error::ParseError("cannot find starting pos".into()))
    })
    .context("parse full board")
}

fn parse_input(input: &str) -> IResult<&str, BoardAndStartPosAndEndPos, NomError> {
    parse_board(parse_board_line).parse(input)
}

fn count_cheat<Predicate: Fn(usize) -> bool>(
    board: &Board2d<Option<usize>>,
    max_cheat_sec: usize,
    save_count_filter: Predicate,
) -> ures {
    let mut count = 0;
    for (pos, distance) in board.iter() {
        if let &Some(distance) = distance {
            let [x, y] = pos;
            for y_diff in 0..=max_cheat_sec {
                let y = y.checked_add(y_diff);
                if let Some(y) = y {
                    for x_diff in 2usize.saturating_sub(y_diff)..=(max_cheat_sec - y_diff) {
                        if x_diff == 0 {
                            if let Some(&Some(distance_2)) = board.get(&[x, y]) {
                                if save_count_filter(distance_2.abs_diff(distance) - y_diff)
                                {
                                    count += 1;
                                }
                            }
                        } else if y_diff == 0 {
                            if let Some(&Some(distance_2)) = x.checked_add(x_diff).and_then(|x| board.get(&[x, y])) {
                                if save_count_filter(distance_2.abs_diff(distance) - x_diff)
                                {
                                    count += 1;
                                }
                            }
                        } else {
                            let euclid_distance = x_diff + y_diff;
                            let add_count = [x.checked_sub(x_diff), x.checked_add(x_diff)]
                                .into_iter()
                                .filter_map(identity)
                                .filter_map(|x| board.get(&[x, y]))
                                .filter_map(|x| x.as_ref())
                                .filter(|&&distance_2| save_count_filter(distance_2.abs_diff(distance) - euclid_distance))
                                .count();
                            count += add_count as ures;
                        };
                    }
                }
            }
        }
    }

    count
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::utils::tests_utils::{get_input, human_text_duration};
    use chrono::Utc;

    #[test]
    pub fn part1() -> Result<(), Error> {
        let start = Utc::now();
        let input = get_input(2024, 20)?;
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
        let input = get_input(2024, 20)?;
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
