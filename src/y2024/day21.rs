use crate::error::{Error, NomError};
use crate::graph::MaybeProcessed::Processed;
use crate::graph::{Dijkstra, NeighborFn};
use crate::nom::{single_line, FinalParse};
use crate::part_solver;
use crate::utils::ures;
use nom::error::FromExternalError;
use nom::multi::many1;
use nom::{IResult, Parser};
use std::cmp::PartialEq;
use std::collections::{BinaryHeap, HashSet};
use std::slice::Iter;

part_solver!();

pub fn part1(input: &str) -> Result<ures, Error> {
    inner_solver(input, 2)
}

pub fn part2(input: &str) -> Result<ures, Error> {
    inner_solver(input, 25)
}

fn inner_solver(input: &str, num_robot: u8) -> Result<ures, Error> {
    let cost_table = compute_press_cost_table(num_robot);
    let codes = parse_input.final_parse(input)?;

    let mut res = 0;

    for (code, code_numeric) in codes {
        let min_cost = calculate_min_cost(&cost_table, code)?;
        res += min_cost * code_numeric;
    }

    Ok(res)
}

fn calculate_min_cost<I: Iterator<Item = NumPad>, II: IntoIterator<IntoIter = I, Item = NumPad>>(
    cost_table: &[[ures; 5]; 5],
    code: II,
) -> Result<ures, Error> {
    let mut dijkstra = Dijkstra {
        queue: BinaryHeap::new(),
        neighbor_fn: TargetButtonNeighbor {
            target: NumPad::A,
            cost_table,
        },
        visited: HashSet::new(),
    };

    let mut current = ((NumPad::A, KeyPad::A), 0, ());
    for target in code.into_iter() {
        dijkstra.queue.clear();
        dijkstra.visited.clear();
        dijkstra.neighbor_fn.target = target;
        dijkstra.push_queue(current);
        let mut first = true;

        loop {
            match dijkstra.next() {
                None => return Err(Error::Unsolvable("cannot find path to target".into())),
                Some(Err(e)) => return Err(e),
                Some(Ok(Processed((state, weight, _metadata)))) => {
                    if !first {
                        let (_, key_pad) = state;
                        if key_pad == KeyPad::A {
                            current = (state, weight, ());
                            break;
                        }
                    } else {
                        first = false;
                    }
                }
                _ => continue,
            }
        }
    }

    let (_, min_cost, _) = current;
    Ok(min_cost)
}

fn parse_code(input: &str) -> IResult<&str, (Vec<NumPad>, ures), NomError> {
    let bytes = input.as_bytes();
    let mut res = Vec::with_capacity(4);
    for val in bytes.iter() {
        match NumPad::try_from(*val) {
            Ok(v) => {
                res.push(v);
                if v == NumPad::A {
                    break;
                }
            }
            Err(e) => {
                if res.is_empty() {
                    return Err(nom::Err::Error(NomError::from_external_error(
                        input,
                        nom::error::ErrorKind::Many1,
                        e,
                    )));
                }
                break;
            }
        }
    }
    if res.len() < 2 || res[res.len() - 1] != NumPad::A {
        return Err(nom::Err::Error(NomError::from_external_error(
            input,
            nom::error::ErrorKind::OneOf,
            Error::ParseError("expect one or more digit followed by A".into()),
        )));
    }
    let mut code_numeric = 0;
    for b in bytes[0..res.len() - 1].iter() {
        code_numeric *= 10;
        code_numeric += (*b - b'0') as ures;
    }

    Ok((&input[res.len()..], (res, code_numeric)))
}

fn parse_input(input: &str) -> IResult<&str, Vec<(Vec<NumPad>, ures)>, NomError> {
    many1(single_line(parse_code)).parse(input)
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
enum KeyPad {
    A = 0,
    Left = 1,
    Up = 2,
    Right = 3,
    Down = 4,
}

impl KeyPad {
    pub fn iter() -> Iter<'static, KeyPad> {
        static KEY_PAD_ITER: [KeyPad; 5] = [
            KeyPad::A,
            KeyPad::Left,
            KeyPad::Up,
            KeyPad::Right,
            KeyPad::Down,
        ];
        KEY_PAD_ITER.iter()
    }
}

static BEST_SEQUENCE_TO_PRESS_END_FROM_START: [[&[&[KeyPad]]; 5]; 5] = [
    // A
    [
        // A -> A
        &[&[KeyPad::A]],
        // A -> Left
        &[
            &[KeyPad::Left, KeyPad::Down, KeyPad::Left, KeyPad::A],
            &[KeyPad::Down, KeyPad::Left, KeyPad::Left, KeyPad::A],
        ],
        // A -> Up
        &[&[KeyPad::Left, KeyPad::A]],
        // A -> Right
        &[&[KeyPad::Down, KeyPad::A]],
        // A -> Down
        &[
            &[KeyPad::Left, KeyPad::Down, KeyPad::A],
            &[KeyPad::Down, KeyPad::Left, KeyPad::A],
        ],
    ],
    // Left
    [
        // Left -> A
        &[
            &[KeyPad::Right, KeyPad::Up, KeyPad::Right, KeyPad::A],
            &[KeyPad::Right, KeyPad::Right, KeyPad::Up, KeyPad::A],
        ],
        // Left -> Left
        &[&[KeyPad::A]],
        // Left -> Up
        &[&[KeyPad::Right, KeyPad::Up, KeyPad::A]],
        // Left -> Right
        &[&[KeyPad::Right, KeyPad::Right, KeyPad::A]],
        // Left -> Down
        &[&[KeyPad::Right, KeyPad::A]],
    ],
    // Up
    [
        // Up -> A
        &[&[KeyPad::Right, KeyPad::A]],
        // Up -> Left
        &[&[KeyPad::Down, KeyPad::Left, KeyPad::A]],
        // Up -> Up
        &[&[KeyPad::A]],
        // Up -> Right
        &[
            &[KeyPad::Down, KeyPad::Right, KeyPad::A],
            &[KeyPad::Right, KeyPad::Down, KeyPad::A],
        ],
        // Up -> Down
        &[&[KeyPad::Down, KeyPad::A]],
    ],
    // Right
    [
        // Right -> A
        &[&[KeyPad::Up, KeyPad::A]],
        // Right -> Left
        &[&[KeyPad::Left, KeyPad::Left, KeyPad::A]],
        // Right -> Up
        &[
            &[KeyPad::Left, KeyPad::Up, KeyPad::A],
            &[KeyPad::Up, KeyPad::Left, KeyPad::A],
        ],
        // Right -> Right
        &[&[KeyPad::A]],
        // Right -> Down
        &[&[KeyPad::Left, KeyPad::A]],
    ],
    // Down
    [
        // Down -> A
        &[
            &[KeyPad::Up, KeyPad::Right, KeyPad::A],
            &[KeyPad::Right, KeyPad::Up, KeyPad::A],
        ],
        // Down -> Left
        &[&[KeyPad::Left, KeyPad::A]],
        // Down -> Up
        &[&[KeyPad::Up, KeyPad::A]],
        // Down -> Right
        &[&[KeyPad::Right, KeyPad::A]],
        // Down -> Down
        &[&[KeyPad::A]],
    ],
];

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
enum NumPad {
    A,
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
}

impl TryFrom<u8> for NumPad {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            b'A' => Ok(Self::A),
            b'0' => Ok(Self::Zero),
            b'1' => Ok(Self::One),
            b'2' => Ok(Self::Two),
            b'3' => Ok(Self::Three),
            b'4' => Ok(Self::Four),
            b'5' => Ok(Self::Five),
            b'6' => Ok(Self::Six),
            b'7' => Ok(Self::Seven),
            b'8' => Ok(Self::Eight),
            b'9' => Ok(Self::Nine),
            value => Err(Error::ParseError(
                format!("invalid numpad character {:?}", value as char).into(),
            )),
        }
    }
}

impl NumPad {
    const fn get_neighbor(&self) -> &'static [State] {
        match self {
            NumPad::A => &[(Self::Zero, KeyPad::Left), (Self::Three, KeyPad::Up)],
            NumPad::Zero => &[(Self::Two, KeyPad::Up), (Self::A, KeyPad::Right)],
            NumPad::One => &[(Self::Four, KeyPad::Up), (Self::Two, KeyPad::Right)],
            NumPad::Two => &[
                (Self::One, KeyPad::Left),
                (Self::Five, KeyPad::Up),
                (Self::Three, KeyPad::Right),
                (Self::Zero, KeyPad::Down),
            ],
            NumPad::Three => &[
                (Self::Two, KeyPad::Left),
                (Self::Six, KeyPad::Up),
                (Self::A, KeyPad::Down),
            ],
            NumPad::Four => &[
                (Self::Seven, KeyPad::Up),
                (Self::Five, KeyPad::Right),
                (Self::One, KeyPad::Down),
            ],
            NumPad::Five => &[
                (Self::Four, KeyPad::Left),
                (Self::Eight, KeyPad::Up),
                (Self::Six, KeyPad::Right),
                (Self::Two, KeyPad::Down),
            ],
            NumPad::Six => &[
                (Self::Five, KeyPad::Left),
                (Self::Nine, KeyPad::Up),
                (Self::Three, KeyPad::Down),
            ],
            NumPad::Seven => &[(Self::Eight, KeyPad::Right), (Self::Four, KeyPad::Down)],
            NumPad::Eight => &[
                (Self::Seven, KeyPad::Left),
                (Self::Nine, KeyPad::Right),
                (Self::Five, KeyPad::Down),
            ],
            NumPad::Nine => &[(Self::Eight, KeyPad::Left), (Self::Six, KeyPad::Down)],
        }
    }

    const fn press(&self) -> &'static [State] {
        match self {
            NumPad::A => &[(NumPad::A, KeyPad::A)],
            NumPad::Zero => &[(NumPad::Zero, KeyPad::A)],
            NumPad::One => &[(NumPad::One, KeyPad::A)],
            NumPad::Two => &[(NumPad::Two, KeyPad::A)],
            NumPad::Three => &[(NumPad::Three, KeyPad::A)],
            NumPad::Four => &[(NumPad::Four, KeyPad::A)],
            NumPad::Five => &[(NumPad::Five, KeyPad::A)],
            NumPad::Six => &[(NumPad::Six, KeyPad::A)],
            NumPad::Seven => &[(NumPad::Seven, KeyPad::A)],
            NumPad::Eight => &[(NumPad::Eight, KeyPad::A)],
            NumPad::Nine => &[(NumPad::Nine, KeyPad::A)],
        }
    }
}

trait CostTable {
    fn cost(&self, start: KeyPad, end: KeyPad) -> ures;
}

impl CostTable for [[ures; 5]; 5] {
    fn cost(&self, start: KeyPad, end: KeyPad) -> ures {
        self[start as usize][end as usize]
    }
}

type State = (NumPad, KeyPad);
type Weight = ures;
type Metadata = ();

fn compute_press_cost_table(depth: u8) -> [[ures; 5]; 5] {
    if depth == 0 {
        return [[1; 5]; 5];
    }

    let mut res = [[0; 5]; 5];
    let prev_cost = compute_press_cost_table(depth - 1);
    for start in KeyPad::iter() {
        for end in KeyPad::iter() {
            res[*start as usize][*end as usize] = BEST_SEQUENCE_TO_PRESS_END_FROM_START
                [*start as usize][*end as usize]
                .iter()
                .map(|&sequence| compute_input_sequence_cost(KeyPad::A, sequence, &prev_cost))
                .min()
                .unwrap();
        }
    }
    res
}

struct TargetButtonNeighbor<'a> {
    target: NumPad,
    cost_table: &'a [[ures; 5]; 5],
}

impl NeighborFn<(State, Weight, Metadata)> for TargetButtonNeighbor<'_> {
    fn get_neighbors(
        &mut self,
        swm: &(State, Weight, Metadata),
    ) -> impl IntoIterator<Item = (State, Weight, Metadata)> {
        let &((current_num, current_key), weight, _) = swm;
        let next_state_iter = if current_num == self.target {
            current_num.press().into_iter()
        } else {
            current_num.get_neighbor().into_iter()
        };
        next_state_iter.map(move |next_state| {
            let (_, next_key) = next_state;
            (
                *next_state,
                weight + self.cost_table.cost(current_key, *next_key),
                (),
            )
        })
    }
}

fn compute_input_sequence_cost(
    start: KeyPad,
    sequence: &[KeyPad],
    cost_table: &[[ures; 5]; 5],
) -> ures {
    let (_, cost) = sequence
        .iter()
        .fold((start, 0), |(previous, mut cost), key| {
            cost += cost_table[previous as usize][*key as usize];
            (*key, cost)
        });

    cost
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::utils::tests_utils::{get_input, human_text_duration};
    use chrono::Utc;

    #[test]
    pub fn part1() -> Result<(), Error> {
        let start = Utc::now();
        let input = get_input(2024, 21)?;
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
        let input = get_input(2024, 21)?;
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
