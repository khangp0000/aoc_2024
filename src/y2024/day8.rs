use crate::error::Error;
use crate::part_solver;
use crate::space::space2d::RefBoard2d;
use crate::space::{IterSpace, Space};
use crate::utils::{ires, ures};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;

part_solver!();

pub fn part1(input: &str) -> Result<ures, Error> {
    let board = parse_input(input);
    let (_, antinode_set) = board.iter().fold(
        (HashMap::<u8, Vec<[usize; 2]>>::new(), HashSet::new()),
        |(mut map, mut antinode_set), (coord, &value)| {
            if value != b'.' {
                let prev_coords = map.entry(value).or_default();
                prev_coords
                    .iter()
                    .flat_map(|node| antinodes_1(node, &coord, &board))
                    .for_each(|antinode| {
                        antinode_set.insert(antinode);
                    });
                prev_coords.push(coord);
            }
            (map, antinode_set)
        },
    );

    Ok(antinode_set.len() as ures)
}

pub fn part2(input: &str) -> Result<ures, Error> {
    let board = parse_input(input);
    let (_, antinode_set) = board.iter().fold(
        (HashMap::<u8, Vec<[usize; 2]>>::new(), HashSet::new()),
        |(mut map, mut antinode_set), (coord, &value)| {
            if value != b'.' {
                let prev_coords = map.entry(value).or_default();
                prev_coords
                    .iter()
                    .flat_map(|node| antinodes_2(node, &coord, &board))
                    .for_each(|antinode| {
                        antinode_set.insert(antinode);
                    });
                prev_coords.push(coord);
            }
            (map, antinode_set)
        },
    );

    Ok(antinode_set.len() as ures)
}

#[inline]
fn antinodes_1<'a, T, B: Space<T, usize, 2>>(
    &node1: &[usize; 2],
    &node2: &[usize; 2],
    board: &'a B,
) -> impl Iterator<Item = [usize; 2]> + use<'a, T, B> {
    forward_antinode_iter(node1, node2, board)
        .take(1)
        .chain(forward_antinode_iter(node2, node1, board).take(1))
}

#[inline]
fn antinodes_2<'a, T, B: Space<T, usize, 2>>(
    &node1: &[usize; 2],
    &node2: &[usize; 2],
    board: &'a B,
) -> impl Iterator<Item = [usize; 2]> + use<'a, T, B> {
    [node1, node2]
        .into_iter()
        .chain(forward_antinode_iter(node1, node2, board))
        .chain(forward_antinode_iter(node2, node1, board))
}

#[inline]
fn forward_antinode(node1: [usize; 2], node2: [usize; 2]) -> Option<[usize; 2]> {
    let [x1, y1] = node1;
    let [x2, y2] = node2;
    let x1 = x1 as ires;
    let x2 = x2 as ires;
    let y1 = y1 as ires;
    let y2 = y2 as ires;

    Some([
        x2.checked_mul(2)?.checked_sub(x1)? as usize,
        y2.checked_mul(2)?.checked_sub(y1)? as usize,
    ])
}

#[inline]
fn forward_antinode_iter<T, B: Space<T, usize, 2>>(
    node1: [usize; 2],
    node2: [usize; 2],
    board: &B,
) -> impl Iterator<Item = [usize; 2]> + use<'_, T, B> {
    struct InternalIter<'b, BB: Space<U, usize, 2>, U> {
        current: [usize; 2],
        next: [usize; 2],
        board: &'b BB,
        phantom_data: PhantomData<U>,
    }

    impl<BB: Space<U, usize, 2>, U> Iterator for InternalIter<'_, BB, U> {
        type Item = [usize; 2];

        fn next(&mut self) -> Option<Self::Item> {
            let new_next =
                forward_antinode(self.current, self.next).filter(|c| self.board.get(c).is_some());
            if let Some(new_next) = new_next {
                self.current = self.next;
                self.next = new_next;
            }
            new_next
        }
    }

    InternalIter::<B, T> {
        current: node1,
        next: node2,
        board,
        phantom_data: PhantomData,
    }
}

fn parse_input(input: &str) -> RefBoard2d<u8> {
    input
        .lines()
        .map(|line| Cow::Borrowed(line.as_bytes()))
        .collect::<Vec<_>>()
        .into()
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::utils::tests_utils::{get_input, human_text_duration};
    use chrono::Utc;

    #[test]
    pub fn part1() -> Result<(), Error> {
        let start = Utc::now();
        let input = get_input(2024, 8)?;
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
        let input = get_input(2024, 8)?;
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
