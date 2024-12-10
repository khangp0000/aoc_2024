use crate::error::Error;
use crate::utils::{check_valid_question, get_input, submit, DisplayDebug};
use clap::{ArgAction, Parser};
use dotenv::dotenv;
use std::env;
use std::process::exit;
use std::time::Duration;

mod error;
mod space;
mod utils;
mod y2024;

/// Simple Advent of Code solver
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// year of advent of code
    #[arg(short, long, required = true)]
    year: u16,

    /// day of year, 1 to 25
    #[arg(short, long, requires = "year")]
    day: Option<u8>,

    /// part of day, 1 or 2
    #[arg(short, long, requires = "day")]
    part: Option<u8>,

    /// try to run submission. any failure will disable submission of next part in same day
    #[arg(short, long, default_value_t = false, default_missing_value = "true", num_args=0..=1, action = ArgAction::Set)]
    submit: bool,

    /// if want to exit on failure without running subsequence day
    #[arg(short, long, default_value_t = false, default_missing_value = "true", num_args=0..=1, action = ArgAction::Set)]
    exit_on_failure: bool,
}

fn main() {
    let args = Args::parse();
    let _ = dotenv();
    let session = env::var("SESSION_COOKIE").expect("Missing cookie \"SESSION_COOKIE\"");
    let (year, day, part) = (args.year, args.day, args.part);
    let day_range = check_valid_question(year, day);

    let day_range = match day_range {
        Ok(dr) => dr,
        Err(e) => {
            eprintln!("Cannot retrieve aoc problem: {}", e);
            exit(1);
        }
    };

    let part_range = if let Some(part) = part {
        part..=part
    } else {
        1..=2
    };

    let mut error_code = 0;
    'outer: for day in day_range {
        let mut submit = args.submit;
        for part in part_range.clone() {
            let res = solve_and_print_result(year, day, part, session.as_str());
            if res.is_err() {
                error_code = 1;
                if args.exit_on_failure {
                    eprintln!("Exit early on error: {:?}", res);
                    break;
                }
            }
            let mut retry = Some(Duration::from_secs(0));
            while submit && retry.is_some() {
                std::thread::sleep(retry.take().unwrap());
                let res = res
                    .as_ref()
                    .map_err(|e| e.clone())
                    .and_then(|res| submit_result(year, day, part, res, session.as_str()));
                match &res {
                    Ok(_) => {
                        println!(
                            "Submission successful for {} day {} part {}",
                            year, day, part
                        );
                    }
                    Err(e) => match e {
                        Error::UtilsError(utils::UtilsError::AlreadySubmitted(_)) => {
                            println!(
                                "Submission is previously done for {} day {} part {}",
                                year, day, part
                            );
                        }
                        Error::UtilsError(utils::UtilsError::SubmissionThrottled(_, duration)) => {
                            if let Some(duration) = duration {
                                println!(
                                        "Submission for {} day {} part {} is throttled: {}. Waiting for throttle to finish...",
                                        year, day, part, humantime::format_duration(*duration)
                                    );
                                retry = Some(*duration);
                            } else {
                                error_code = 1;
                                eprintln!(
                                    "Disabling submission due to previous submission error: {:?}",
                                    res
                                );
                                submit = false;
                                if args.exit_on_failure {
                                    eprintln!("Exit early on error: {:?}", res);
                                    break 'outer;
                                }
                            }
                        }
                        _ => {
                            error_code = 1;
                            eprintln!(
                                "Disabling submission due to previous submission error: {:?}",
                                res
                            );
                            submit = false;
                            if args.exit_on_failure {
                                eprintln!("Exit early on error: {:?}", res);
                                break 'outer;
                            }
                        }
                    },
                }
            }
        }
    }

    exit(error_code);
}

fn solve_and_print_result(
    year: u16,
    day: u8,
    part: u8,
    session: &str,
) -> Result<Box<dyn DisplayDebug>, Error> {
    let res = get_input(year, day, session)
        .map_err(|e| e.into())
        .and_then(|input| utils::solve(year, day, part, input.as_str()));

    println!("Result {} day {} part {}: {:?}", year, day, part, res);
    res
}

fn submit_result(
    year: u16,
    day: u8,
    part: u8,
    answer: &Box<dyn DisplayDebug>,
    session: &str,
) -> Result<(), Error> {
    Ok(submit(year, day, part, answer, session)?)
}
