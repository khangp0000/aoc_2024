use crate::error::Error;
use crate::part_solver;
use crate::utils::ures;

part_solver!();

pub fn part1(input: &str) -> Result<ures, Error> {
    let (board, width) = parse_input(input)?;
    let mut sum = 0;
    if width == 0 {
        return Ok(0);
    }

    for y in 0..board.len() {
        for x in 0..width {
            if match_horizontal(x, y, width, &board, "XMAS") {
                sum += 1;
            }
            if match_vertical(x, y, &board, "XMAS") {
                sum += 1;
            }
            if match_diagonal_down_right(x, y, width, &board, "XMAS") {
                sum += 1;
            }
            if match_diagonal_up_right_from_bottom_left(x, y, width, &board, "XMAS") {
                sum += 1;
            }
            if match_horizontal(x, y, width, &board, "SAMX") {
                sum += 1;
            }
            if match_vertical(x, y, &board, "SAMX") {
                sum += 1;
            }
            if match_diagonal_down_right(x, y, width, &board, "SAMX") {
                sum += 1;
            }
            if match_diagonal_up_right_from_bottom_left(x, y, width, &board, "SAMX") {
                sum += 1;
            }
        }
    }

    Ok(sum)
}

pub fn part2(input: &str) -> Result<ures, Error> {
    let (board, width) = parse_input(input)?;
    let mut sum = 0;
    if width == 0 {
        return Ok(0);
    }

    for y in 0..board.len() {
        for x in 0..width {
            let down_right = match_diagonal_down_right(x, y, width, &board, "MAS")
                || match_diagonal_down_right(x, y, width, &board, "SAM");
            if down_right {
                let up_right_from_bottom_left =
                    match_diagonal_up_right_from_bottom_left(x, y, width, &board, "MAS")
                        || match_diagonal_up_right_from_bottom_left(x, y, width, &board, "SAM");

                if up_right_from_bottom_left {
                    sum += 1;
                }
            }
        }
    }

    Ok(sum)
}

// This is full retarded solution..., have function for every type of match :X

fn match_horizontal(x: usize, y: usize, width: usize, board: &[&str], s: &str) -> bool {
    let len = s.len();
    if x + len <= width {
        let mut i = 0;
        while i < len {
            if board[y].as_bytes()[x + i] != s.as_bytes()[i] {
                return false;
            }
            i += 1;
        }
        return true;
    }
    false
}

fn match_vertical(x: usize, y: usize, board: &[&str], s: &str) -> bool {
    let len = s.len();
    if y + len <= board.len() {
        let mut i = 0;
        while i < len {
            if board[y + i].as_bytes()[x] != s.as_bytes()[i] {
                return false;
            }
            i += 1;
        }
        return true;
    }
    false
}

fn match_diagonal_down_right(x: usize, y: usize, width: usize, board: &[&str], s: &str) -> bool {
    let len = s.len();
    if x + len <= width && y + len <= board.len() {
        let mut i = 0;
        while i < len {
            if board[y + i].as_bytes()[x + i] != s.as_bytes()[i] {
                return false;
            }
            i += 1;
        }
        return true;
    }
    false
}

fn match_diagonal_up_right_from_bottom_left(
    x: usize,
    y: usize,
    width: usize,
    board: &[&str],
    s: &str,
) -> bool {
    let len = s.len();
    if x + len <= width && y + len <= board.len() {
        let mut i = 0;
        while i < len {
            if board[y + len - i - 1].as_bytes()[x + i] != s.as_bytes()[i] {
                return false;
            }
            i += 1;
        }
        return true;
    }
    false
}

fn parse_input(input: &str) -> Result<(Vec<&str>, usize), Error> {
    let mut lines = input.lines();
    let mut board = Vec::new();
    if let Some(first_line) = lines.next() {
        board.push(first_line);
        let first_line_len = first_line.len();
        lines
            .try_fold(board, |mut vec, next_line| {
                if next_line.len() != first_line_len {
                    Err(Error::ParseError(
                        "Not all line have same length!".to_string(),
                    ))
                } else {
                    vec.push(next_line);
                    Ok(vec)
                }
            })
            .map(|concat_line| (concat_line, first_line_len))
    } else {
        Ok((board, 0))
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
        let input = get_input(2024, 4)?;
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
        let input = get_input(2024, 4)?;
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
