use crate::error::{Error, NomError};
use crate::nom::{fold_separated_res_many1, single_line_not_eof, trim_space, FinalParse};
use crate::part_solver;
use crate::trie::{ArrayTrie, TrieNode};
use crate::utils::ures;
use nom::character::complete::{char, line_ending, multispace0, space0};
use nom::error::ParseError;
use nom::multi::separated_list1;
use nom::sequence::separated_pair;
use nom::{IResult, Parser, Slice};
use nom_supreme::ParserExt;
use std::borrow::Borrow;
use std::iter::Peekable;

part_solver!();

pub fn part1(input: &str) -> Result<ures, Error> {
    let (towels, designs) = input_parser.final_parse(input)?;

    designs
        .into_iter()
        .map(|design| design.collect::<Vec<_>>())
        .map(|d| can_match(d.iter().peekable(), &towels, &towels))
        .try_fold(0, |mut count, res| {
            if res? {
                count += 1;
            }
            Ok(count)
        })
}

pub fn part2(input: &str) -> Result<ures, Error> {
    let (towels, designs) = input_parser.final_parse(input)?;
    designs
        .into_iter()
        .map(|design| {
            match_count(
                design.peekable(),
                0,
                &towels,
                &towels,
                &mut Vec::new(),
                true,
            )
        })
        .try_fold(0, |sum, res| res.map(|v| sum + v))
}

fn parse_word(
    input: &str,
) -> IResult<&str, impl Iterator<Item = usize> + Clone + '_, NomError<'_>> {
    let bytes = input.as_bytes();
    let mut idx = 0;
    for val in bytes.iter() {
        match *val {
            b'w' | b'u' | b'b' | b'r' | b'g' => {}
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

    let res_iter = bytes.slice(0..idx).iter().map(|&b| match b {
        b'b' => 0usize,
        b'g' => 1usize,
        b'r' => 2usize,
        b'u' => 3usize,
        b'w' => 4usize,
        _ => unreachable!(),
    });
    let remaining = input.slice(idx..);

    Ok((remaining, res_iter))
}

fn parse_towels<T: TrieNode<usize>>(input: &str) -> IResult<&str, T, NomError> {
    fold_separated_res_many1(
        char(','),
        trim_space(parse_word),
        || T::default(),
        |mut trie, word| match trie.add(word) {
            Ok(_) => Ok(trie),
            Err(e) => Err((trie, None, nom::Err::Failure(e))),
        },
    )
    .parse(input)
}

fn input_parser(
    input: &str,
) -> IResult<&str, (ArrayTrie<5>, Vec<impl Iterator<Item = usize> + Clone + '_>), NomError> {
    separated_pair(
        single_line_not_eof(parse_towels::<ArrayTrie<5>>),
        single_line_not_eof(space0),
        separated_list1(line_ending, trim_space(parse_word)).terminated(multispace0),
    )
    .parse(input)
}
fn can_match<T, R: Borrow<T> + Clone, I: Iterator<Item = R> + Clone, Trie: TrieNode<T>>(
    mut iter: Peekable<I>,
    trie: &Trie,
    root: &Trie,
) -> Result<bool, Error> {
    match trie.find_prefix(&mut iter)? {
        None => Ok(false),
        Some(node) => {
            if iter.peek().is_none() {
                Ok(true)
            } else {
                Ok(can_match(iter.clone(), root, root)? || can_match(iter.clone(), node, root)?)
            }
        }
    }
}

fn match_count<T, R: Borrow<T> + Clone, I: Iterator<Item = R> + Clone, Trie: TrieNode<T>>(
    mut iter: Peekable<I>,
    pos: usize,
    trie: &Trie,
    root: &Trie,
    cache: &mut Vec<Option<usize>>,
    first: bool,
) -> Result<ures, Error> {
    if first {
        if let Some(Some(v)) = cache.get(pos) {
            return Ok(*v);
        }
    }
    match trie.find_prefix(&mut iter)? {
        None => Ok(0),
        Some(node) => {
            let n_pos = pos + node.depth();
            if iter.peek().is_none() {
                return Ok(1);
            }
            let count = match_count(iter.clone(), n_pos, root, root, cache, true)?
                + match_count(iter.clone(), pos, node, root, cache, false)?;
            if first {
                if pos >= cache.len() {
                    cache.resize(pos + 1, None);
                }
                cache[pos].replace(count);
            }
            Ok(count)
        }
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
        let input = get_input(2024, 19)?;
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
        let input = get_input(2024, 19)?;
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
