use crate::error::Error;
use crate::part_solver;
use crate::utils::ures;
use bit_set::BitSet;
use std::collections::HashMap;
use std::str::FromStr;

part_solver!();

type ChildrenRelationShip = HashMap<usize, BitSet<usize>>;

pub fn part1(input: &str) -> Result<ures, Error> {
    let (relationship, all_lists) = parse_input(input)?;
    let res = all_lists
        .iter()
        .filter(|list| is_valid(list, &relationship))
        .map(|list| list[list.len() / 2] as ures)
        .sum();

    Ok(res)
}

pub fn part2(input: &str) -> Result<ures, Error> {
    let (relationship, all_lists) = parse_input(input)?;
    let res = all_lists
        .iter()
        .filter(|list| !is_valid(list, &relationship))
        .map(|list| (BitSet::<usize>::from_iter(list.iter().cloned()), list))
        .map(|(original_list_bit_set, list)| {
            let mut fixed: Vec<usize> = Vec::with_capacity(list.len());
            let mut visited = BitSet::<usize>::default();
            list.iter().for_each(|val| {
                fix_reverse(
                    *val,
                    &original_list_bit_set,
                    &relationship,
                    &mut visited,
                    &mut fixed,
                )
            });
            fixed
        })
        .map(|list| list[list.len() / 2] as ures)
        .sum();
    Ok(res)
}

fn fix_reverse(
    val: usize,
    original_list_bit_set: &BitSet<usize>,
    relationship: &ChildrenRelationShip,
    visited: &mut BitSet<usize>,
    res: &mut Vec<usize>,
) {
    if !visited.contains(val) {
        visited.insert(val);
        let mut children = relationship.get(&val).cloned().unwrap_or_default();
        children.intersect_with(original_list_bit_set);
        for child in children.iter() {
            fix_reverse(child, original_list_bit_set, relationship, visited, res);
        }
        res.push(val);
    }
}

fn is_valid(list: &Vec<usize>, relationship: &ChildrenRelationShip) -> bool {
    let mut parents = BitSet::<usize>::default();
    for child in list {
        if let Some(children) = relationship.get(child) {
            if !children.is_disjoint(&parents) {
                return false;
            }
        }
        parents.insert(*child);
    }

    true
}

fn parse_input(input: &str) -> Result<(ChildrenRelationShip, Vec<Vec<usize>>), Error> {
    let (left, right) = input.split_once("\n\n").ok_or_else(|| {
        Error::ParseError("Failed to parse day 5 input: no empty line delimiter".into())
    })?;
    Ok((parse_relationship(left)?, parse_list(right)?))
}

fn parse_relationship(input: &str) -> Result<ChildrenRelationShip, Error> {
    input.lines().map(parse_relationship_line).try_fold(
        HashMap::<usize, BitSet<usize>>::new(),
        |mut map, parent_child| {
            let (parent, child) = parent_child?;
            map.entry(parent).or_default().insert(child);
            Ok(map)
        },
    )
}

fn parse_relationship_line(line: &str) -> Result<(usize, usize), Error> {
    let (left, right) = line.split_once("|").ok_or_else(|| {
        Error::ParseError(format!("Failed to parse {:?}: no | delimiter", line).into())
    })?;
    let left = usize::from_str(left).map_err(|e| {
        Error::ParseError(format!("Failed to parse ures: {:?}: {}", line, e).into())
    })?;
    let right = usize::from_str(right).map_err(|e| {
        Error::ParseError(format!("Failed to parse ures: {:?}: {}", line, e).into())
    })?;
    Ok((left, right))
}

fn parse_list(input: &str) -> Result<Vec<Vec<usize>>, Error> {
    input
        .lines()
        .map(parse_list_line)
        .try_fold(Vec::new(), |mut vec, val| {
            vec.push(val?);
            Ok(vec)
        })
}

fn parse_list_line(line: &str) -> Result<Vec<usize>, Error> {
    line.split(",")
        .map(|val| {
            usize::from_str(val).map_err(|e| {
                Error::ParseError(format!("Failed to parse: {:?}: {}", line, e).into())
            })
        })
        .try_fold(Vec::new(), |mut vec, val| {
            vec.push(val?);
            Ok(vec)
        })
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::utils::tests_utils::{get_input, human_text_duration};
    use chrono::Utc;

    #[test]
    pub fn part1() -> Result<(), Error> {
        let start = Utc::now();
        let input = get_input(2024, 5)?;
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
        let input = get_input(2024, 5)?;
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
