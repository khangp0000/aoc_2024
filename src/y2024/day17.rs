use crate::error::{Error, NomError};
use crate::nom::{single_line, single_line_not_eof, ures, FinalParse};
use crate::part_solver;
use crate::utils::ures;
use nom::bytes::complete::tag;
use nom::character::complete::{anychar, char, space0};
use nom::multi::separated_list0;
use nom::sequence::{pair, tuple};
use nom::{IResult, Parser};
use nom_supreme::ParserExt;
use std::fmt::Write;

part_solver!();

#[derive(Debug)]
struct Machine {
    reg: [ures; 3],
    instruction_counter: u8,
}

impl Machine {
    fn step(&mut self, instruction: &[u8]) -> Result<Option<u8>, Error> {
        if let Some(opcode) = instruction.get(self.instruction_counter as usize) {
            if let Some(operand) = instruction.get(self.instruction_counter as usize + 1) {
                match opcode {
                    0 => self.reg[0] >>= self.combo(*operand)?,
                    1 => self.reg[1] ^= *operand as ures,
                    2 => self.reg[1] = self.combo(*operand)? & 7,
                    3 => {
                        if self.reg[0] != 0 {
                            self.instruction_counter = *operand;
                            return Ok(None);
                        }
                    }
                    4 => self.reg[1] ^= self.reg[2],
                    5 => {
                        self.instruction_counter += 2;
                        return Ok(Some((self.combo(*operand)? & 7) as u8));
                    }
                    6 => self.reg[1] = self.reg[0] >> self.combo(*operand)?,
                    7 => self.reg[2] = self.reg[0] >> self.combo(*operand)?,
                    _ => {
                        return Err(Error::InvalidState(
                            format!("invalid opcode {}", opcode).into(),
                        ))
                    }
                }
            } else {
                return Err(Error::InvalidState(
                    format!(
                        "expected operand but reached the end at index {}",
                        self.instruction_counter as usize + 1
                    )
                    .into(),
                ));
            }
        } else {
            return Err(Error::InvalidState(
                format!(
                    "expected opcode but reached the end at index {}",
                    self.instruction_counter as usize
                )
                .into(),
            ));
        }

        self.instruction_counter += 2;
        Ok(None)
    }

    fn step_til_end(&mut self, instruction: &[u8]) -> Result<Vec<u8>, Error> {
        let mut res = Vec::new();
        let mut count = 0;
        while (self.instruction_counter as usize) < instruction.len() {
            if let Some(val) = self.step(instruction)? {
                res.push(val);
            }
            count += 1;
            if count > 100 {
                return Err(Error::Unsolvable("program loop more than 100 times".into()));
            }
        }
        Ok(res)
    }

    fn combo(&self, val: u8) -> Result<ures, Error> {
        match val {
            0..=3 => Ok(val as ures),
            4..=6 => Ok(self.reg[(val - 4) as usize]),
            _ => Err(Error::ParseError(format!("invalid combo '{}'", val).into())),
        }
    }
}

pub fn part1(input: &str) -> Result<String, Error> {
    let (mut machine, program) = input_parser.final_parse(input)?;
    let mut iter = machine.step_til_end(&program)?.into_iter();
    let mut s = String::with_capacity(iter.len() * 2);

    if let Some(first) = iter.next() {
        write!(s, "{}", first)
            .map_err(|e| Error::InvalidState(format!("string build error< {}", e).into()))?;
        iter.try_for_each(|v| {
            write!(s, ",{}", v)
                .map_err(|e| Error::InvalidState(format!("string build error< {}", e).into()))
        })?
    }
    Ok(s)
}

pub fn part2(input: &str) -> Result<ures, Error> {
    let (_, program) = input_parser.final_parse(input)?;

    validate_part_2_solvable(&program)?;

    let no_loop_program: Vec<u8> = program.iter().copied().take(program.len() - 2).collect();
    find_a_val_match_program_to_output(0, program.iter().rev(), &no_loop_program)?
        .ok_or_else(|| Error::Unsolvable("cannot find valid a_val".into()))
}

fn find_a_val_match_program_to_output<'a, I: Iterator<Item = &'a u8> + Clone>(
    a_val: ures,
    mut program_rev_iter: I,
    no_loop_program: &[u8],
) -> Result<Option<ures>, Error> {
    if let Some(&next_expected_output) = program_rev_iter.next() {
        let a_val = a_val << 3;
        for a_val in a_val..=(a_val + 7) {
            if a_val == 0 {
                continue;
            }
            let program_rev_iter = program_rev_iter.clone();
            let program_output = Machine {
                reg: [a_val, 0, 0],
                instruction_counter: 0,
            }
            .step_til_end(no_loop_program)?;
            if program_output.len() != 1 {
                return Err(Error::Unsolvable(
                    "no-loop program does not output exactly once".into(),
                ));
            }
            if next_expected_output == program_output[0] {
                let check_next = find_a_val_match_program_to_output(
                    a_val,
                    program_rev_iter.clone(),
                    no_loop_program,
                )?;
                if check_next.is_some() {
                    return Ok(check_next);
                }
            }
        }

        Ok(None)
    } else {
        Ok(Some(a_val))
    }
}

fn validate_part_2_solvable(program: &[u8]) -> Result<(), Error> {
    let mut error = false;
    if program[(program.len() - 2)..] != [3, 0] {
        error = true;
    }

    if !error {
        if let Some(first_a_change_idx) = first_a_change(program) {
            if Some(&3) != program.get(first_a_change_idx + 1)
                || first_a_change(&program[(first_a_change_idx + 2)..]).is_some()
            {
                error = true;
            }
        } else {
            error = true;
        }
    }

    if !error {
        if let Some(first_b_change_idx) = first_b_change(program) {
            if let Some(first_b_use_idx) = first_b_use(program) {
                if first_b_change_idx >= first_b_use_idx {
                    error = true;
                }
            }
        }
    }

    if !error {
        if let Some(first_c_change_idx) = first_c_change(program) {
            if let Some(first_c_use_idx) = first_c_use(program) {
                if first_c_change_idx >= first_c_use_idx {
                    error = true;
                }
            }
        }
    }

    if error {
        Err(Error::Unsolvable("unsolvable part 2".into()))
    } else {
        Ok(())
    }
}

fn first_a_change(program: &[u8]) -> Option<usize> {
    let mut i = 0;
    while let Some(val) = program.get(i) {
        if *val == 0 {
            return Some(i);
        }
        i += 2;
    }
    None
}

fn first_b_change(program: &[u8]) -> Option<usize> {
    let mut i = 0;
    while let Some(val) = program.get(i) {
        if *val == 1 || *val == 2 || *val == 4 || *val == 6 {
            return Some(i);
        }
        i += 2;
    }
    None
}

fn first_c_change(program: &[u8]) -> Option<usize> {
    let mut i = 0;
    while let Some(val) = program.get(i) {
        if *val == 7 {
            return Some(i);
        }
        i += 2;
    }
    None
}

fn first_b_use(program: &[u8]) -> Option<usize> {
    let mut i = 0;
    while let Some(val) = program.get(i) {
        let b_used = match *val {
            0 | 2 | 5 | 6 | 7 => Some(&5) == program.get(i),
            1 | 4 => true,
            _ => false,
        };
        if b_used {
            return Some(i);
        }
        i += 2;
    }
    None
}

fn first_c_use(program: &[u8]) -> Option<usize> {
    let mut i = 0;
    while let Some(val) = program.get(i) {
        let b_used = match *val {
            0 | 2 | 5 | 6 | 7 => Some(&6) == program.get(i),
            4 => true,
            _ => false,
        };
        if b_used {
            return Some(i);
        }
        i += 2;
    }
    None
}

fn register_parser<'a>(c: char) -> impl Parser<&'a str, ures, NomError<'a>> {
    tag("Register ")
        .precedes(char(c))
        .precedes(tag(": "))
        .precedes(ures)
}

fn machine_parser(input: &str) -> IResult<&str, Machine, NomError> {
    tuple((
        single_line_not_eof(register_parser('A')).context("parse register A"),
        single_line_not_eof(register_parser('B')).context("parse register B"),
        single_line_not_eof(register_parser('C')).context("parse register C"),
    ))
    .map(|(a, b, c)| Machine {
        reg: [a, b, c],
        instruction_counter: 0,
    })
    .context("parse machine init state")
    .parse(input)
}

fn program_parser(input: &str) -> IResult<&str, Vec<u8>, NomError> {
    tag("Program: ")
        .precedes(separated_list0(
            char(','),
            anychar
                .map_res(|c| {
                    if ('0'..='7').contains(&c) {
                        Ok(c as u8 - b'0')
                    } else {
                        Err(Error::ParseError("invalid program".into()))
                    }
                })
                .cut(),
        ))
        .context("parse program")
        .parse(input)
}

fn input_parser(input: &str) -> IResult<&str, (Machine, Vec<u8>), NomError> {
    pair(
        machine_parser,
        single_line_not_eof(space0).precedes(single_line(program_parser)),
    )
    .context("parse input")
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
        let input = get_input(2024, 17)?;
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
        let input = get_input(2024, 17)?;
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
