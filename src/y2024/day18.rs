use crate::error::{Error, NomError};
use crate::graph::MaybeProcessed::Processed;
use crate::graph::{Bfs, NeighborFn};
use crate::nom::{single_line, single_line_not_eof, trim_space, ures, FinalParse};
use crate::part_solver;
use crate::set::{BoolSpace, Set};
use crate::space::space2d::{Board2d, Direction};
use crate::space::{Pos, Space};
use crate::utils::{cardinal, musize, ures};
use derive_more::{Deref, DerefMut, From};
use nom::character::complete::char;
use nom::multi::{many1, many_m_n};
use nom::sequence::separated_pair;
use nom::{IResult, Parser};
use nom_supreme::ParserExt;
use std::borrow::BorrowMut;
use std::collections::VecDeque;
use std::ops::ControlFlow::{Break, Continue};

part_solver!();

#[derive(Deref, DerefMut, From)]
struct NonCorruptedNeighbor<'a, Vy: BorrowMut<[Vx]>, Vx: BorrowMut<[bool]>>(
    &'a mut Board2d<bool, Vy, Vx>,
);

type State = [usize; 2];
type Metadata = ures;
type Metadata2 = ();

impl<Vy: BorrowMut<[Vx]>, Vx: BorrowMut<[bool]>> NeighborFn<(State, Metadata)>
    for NonCorruptedNeighbor<'_, Vy, Vx>
{
    fn get_neighbors(
        &mut self,
        sm: &(State, Metadata),
    ) -> impl IntoIterator<Item = (State, Metadata)> {
        let (state, cost) = &sm;
        let board = self.deref_mut();
        let pos = state;
        let res = Direction::cardinal()
            .iter()
            .filter_map(|new_direction| {
                pos.shift(new_direction.get_movement_vec())
                    .filter(|new_pos| board.get(new_pos) == Some(&false))
            })
            .map(|new_state| (new_state, *cost + 1));
        res
    }
}

impl<Vy: BorrowMut<[Vx]>, Vx: BorrowMut<[bool]>> NeighborFn<(State, Metadata2)>
    for NonCorruptedNeighbor<'_, Vy, Vx>
{
    fn get_neighbors(
        &mut self,
        sm: &(State, Metadata2),
    ) -> impl IntoIterator<Item = (State, Metadata2)> {
        let (state, _) = &sm;
        let board = self.deref_mut();
        let pos = state;
        let res = Direction::cardinal()
            .iter()
            .filter_map(|new_direction| {
                pos.shift(new_direction.get_movement_vec())
                    .filter(|new_pos| board.get(new_pos) == Some(&false))
            })
            .map(|new_state| (new_state, ()));
        res
    }
}

pub fn part1(input: &str) -> Result<ures, Error> {
    let mut board = Board2d::from([[false; 71]; 71]);
    parse_coords_part_1
        .partial_parse(input)?
        .into_iter()
        .try_for_each(|(x, y)| board.set(&[x, y], true).map(|_| ()))
        .ok_or_else(|| Error::InvalidState("out of bound".into()))?;

    let mut bfs = Bfs {
        queue: VecDeque::new(),
        neighbor_fn: NonCorruptedNeighbor::from(&mut board),
        visited: BoolSpace::from(Board2d::from([[false; 71]; 71])),
    };
    bfs.queue.push_back(([0, 0], 0));

    loop {
        match bfs.next() {
            None => return Err(Error::Unsolvable("cannot find path to end".into())),
            Some(Err(e)) => return Err(e),
            Some(Ok(Processed((state, cost)))) => {
                if state == [70, 70] {
                    return Ok(cost);
                }
            }
            _ => continue,
        }
    }
}

pub fn part2(input: &str) -> Result<String, Error> {
    let mut board = Board2d::from([[false; 71]; 71]);
    let corrupted: Vec<_> = parse_coords_part_2.final_parse(input)?;
    corrupted
        .iter()
        .try_for_each(|(x, y)| board.set(&[*x, *y], true).map(|_| ()))
        .ok_or_else(|| Error::InvalidState("out of bound".into()))?;

    let mut bfs = Bfs {
        queue: VecDeque::new(),
        neighbor_fn: NonCorruptedNeighbor::from(&mut board),
        visited: BoolSpace::from(Board2d::from([[false; 71]; 71])),
    };
    bfs.queue.push_back(([0, 0], ()));

    loop {
        match bfs.next() {
            None => break,
            Some(Err(e)) => return Err(e),
            Some(Ok(Processed((state, _)))) => {
                if state == [70, 70] {
                    return Err(Error::Unsolvable(
                        "reach exit without remove any block!".into(),
                    ));
                }
            }
            _ => continue,
        }
    }

    for (rm_x, rm_y) in corrupted.into_iter().rev() {
        bfs.neighbor_fn.deref_mut().set(&[rm_x, rm_y], false);

        if let Break(res) = cardinal(&[rm_x, rm_y])
            .filter(|p| bfs.neighbor_fn.deref().get(p).is_some())
            .try_fold((), |_, elem| {
                let res = bfs.visited.contains(&elem);
                if res == Ok(false) {
                    Continue(())
                } else {
                    Break(res)
                }
            })
        {
            res?;
            bfs.queue.push_back(([rm_x, rm_y], ()));
            loop {
                match bfs.next() {
                    None => break,
                    Some(Err(e)) => return Err(e),
                    Some(Ok(Processed((state, _)))) => {
                        if state == [70, 70] {
                            return Ok(format!("{},{}", rm_x, rm_y));
                        }
                    }
                    _ => continue,
                }
            }
        }
    }

    Err(Error::InvalidState(
        "cannot find any path after remove all corruption".into(),
    ))
}

fn parse_coord_line(input: &str) -> IResult<&str, (usize, usize), NomError<'_>> {
    separated_pair(
        ures.map(|v| v as musize),
        char(','),
        ures.map(|v| v as musize),
    )
    .context("parsing ures pair")
    .parse(input)
}

fn parse_coords_part_1(input: &str) -> IResult<&str, Vec<(usize, usize)>, NomError<'_>> {
    many_m_n(
        1024,
        1024,
        single_line_not_eof(trim_space(parse_coord_line)),
    )
    .parse(input)
}

fn parse_coords_part_2(input: &str) -> IResult<&str, Vec<(usize, usize)>, NomError<'_>> {
    many1(single_line(trim_space(parse_coord_line))).parse(input)
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::utils::tests_utils::{get_input, human_text_duration};
    use chrono::Utc;

    #[test]
    pub fn part1() -> Result<(), Error> {
        let start = Utc::now();
        let input = get_input(2024, 18)?;
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
        let input = get_input(2024, 18)?;
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
