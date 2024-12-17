use crate::error::{Error, NomError};
use crate::graph::{Dijkstra, NeighborFn};
use crate::nom::{fold_res_many1, single_line, FinalParse};
use crate::part_solver;
use crate::space::space2d::{Direction, RefBoard2d};
use crate::space::{Pos, Space};
use crate::utils::ures;
use derive_more::{Deref, DerefMut, From, Into};
use nom::error::FromExternalError;
use nom::error::ParseError;
use nom::{IResult, Parser, Slice};
use nom_supreme::ParserExt;
use std::borrow::Cow;
use std::cmp::{Ordering, Reverse};
use std::collections::{BinaryHeap, HashMap, HashSet};

part_solver!();
type LineAndStartPosAndEndPosCow<'a> = (
    Cow<'a, [u8]>,
    Option<(usize, &'a str)>,
    Option<(usize, &'a str)>,
);

type BoardAndStartPosAndEndPos<'a> = (RefBoard2d<'a, u8>, [usize; 2], [usize; 2]);

type State = ([usize; 2], Direction);
type Weight = ures;
type Metadata = ();

#[derive(Deref, DerefMut, From)]
struct BoardNeighbor<'a>(RefBoard2d<'a, u8>);

struct TrackBestParents<'a> {
    board_neighbor_fn: BoardNeighbor<'a>,
    best_parents: HashMap<State, (Weight, Vec<State>)>,
}

impl<'a> From<RefBoard2d<'a, u8>> for TrackBestParents<'a> {
    fn from(value: RefBoard2d<'a, u8>) -> Self {
        Self {
            board_neighbor_fn: value.into(),
            best_parents: HashMap::new(),
        }
    }
}

impl NeighborFn<State, Weight, Metadata> for BoardNeighbor<'_> {
    fn get_neighbors(
        &mut self,
        state: &State,
        weight: &Weight,
        _: &Metadata,
    ) -> impl IntoIterator<Item = (State, Weight, Metadata)> {
        let board = self.deref_mut();
        let (pos, old_direction) = state;
        let res = Direction::cardinal()
            .iter()
            .filter_map(|new_direction| {
                pos.shift(new_direction.get_movement_vec())
                    .filter(|new_pos| board.get(new_pos).is_some_and(|&v| v != b'#'))
                    .map(|new_pos| (new_pos, *new_direction))
            })
            .map(|new_state| {
                let (_, new_direction) = &new_state;
                (
                    new_state,
                    *weight + calculate_cost(*old_direction, *new_direction),
                    (),
                )
            });
        res
    }
}

impl NeighborFn<State, Weight, Metadata> for TrackBestParents<'_> {
    fn get_neighbors(
        &mut self,
        state: &State,
        weight: &Weight,
        m: &Metadata,
    ) -> impl IntoIterator<Item = (State, Weight, Metadata)> {
        self.board_neighbor_fn
            .get_neighbors(state, weight, m)
            .into_iter()
            .inspect(|(child_state, weight, _)| {
                self.best_parents
                    .entry(*child_state)
                    .and_modify(|(best_weight, best_parents_list)| {
                        match (*best_weight).cmp(weight) {
                            Ordering::Equal => best_parents_list.push(*state),
                            Ordering::Greater => {
                                best_parents_list.clear();
                                best_parents_list.push(*state);
                                *best_weight = *weight;
                            }
                            Ordering::Less => {}
                        }
                    })
                    .or_insert((*weight, vec![*state]));
            })
    }
}

fn calculate_cost(old_direction: Direction, new_direction: Direction) -> ures {
    match new_direction {
        d if d == old_direction => 1,
        d if d == old_direction.opposite() => 2001,
        _ => 1001,
    }
}

pub fn part1(input: &str) -> Result<ures, Error> {
    let (board, start, end) = parse_board_1.final_parse(input)?;

    let mut dijkstra = Dijkstra {
        queue: BinaryHeap::new(),
        neighbor_fn: BoardNeighbor::from(board),
        visited: HashSet::new(),
    };
    dijkstra
        .queue
        .push(Reverse(((start, Direction::East), 0, ()).into()));
    loop {
        match dijkstra.next() {
            None => return Err(Error::Unsolvable("cannot find path to end".into())),
            Some(Err(e)) => return Err(e),
            Some(Ok((state, weight, _metadata))) => {
                let (next_pos, _) = state;
                if next_pos == end {
                    return Ok(weight);
                }
            }
        }
    }
}

pub fn part2(input: &str) -> Result<ures, Error> {
    let (board, start, end) = parse_board_1.final_parse(input)?;
    let mut dijkstra = Dijkstra {
        queue: BinaryHeap::new(),
        neighbor_fn: TrackBestParents::from(board),
        visited: HashSet::new(),
    };
    dijkstra
        .queue
        .push(Reverse(((start, Direction::East), 0, ()).into()));
    loop {
        match dijkstra.next() {
            None => return Err(Error::Unsolvable("cannot find path to end".into())),
            Some(Err(e)) => return Err(e),
            Some(Ok((state, _weight, _metadata))) => {
                let (next_pos, _) = state;
                if next_pos == end {
                    break;
                }
            }
        }
    }
    let best_parents = dijkstra.neighbor_fn.best_parents;
    let mut visited = HashSet::new();
    visited.insert((start, Direction::East));
    let (mut work, _) = Direction::cardinal()
        .iter()
        .map(|d| State::from((end, *d)))
        .fold((Vec::new(), None), |(mut vec, mut min_cost), state| {
            if let Some(&(weight, _)) = best_parents.get(&state) {
                let stored_min_cost = min_cost.get_or_insert(weight);
                match (*stored_min_cost).cmp(&weight) {
                    Ordering::Equal => vec.push(state),
                    Ordering::Greater => {
                        vec.clear();
                        vec.push(state);
                        *stored_min_cost = weight;
                    }
                    Ordering::Less => {}
                }
            }
            (vec, min_cost)
        });
    while let Some(state) = work.pop() {
        if visited.contains(&state) {
            continue;
        }
        if let Some((_, best_parents_of_state)) = best_parents.get(&state) {
            best_parents_of_state
                .iter()
                .filter(|&s| !visited.contains(s))
                .for_each(|s| work.push(*s))
        }
        visited.insert(state);
    }

    let pos_set: HashSet<[usize; 2]> = visited
        .iter()
        .map(|state| {
            let &(pos, _) = state;
            pos
        })
        .collect();

    Ok(pos_set.len() as ures)
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

fn parse_board_1(input: &str) -> IResult<&str, BoardAndStartPosAndEndPos, NomError> {
    parse_board(parse_board_line).parse(input)
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::utils::tests_utils::{get_input, human_text_duration};
    use chrono::Utc;

    #[test]
    pub fn part1() -> Result<(), Error> {
        let start = Utc::now();
        let input = get_input(2024, 16)?;
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
        let input = get_input(2024, 16)?;
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
