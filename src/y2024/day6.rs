use crate::error::Error;
use crate::part_solver;
use crate::space::{Board2d, Space};
use crate::utils::ures;
use std::cmp::PartialEq;

part_solver!();

#[derive(Copy, Clone, PartialEq, Eq)]
enum Direction {
    Up,
    Right,
    Down,
    Left,
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum Mask {
    Empty = 0,
    Up = 1,
    Right = 2,
    Down = 4,
    Left = 8,
    Wall = 16,
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum Status {
    OkFirst,
    OkRepeat,
    Cycle,
    Done,
}

impl Direction {
    #[inline]
    const fn rotate(&self) -> Direction {
        match self {
            Direction::Up => Direction::Right,
            Direction::Right => Direction::Down,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
        }
    }

    #[inline]
    fn rotate_self(&mut self) {
        *self = self.rotate();
    }

    #[inline]
    const fn mask(&self) -> Mask {
        match self {
            Direction::Up => Mask::Up,
            Direction::Right => Mask::Right,
            Direction::Down => Mask::Down,
            Direction::Left => Mask::Left,
        }
    }

    #[inline]
    const fn from_mask_u8(val: u8) -> Result<Direction, u8> {
        match val {
            1 => Ok(Direction::Up),
            2 => Ok(Direction::Right),
            4 => Ok(Direction::Down),
            8 => Ok(Direction::Left),
            _ => Err(val),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
struct Guard {
    coord: [usize; 2],
    facing: Direction,
}

impl Guard {
    fn step(&mut self, board: &mut Board2d<u8>) -> Status {
        let orig_x = self.coord[0];
        let orig_y = self.coord[1];
        match self.facing {
            Direction::Up => {
                self.coord[1] = orig_y.wrapping_sub(1);
            }
            Direction::Right => {
                self.coord[0] += 1;
            }
            Direction::Down => {
                self.coord[1] += 1;
            }
            Direction::Left => {
                self.coord[0] = orig_x.wrapping_sub(1);
            }
        }

        let pos_mask = board.get(&self.coord);
        match pos_mask {
            None => Status::Done,
            Some(&pos_mask) => {
                if pos_mask == Mask::Wall as u8 {
                    match self.facing {
                        Direction::Up => {
                            self.coord[1] = orig_y;
                        }
                        Direction::Right => {
                            self.coord[0] = orig_x;
                        }
                        Direction::Down => {
                            self.coord[1] = orig_y;
                        }
                        Direction::Left => {
                            self.coord[0] = orig_x;
                        }
                    }

                    self.facing.rotate_self();
                }

                let val = board.get_mut(&self.coord).unwrap();
                let mask = self.facing.mask() as u8;
                if (*val & mask) != 0 {
                    Status::Cycle
                } else {
                    *val |= mask;
                    if *val == mask {
                        Status::OkFirst
                    } else {
                        Status::OkRepeat
                    }
                }
            }
        }
    }
}

pub fn part1(input: &str) -> Result<ures, Error> {
    let (mut board, mut guard) = parse_input(input)?;
    loop {
        let status = guard.step(&mut board);
        if status != Status::OkRepeat && status != Status::OkFirst {
            break;
        }
    }

    let res = board
        .as_ref()
        .iter()
        .map(|v| v.iter().filter(|&&val| val > 0 && val < 16).count() as ures)
        .sum();

    Ok(res)
}

pub fn part2(input: &str) -> Result<ures, Error> {
    let (mut board, mut guard) = parse_input(input)?;
    let mut sum = 0;
    let mut prev = guard;
    loop {
        guard = prev;
        let status = guard.step(&mut board);
        if status != Status::OkRepeat && status != Status::OkFirst {
            break;
        }
        if status == Status::OkFirst {
            let mut board = board.clone();
            *board.get_mut(&guard.coord).unwrap() = Mask::Wall as u8;
            if has_loop(&mut prev, &mut board) {
                sum += 1;
            }
        }
        prev = guard;
    }

    Ok(sum)
}

fn has_loop(guard: &mut Guard, board: &mut Board2d<u8>) -> bool {
    loop {
        let status = guard.step(board);
        match status {
            Status::OkFirst => {}
            Status::OkRepeat => {}
            Status::Cycle => return true,
            Status::Done => return false,
        }
    }
}

fn parse_input(input: &str) -> Result<(Board2d<u8>, Guard), Error> {
    let (board, start) = input
        .lines()
        .map(|line| {
            let line_bytes = line.bytes();
            let len = line_bytes.len();
            line_bytes.map(parse_byte).enumerate().try_fold(
                (Vec::with_capacity(len), None),
                |(mut vec, mut x), (index, b)| {
                    let b = b?;
                    if b != Mask::Wall as u8 && b != Mask::Empty as u8 && x.replace(index).is_some() {
                        return Err(Error::ParseError(
                            "more than 1 starting position".to_string(),
                        ));
                    }
                    vec.push(b);
                    Ok((vec, x))
                },
            )
        })
        .enumerate()
        .try_fold((Vec::new(), None), |(mut board, mut start), (y, line)| {
            let (board_line, x) = line?;
            if let Some(x) = x {
                if start.replace((x, y)).is_some() {
                    return Err(Error::ParseError(
                        "more than 1 starting position".to_string(),
                    ));
                }
            }
            board.push(board_line);
            Ok((board, start))
        })?;
    let board: Board2d<u8> = board.into();
    let coord: [usize; 2] = start
        .ok_or_else(|| Error::ParseError("starting position not found".to_string()))?
        .into();
    let direction =
        Direction::from_mask_u8(*board.get(&coord).ok_or_else(|| {
            Error::InvalidState("cannot get value at start position".to_string())
        })?)
        .map_err(|val| Error::ParseError(format!("invalid u8 value for direction: {}", val)))?;
    let guard = Guard {
        coord,
        facing: direction,
    };

    Ok((board, guard))
}

fn parse_byte(b: u8) -> Result<u8, Error> {
    match b {
        b'^' => Ok(Mask::Up as u8),
        b'>' => Ok(Mask::Right as u8),
        b'v' => Ok(Mask::Down as u8),
        b'<' => Ok(Mask::Left as u8),
        b'#' => Ok(Mask::Wall as u8),
        b'.' => Ok(Mask::Empty as u8),
        _ => Err(Error::ParseError(format!(
            "Invalid character: {:?}",
            b as char
        ))),
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
        let input = get_input(2024, 6)?;
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
        let input = get_input(2024, 6)?;
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
