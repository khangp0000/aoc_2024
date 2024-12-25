use crate::error::{Error, NomError};
use crate::nom::{single_line, FinalParse};
use crate::part_solver;
use crate::utils::ures;
use indexmap::{IndexMap, IndexSet};
use nom::character::complete::{alpha1, char};
use nom::multi::many1;
use nom::sequence::separated_pair;
use nom::{IResult, Parser};
use nom_supreme::ParserExt;
use std::cell::RefCell;
use std::collections::{BTreeSet, HashSet};

part_solver!();

pub fn part1(input: &str) -> Result<ures, Error> {
    let edges = parse_input.final_parse(input)?;
    let mut neighbor = IndexMap::new();
    for (v1, v2) in edges {
        let entry_1 = neighbor.entry(v1);
        let idx_1 = entry_1.index();
        entry_1.or_insert_with(|| {
            if v1.starts_with('t') {
                (true, RefCell::new(HashSet::new()))
            } else {
                (false, RefCell::new(HashSet::new()))
            }
        });
        let entry_2 = neighbor.entry(v2);
        let idx_2 = entry_2.index();
        entry_2.or_insert_with(|| {
            if v2.starts_with('t') {
                (true, RefCell::new(HashSet::new()))
            } else {
                (false, RefCell::new(HashSet::new()))
            }
        });
        let (_, (_, neighbor1)) = neighbor.get_index_mut(idx_1).unwrap();
        neighbor1.get_mut().insert(idx_2);
        let (_, (_, neighbor2)) = neighbor.get_index_mut(idx_2).unwrap();
        neighbor2.get_mut().insert(idx_1);
    }

    let res = neighbor
        .values()
        .enumerate()
        .filter(|(_node, (start_t, _neighbor))| *start_t)
        .map(|(node, (_start_t, my_neighbor))| {
            let mut sum = 0;
            let binding = my_neighbor.borrow();
            let mut iter = binding.iter();
            while let Some(first_neighbor) = iter.next() {
                let iter = iter.clone();
                let (_, (_, first_neighbor_neighbors)) =
                    neighbor.get_index(*first_neighbor).unwrap();
                first_neighbor_neighbors.borrow_mut().remove(&node);
                for second_neighbor in iter {
                    if first_neighbor_neighbors.borrow().contains(second_neighbor) {
                        sum += 1
                    }
                }
            }
            sum
        })
        .sum();
    Ok(res)
}

pub fn part2(input: &str) -> Result<String, Error> {
    let edges = parse_input.final_parse(input)?;
    let mut neighbor = IndexMap::new();
    for (v1, v2) in edges {
        let entry_1 = neighbor.entry(v1);
        let idx_1 = entry_1.index();
        entry_1.or_insert_with(IndexSet::new);
        let entry_2 = neighbor.entry(v2);
        let idx_2 = entry_2.index();
        entry_2.or_insert_with(IndexSet::new);
        let (_, neighbor1) = neighbor.get_index_mut(idx_1).unwrap();
        neighbor1.insert(idx_2);
        let (_, neighbor2) = neighbor.get_index_mut(idx_2).unwrap();
        neighbor2.insert(idx_1);
    }

    let potentials = (0..neighbor.len()).collect::<IndexSet<_>>();
    let mut all_cliques = HashSet::new();
    bron_kerbosch(
        &BTreeSet::new(),
        potentials,
        IndexSet::new(),
        &neighbor,
        &mut all_cliques,
    );
    let mut largest_clique = all_cliques
        .into_iter()
        .max_by_key(|clique| clique.len())
        .ok_or_else(|| Error::InvalidState("no clique found".into()))?
        .into_iter()
        .map(|index| {
            let (name, _) = neighbor.get_index(index).unwrap();
            name
        })
        .cloned()
        .collect::<Vec<_>>();

    largest_clique.sort();
    Ok(largest_clique.join(","))
}

fn parse_edge(input: &str) -> IResult<&str, (&str, &str), NomError> {
    separated_pair(alpha1, char('-'), alpha1)
        .context("parse edge")
        .parse(input)
}

fn parse_input(input: &str) -> IResult<&str, Vec<(&str, &str)>, NomError> {
    many1(single_line(parse_edge)).parse(input)
}

fn bron_kerbosch(
    current_clique: &BTreeSet<usize>,
    mut potentials: IndexSet<usize>,
    mut visited: IndexSet<usize>,
    graph: &IndexMap<&str, IndexSet<usize>>,
    cliques: &mut HashSet<BTreeSet<usize>>,
) {
    while let Some(&v) = potentials.last() {
        let mut next_clique = current_clique.clone();
        next_clique.insert(v);
        let (_, neighbor) = graph.get_index(v).unwrap();
        let next_potentials = potentials
            .intersection(neighbor)
            .cloned()
            .collect::<IndexSet<usize>>();
        let next_visited = visited
            .intersection(neighbor)
            .cloned()
            .collect::<IndexSet<usize>>();
        if next_potentials.is_empty() && next_visited.is_empty() {
            cliques.insert(next_clique);
        } else {
            bron_kerbosch(&next_clique, next_potentials, next_visited, graph, cliques);
        }
        potentials.swap_remove(&v);
        visited.insert(v);
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
        let input = get_input(2024, 23)?;
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
        let input = get_input(2024, 23)?;
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
