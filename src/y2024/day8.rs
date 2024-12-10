use crate::error::Error;
use crate::part_solver;
use crate::space::{Board2d, IterSpace, Space};
use crate::utils::{ires, ures};
use std::collections::{HashMap, HashSet};

part_solver!();

pub fn part1(input: &str) -> Result<ures, Error> {
    let board = parse_input(input)?;
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

    antinode_set
        .len()
        .try_into()
        .map_err(|e| Error::InvalidState(format!("cannot return correct result type: {}", e)))
}

pub fn part2(input: &str) -> Result<ures, Error> {
    let board = parse_input(input)?;
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

    antinode_set
        .len()
        .try_into()
        .map_err(|e| Error::InvalidState(format!("cannot return correct result type: {}", e)))
}

#[inline]
fn antinodes_1<'a, T>(
    &node1: &[usize; 2],
    &node2: &[usize; 2],
    board: &'a Board2d<T>,
) -> impl Iterator<Item = [usize; 2]> + use<'a, T> {
    forward_antinode_iter(node1, node2, board)
        .take(1)
        .chain(forward_antinode_iter(node2, node1, board).take(1))
}

#[inline]
fn antinodes_2<'a, T>(
    &node1: &[usize; 2],
    &node2: &[usize; 2],
    board: &'a Board2d<T>,
) -> impl Iterator<Item = [usize; 2]> + use<'a, T> {
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

    usize::try_from(x2 * 2 - x1)
        .and_then(|x| usize::try_from(y2 * 2 - y1).map(|y| [x, y]))
        .ok()
}

#[inline]
fn forward_antinode_iter<'a, T>(
    node1: [usize; 2],
    node2: [usize; 2],
    board: &'a Board2d<T>,
) -> impl Iterator<Item = [usize; 2]> + use<'a, T> {
    struct InternalIter<'b, U> {
        current: [usize; 2],
        next: [usize; 2],
        board: &'b Board2d<U>,
    }

    impl<'b, U> Iterator for InternalIter<'b, U> {
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

    InternalIter {
        current: node1,
        next: node2,
        board,
    }
}

fn parse_input(input: &str) -> Result<Board2d<u8>, Error> {
    let board_vec: Vec<_> = input.lines().map(|v| v.as_bytes().to_vec()).collect();

    Ok(board_vec.into())
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
        super::part2(input.as_str())?;
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
        super::part2(input.as_str())?;
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
