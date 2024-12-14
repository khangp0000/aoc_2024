use crate::error::{Error, NomError};
use crate::nom::{ires, single_line, trim_space, FinalParse};
use crate::part_solver;
use crate::utils::{ires, ures};
use nom::character::complete::space1;
use nom::multi::{fold_many0, many0};
use nom::sequence::separated_pair;
use nom::IResult;
use nom::Parser;
use nom_supreme::tag::complete::tag;
use nom_supreme::ParserExt;
use std::cmp::Ordering;

part_solver!();

type RobotPos = (ires, ires);
type RobotVec = (ires, ires);

pub fn part1(input: &str) -> Result<ures, Error> {
    let res = input_parser_and_processor(101, 103, 100).final_parse(input)?;
    Ok(res)
}

pub fn part2(input: &str) -> Result<ures, Error> {
    let mut robots = parse_robot_coord_and_vec_list.final_parse(input)?;
    let mut step_count = 0;
    let mut board;
    loop {
        step_count += 1;
        step(&mut robots, 101, 103);
        board = create_board::<101, 103>(&robots);
        if maybe_has_tree(&board) {
            break;
        }

        if step_count > 10000 {
            return Err(Error::Unsolvable(
                "cannot find the christmas tree after 10000 step".into(),
            ));
        }
    }

    Ok(step_count)
}

fn maybe_has_tree<const W: usize, const H: usize>(board: &[[bool; W]; H]) -> bool {
    static TREE_LINE_1: &[bool] = &[
        false, false, true, true, true, true, true, true, true, true, true, true, true, true, true,
        true, true, true, true, false, false,
    ];
    static TREE_LINE_2: &[bool] = &[true; 21];

    for (idx, line) in board.iter().enumerate() {
        if let Some(pos) = find_subsequence(line, TREE_LINE_1) {
            if Some(pos)
                == board
                    .get(idx + 5)
                    .and_then(|l| find_subsequence(l, TREE_LINE_2))
            {
                return true;
            }
        }
    }

    false
}

fn input_parser_and_processor<'a>(
    width: ires,
    height: ires,
    movement_times: ires,
) -> impl Parser<&'a str, ures, NomError<'a>> {
    fold_many0(
        single_line(trim_space(parse_robot_coord_and_vec)),
        || [0 as ures; 4],
        move |mut count, (pos, vec)| {
            let final_pos = process_pos_vec(&pos, &vec, width, height, movement_times);
            if let Some(quadrant) = get_quadrant(&final_pos, width, height) {
                count[quadrant] += 1;
            }
            count
        },
    )
    .map(|count| count.iter().product())
}

fn parse_ires_pair(input: &str) -> IResult<&str, (ires, ires), NomError> {
    separated_pair(ires, tag(","), ires)
        .context("parse coord")
        .parse(input)
}

fn parse_robot_coord_and_vec(input: &str) -> IResult<&str, (RobotPos, RobotVec), NomError> {
    separated_pair(
        tag("p=")
            .precedes(parse_ires_pair)
            .context("parse position"),
        space1,
        tag("v=").precedes(parse_ires_pair).context("parse vector"),
    )
    .context("parse robot position and movement")
    .parse(input)
}

fn parse_robot_coord_and_vec_list(
    input: &str,
) -> IResult<&str, Vec<(RobotPos, RobotVec)>, NomError> {
    many0(single_line(trim_space(parse_robot_coord_and_vec)))
        .context("parse list of robot")
        .parse(input)
}

fn step(robots: &mut [(RobotPos, RobotVec)], width: ires, height: ires) {
    robots
        .iter_mut()
        .for_each(|((pos_x, pos_y), (vec_x, vec_y))| {
            *pos_x += *vec_x;
            *pos_x %= width;
            if *pos_x < 0 {
                *pos_x += width;
            }

            *pos_y += *vec_y;
            *pos_y %= height;
            if *pos_y < 0 {
                *pos_y += height;
            }
        })
}

fn create_board<const W: usize, const H: usize>(
    robots: &Vec<(RobotPos, RobotVec)>,
) -> [[bool; W]; H] {
    let mut res = [[false; W]; H];
    for ((x, y), _) in robots {
        res[*y as usize][*x as usize] = true;
    }

    res
}

fn process_pos_vec(
    pos: &(ires, ires),
    vec: &(ires, ires),
    width: ires,
    height: ires,
    movement_times: ires,
) -> (ires, ires) {
    let (x, y) = pos;
    let (dx, dy) = vec;
    let mut final_x = ((x % width) + ((dx % width) * (movement_times % width))) % width;
    if final_x < 0 {
        final_x += width;
    }
    let mut final_y = ((y % height) + ((dy % height) * (movement_times % height))) % height;
    if final_y < 0 {
        final_y += height;
    }

    (final_x, final_y)
}

fn get_quadrant(pos: &(ires, ires), width: ires, height: ires) -> Option<usize> {
    let &(x, y) = pos;
    let mut quadrant = 0;
    match x.cmp(&(width / 2)) {
        Ordering::Less => {}
        Ordering::Equal => {
            return None;
        }
        Ordering::Greater => {
            quadrant += 1;
        }
    }
    match y.cmp(&(height / 2)) {
        Ordering::Less => {}
        Ordering::Equal => {
            return None;
        }
        Ordering::Greater => {
            quadrant += 2;
        }
    }

    Some(quadrant)
}

fn find_subsequence<T>(haystack: &[T], needle: &[T]) -> Option<usize>
where
    for<'a> &'a [T]: PartialEq,
{
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

#[cfg(test)]
mod tests {
    use crate::error::Error;

    use crate::utils::tests_utils::{get_input, human_text_duration};

    use chrono::Utc;

    #[test]
    pub fn part1() -> Result<(), Error> {
        let start = Utc::now();
        let input = get_input(2024, 14)?;
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
        let input = get_input(2024, 14)?;
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
