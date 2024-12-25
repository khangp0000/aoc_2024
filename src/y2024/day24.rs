use crate::error::{Error, NomError};
use crate::nom::{fold_res_many1, single_line, single_line_not_eof, FinalParse};
use crate::part_solver;
use crate::utils::ures;
use nom::branch::alt;
use nom::bytes::complete::take_while_m_n;
use nom::character::complete::{char, space0, u8};
use nom::combinator::{map_parser, value};
use nom::multi::{fold_many1, many1};
use nom::sequence::{separated_pair, tuple};
use nom::IResult;
use nom::Parser;
use nom_supreme::tag::complete::tag;
use nom_supreme::ParserExt;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

part_solver!();

type WireToChildGateMap<'a> = HashMap<Wire<'a>, Vec<Rc<Gate<'a>>>>;

pub fn part1(_input: &str) -> Result<ures, Error> {
    let (mut processed, children) = parse_input.final_parse(_input)?;
    let mut work = processed.keys().copied().collect::<Vec<_>>();
    while let Some(wire) = work.pop() {
        if let Some(children) = children.get(&wire) {
            for gate in children {
                if let Some(&val1) = processed.get(&gate.input_1) {
                    if let Some(&val2) = processed.get(&gate.input_2) {
                        let compute = gate.op.process(val1, val2);
                        match processed.insert(gate.output, compute) {
                            Some(inserted) => {
                                if inserted != compute {
                                    return Err(Error::InvalidState(
                                        format!("wire {} have multiple input gate", gate.output)
                                            .into(),
                                    ));
                                }
                            }
                            None => work.push(gate.output),
                        }
                    }
                }
            }
        }
    }

    let mut v = Vec::from_iter(processed.into_iter().filter_map(|(node, val)| match node {
        Wire::Z(idx) => Some((idx, val)),
        _ => None,
    }));
    v.sort_unstable();
    let res = v
        .into_iter()
        .rev()
        .fold(0, |res, (_, bit)| (res << 1) | if bit { 1 } else { 0 });
    Ok(res)
}

pub fn part2(input: &str) -> Result<String, Error> {
    let (gates, children) = parse_input_2.final_parse(input)?;
    let highest_z = gates
        .iter()
        .map(|g| g.output)
        .filter_map(|w| match w {
            Wire::Z(val) => Some(val),
            _ => None,
        })
        .max()
        .ok_or_else(|| Error::ParseError("no z output found".into()))?;
    let mut wrong_output = HashSet::new();
    for gate in gates {
        let (&input_1, &input_2, &op, &output) =
            (&gate.input_1, &gate.input_2, &gate.op, &gate.output);
        if let Wire::Z(val) = output {
            if (val != highest_z && op != Op::Xor)
                || (val == highest_z && op != Op::Or)
                || children
                    .get(&output)
                    .is_some_and(|children| !children.is_empty())
            {
                wrong_output.insert(output);
            }
        } else {
            match gate.op {
                Op::And => {
                    if (input_1 != Wire::X(0) || input_2 != Wire::Y(0))
                        && children
                            .get(&output)
                            .is_none_or(|children| children.iter().any(|g| g.op != Op::Or))
                    {
                        wrong_output.insert(output);
                    }
                }
                Op::Or => {}
                Op::Xor => {
                    if (matches!(input_1, Wire::Conn(_)) && matches!(input_2, Wire::Conn(_)))
                        || children
                            .get(&output)
                            .is_none_or(|children| children.iter().any(|g| g.op == Op::Or))
                    {
                        wrong_output.insert(output);
                    }
                }
            }
        }
    }

    if wrong_output.len() != 8 {
        Err(Error::Unsolvable(
            "this heuristic solution does not work for this input".into(),
        ))
    } else {
        let mut res_vec = wrong_output
            .into_iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>();
        res_vec.sort();
        Ok(res_vec.join(","))
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct Gate<'a> {
    input_1: Wire<'a>,
    input_2: Wire<'a>,
    output: Wire<'a>,
    op: Op,
}

impl<'a> Gate<'a> {
    fn new(input_1: Wire<'a>, input_2: Wire<'a>, output: Wire<'a>, op: Op) -> Self {
        if input_1 > input_2 {
            Self {
                input_1: input_2,
                input_2: input_1,
                output,
                op,
            }
        } else {
            Self {
                input_1,
                input_2,
                output,
                op,
            }
        }
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
enum Wire<'a> {
    X(u8),
    Y(u8),
    Z(u8),
    Conn(&'a str),
}

impl Display for Wire<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Wire::X(v) => {
                write!(f, "x{:02}", v)
            }
            Wire::Y(v) => {
                write!(f, "y{:02}", v)
            }
            Wire::Z(v) => {
                write!(f, "z{:02}", v)
            }
            Wire::Conn(s) => {
                write!(f, "{}", s)
            }
        }
    }
}

impl Debug for Wire<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.to_string())
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
enum Op {
    And,
    Or,
    Xor,
}

impl Op {
    fn process(&self, in_1: bool, in_2: bool) -> bool {
        match self {
            Op::And => in_1 & in_2,
            Op::Or => in_1 | in_2,
            Op::Xor => in_1 ^ in_2,
        }
    }
}

fn parse_x(input: &str) -> IResult<&str, Wire, NomError> {
    map_parser(
        char('x').precedes(take_while_m_n(2, 2, |c: char| c.is_ascii_digit())),
        u8,
    )
    .map(Wire::X)
    .parse(input)
}

fn parse_y(input: &str) -> IResult<&str, Wire, NomError> {
    map_parser(
        char('y').precedes(take_while_m_n(2, 2, |c: char| c.is_ascii_digit())),
        u8,
    )
    .map(Wire::Y)
    .parse(input)
}

fn parse_z(input: &str) -> IResult<&str, Wire, NomError> {
    map_parser(
        char('z').precedes(take_while_m_n(2, 2, |c: char| c.is_ascii_digit())),
        u8,
    )
    .map(Wire::Z)
    .parse(input)
}

fn parse_conn(input: &str) -> IResult<&str, Wire, NomError> {
    take_while_m_n(3, 3, |c: char| c.is_ascii_alphabetic())
        .map(Wire::Conn)
        .parse(input)
}

fn parse_wire(input: &str) -> IResult<&str, Wire, NomError> {
    alt((parse_x, parse_y, parse_z, parse_conn))
        .context("parse wire")
        .parse(input)
}

fn parse_op(input: &str) -> IResult<&str, Op, NomError> {
    alt((
        tag("AND").value(Op::And),
        tag("OR").value(Op::Or),
        tag("XOR").value(Op::Xor),
    ))
    .context("parse op")
    .parse(input)
}

fn parse_node(input: &str) -> IResult<&str, (Wire, bool), NomError> {
    separated_pair(
        parse_wire,
        tag(": "),
        alt((value(true, char('1')), value(false, char('0')))),
    )
    .context("parse node")
    .parse(input)
}

fn parse_gate(input: &str) -> IResult<&str, Gate, NomError> {
    tuple((
        parse_wire,
        parse_op.preceded_by(char(' ')),
        parse_wire.preceded_by(char(' ')),
        parse_wire.preceded_by(tag(" -> ")),
    ))
    .map(|(in1, op, in2, out)| Gate::new(in1, in2, out, op))
    .context("parse gate")
    .parse(input)
}

fn parse_input(input: &str) -> IResult<&str, (HashMap<Wire, bool>, WireToChildGateMap), NomError> {
    separated_pair(
        fold_res_many1(
            single_line_not_eof(parse_node),
            HashMap::new,
            |mut inputs, (wire, val)| {
                if Some(!val) == inputs.insert(wire, val) {
                    Err((
                        inputs,
                        None,
                        nom::Err::Failure(Error::ParseError(
                            format!("multiple input value for {}", wire).into(),
                        )),
                    ))
                } else {
                    Ok(inputs)
                }
            },
        ),
        single_line(space0),
        fold_many1(
            single_line(parse_gate),
            HashMap::new,
            |mut children: HashMap<Wire<'_>, Vec<Rc<_>>>, gate: Gate| {
                let wire_1 = gate.input_1;
                let wire_2 = gate.input_2;
                let gate = Rc::new(gate);
                children.entry(wire_1).or_default().push(gate.clone());
                children.entry(wire_2).or_default().push(gate);
                children
            },
        ),
    )
    .parse(input)
}

fn parse_input_2(input: &str) -> IResult<&str, (Vec<Rc<Gate>>, WireToChildGateMap), NomError> {
    many1(single_line_not_eof(parse_node))
        .precedes(single_line(space0))
        .precedes(fold_many1(
            single_line(parse_gate),
            || (Vec::new(), HashMap::new()),
            |(mut ops, mut children), gate: Gate| {
                let wire_1 = gate.input_1;
                let wire_2 = gate.input_2;
                let gate = Rc::new(gate);
                children
                    .entry(wire_1)
                    .or_insert_with(Vec::new)
                    .push(gate.clone());
                children
                    .entry(wire_2)
                    .or_insert_with(Vec::new)
                    .push(gate.clone());
                ops.push(gate);
                (ops, children)
            },
        ))
        .parse(input)
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::utils::tests_utils::{get_input, human_text_duration};
    use chrono::Utc;

    #[test]
    pub fn part1() -> Result<(), Error> {
        let start = Utc::now();
        let input = get_input(2024, 24)?;
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
        let input = get_input(2024, 24)?;
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
