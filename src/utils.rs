use crate::error::Error;
use crate::y2024;
use chrono::{Datelike, Utc};
use chrono_tz::US::Eastern;
use dashmap::DashMap;
use regex::Regex;
use reqwest::blocking::Client;
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::ops::RangeInclusive;
use std::rc::Rc;
use std::sync::{Arc, LazyLock, OnceLock};
use std::time::Duration;
use thiserror::Error;

pub trait DisplayDebug: Display + Debug {}

impl<T: Display + Debug + Clone> DisplayDebug for T {}

#[cfg(any(target_pointer_width = "16", target_pointer_width = "32"))]
#[allow(non_camel_case_types)]
pub type ures = u64;
#[cfg(any(target_pointer_width = "16", target_pointer_width = "32"))]
#[allow(non_camel_case_types)]
pub type ires = i64;

#[cfg(not(any(target_pointer_width = "16", target_pointer_width = "32")))]
#[allow(non_camel_case_types)]
pub type ures = usize;
#[cfg(not(any(target_pointer_width = "16", target_pointer_width = "32")))]
#[allow(non_camel_case_types)]
pub type ires = isize;

#[macro_export]
macro_rules! part_solver {
    () => {
        #[inline]
        pub fn solve(
            part: u8,
            input: &str,
        ) -> Result<Box<dyn crate::utils::DisplayDebug>, crate::error::Error> {
            match part {
                1 => part1(input).map(|v| Box::new(v) as Box<dyn crate::utils::DisplayDebug>),
                2 => part2(input).map(|v| Box::new(v) as Box<dyn crate::utils::DisplayDebug>),
                p => Err(crate::error::Error::InvalidState(format!(
                    "solver not found for part: {}",
                    p
                ))),
            }
        }
    };
}

#[macro_export]
macro_rules! day_solver {
    () => {
        #[inline]
        pub fn solve(
            day: u8,
            part: u8,
            input: &str,
        ) -> Result<Box<dyn crate::utils::DisplayDebug>, crate::error::Error> {
            match day {
                1 => day1::solve(part, input),
                2 => day2::solve(part, input),
                3 => day3::solve(part, input),
                4 => day4::solve(part, input),
                5 => day5::solve(part, input),
                6 => day6::solve(part, input),
                7 => day7::solve(part, input),
                8 => day8::solve(part, input),
                9 => day9::solve(part, input),
                10 => day10::solve(part, input),
                11 => day11::solve(part, input),
                12 => day12::solve(part, input),
                13 => day13::solve(part, input),
                14 => day14::solve(part, input),
                15 => day15::solve(part, input),
                16 => day16::solve(part, input),
                17 => day17::solve(part, input),
                18 => day18::solve(part, input),
                19 => day19::solve(part, input),
                20 => day20::solve(part, input),
                21 => day21::solve(part, input),
                22 => day22::solve(part, input),
                23 => day23::solve(part, input),
                24 => day24::solve(part, input),
                25 => day25::solve(part, input),
                d => Err(crate::error::Error::InvalidState(format!(
                    "solver not found for day: {}",
                    d
                ))),
            }
        }
    };
}

#[inline]
pub fn solve(year: u16, day: u8, part: u8, input: &str) -> Result<Box<dyn DisplayDebug>, Error> {
    match year {
        2024 => y2024::solve(day, part, input),
        y => Err(Error::InvalidState(format!(
            "solver not found for year: {}",
            y
        ))),
    }
}

fn default_reqwest_client() -> Client {
    Client::builder()
        .build()
        .expect("problem building the reqwest client")
}

#[derive(Error, Debug, PartialEq, Clone)]
pub enum UtilsError {
    #[error("error with disk cache: `{0}`")]
    DiskCacheError(String),
    #[error("error with reqwest: `{0}`")]
    ReqwestError(String),
    #[error("response status code is not success: `{0}`")]
    ResponseStatusError(String),
    #[error("error retrieve response body as utf8 string: `{0}`")]
    ResponseStringBodyError(String),
    #[error("submitted answer was incorrect: `{0}`")]
    IncorrectAnswer(String),
    #[error("already submitted: `{0}`")]
    AlreadySubmitted(String),
    #[error("submission throttled: `{0}`")]
    SubmissionThrottled(String, Option<Duration>),
    #[error("regex error: `{0}`")]
    RegexError(String),
    #[error("invalid aoc problem: `{0}`")]
    InvalidAOCProblem(String),
}

pub fn get_input(year: u16, day: u8, session: &str) -> Result<Arc<String>, UtilsError> {
    static REQWEST_CLIENT: LazyLock<Client> = LazyLock::new(|| default_reqwest_client());
    static MEM_CACHE: LazyLock<DashMap<String, Arc<String>>> = LazyLock::new(DashMap::new);

    let key = Rc::new(format!("{}_{}_{}", year, day, session));
    let mem_cache_map = &*MEM_CACHE;
    if let Some(value) = mem_cache_map.get(&*key) {
        return Ok(value.value().clone());
    }
    let value = cacache_sync::read("./cache", &*key)
        .ok()
        .map(String::from_utf8)
        .map(|r| {
            r.map_err(|e| {
                UtilsError::DiskCacheError(format!(
                    "failed to read value of key as utf-8 {:?}: {}",
                    key, e
                ))
            })
        })
        .unwrap_or_else(|| {
            let client = &*REQWEST_CLIENT;
            let cookie = format!("session={}", session);
            let url = format!("https://adventofcode.com/{}/day/{}/input", year, day);
            client
                .get(&url)
                .header("Cookie", cookie)
                .send()
                .map_err(|e| UtilsError::ReqwestError(format!("{:?}", e)))?
                .error_for_status()
                .map_err(|e| UtilsError::ResponseStatusError(format!("{:?}", e)))?
                .text_with_charset("utf-8")
                .map_err(|e| UtilsError::ResponseStringBodyError(format!("{:?}", e)))
                .inspect(|value| {
                    let _ = cacache_sync::write("./cache", &*key, value.as_bytes());
                })
        })?;

    let value = Arc::new(value);
    mem_cache_map.insert((*key).clone(), value.clone());

    Ok(value)
}

pub fn submit<A: Display + Debug>(
    year: u16,
    day: u8,
    part: u8,
    answer: A,
    session: &str,
) -> Result<(), UtilsError> {
    static REQWEST_CLIENT: OnceLock<Client> = OnceLock::new();
    let client = REQWEST_CLIENT.get_or_init(default_reqwest_client);
    let cookie = format!("session={}", session);
    let url = format!("https://adventofcode.com/{}/day/{}/answer", year, day);
    let mut params = HashMap::new();
    params.insert("level", part.to_string());
    params.insert("answer", answer.to_string());
    let body = client
        .post(url)
        .header("Cookie", cookie)
        .form(&params)
        .send()
        .map_err(|e| UtilsError::ReqwestError(format!("{:?}", e)))?
        .error_for_status()
        .map_err(|e| UtilsError::ResponseStatusError(format!("{:?}", e)))?
        .text_with_charset("utf-8")
        .map_err(|e| UtilsError::ResponseStringBodyError(format!("{:?}", e)))?;

    let dom = Html::parse_document(body.as_str());
    let main_selector = Selector::parse("main").unwrap();
    if let Some(main_body) = dom.select(&main_selector).next() {
        let main_body_text = main_body.text().collect::<String>();
        if main_body_text.contains("not the right answer") {
            return Err(UtilsError::IncorrectAnswer(format!(
                "answer {:?} for {} day {} part {}",
                answer, year, day, part
            )));
        }
        if main_body_text.contains("already complete it") {
            return Err(UtilsError::AlreadySubmitted(format!(
                "{} day {} part {}",
                year, day, part
            )));
        }
        if main_body_text.contains("gave an answer too recently") {
            static RE: OnceLock<Result<Regex, UtilsError>> = OnceLock::new();
            let regex = RE
                .get_or_init(|| {
                    Regex::new(r"You have (.+) left to wait.").map_err(|e| {
                        UtilsError::RegexError(format!(
                            "failed to init regex `{}`: {}",
                            r"You have (.+) left to wait.", e
                        ))
                    })
                })
                .as_ref()
                .map_err(Clone::clone)?;

            let throttle_time_str = regex
                .captures(main_body_text.as_str())
                .and_then(|c| c.get(1))
                .map(|c| c.as_str())
                .ok_or_else(|| {
                    UtilsError::SubmissionThrottled(
                        format!(
                            "{} day {} part {}: failed to parse throttling time",
                            year, day, part,
                        ),
                        None,
                    )
                })?;
            let duration = humantime::parse_duration(throttle_time_str).map_err(|e| {
                UtilsError::SubmissionThrottled(
                    format!(
                        "{} day {} part {}: failed to parse throttling time: {}",
                        year, day, part, e
                    ),
                    None,
                )
            })?;

            return Err(UtilsError::SubmissionThrottled(
                format!(
                    "{} day {} part {}: {}",
                    year,
                    day,
                    part,
                    humantime::format_duration(duration)
                ),
                Some(duration),
            ));
        }
    }
    Ok(())
}

pub fn check_valid_question(year: u16, day: Option<u8>) -> Result<RangeInclusive<u8>, UtilsError> {
    let now_eastern = Utc::now().with_timezone(&Eastern);
    if now_eastern.year() < year as i32 {
        return Err(UtilsError::InvalidAOCProblem(format!("year {}", year)));
    }

    if now_eastern.year() > year as i32 {
        if let Some(day) = day {
            match day {
                1..=25 => Ok(day..=day),
                _ => Err(UtilsError::InvalidAOCProblem(format!(
                    "year {} day {}",
                    year, day
                ))),
            }
        } else {
            Ok(1..=25)
        }
    } else {
        if now_eastern.month() < 12 {
            Err(UtilsError::InvalidAOCProblem(format!(
                "not December for current year {}",
                year
            )))
        } else {
            if let Some(day) = day {
                if day >= 1 && (day as u32) <= now_eastern.day() {
                    Ok(day..=day)
                } else {
                    Err(UtilsError::InvalidAOCProblem(format!(
                        "the time has not come year {} day {}",
                        year, day
                    )))
                }
            } else {
                Ok(1..=now_eastern.day() as u8)
            }
        }
    }
}

#[cfg(test)]
pub mod tests_utils {
    use crate::utils::UtilsError;
    use dotenv::dotenv;
    use std::env;
    use std::fmt::{Debug, Display};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, OnceLock};

    static SESSION: OnceLock<String> = OnceLock::new();
    static NOT_INIT: AtomicBool = AtomicBool::new(true);

    pub fn init() {
        if NOT_INIT.load(Ordering::Acquire) {
            dotenv().expect("`.env` file not found");
            SESSION.get_or_init(|| {
                env::var("SESSION_COOKIE").expect("Missing cookie \"SESSION_COOKIE\"")
            });
            NOT_INIT.store(false, Ordering::Release);
        }
    }

    pub fn get_input(year: u16, day: u8) -> Result<Arc<String>, UtilsError> {
        init();
        super::get_input(year, day, SESSION.get().unwrap())
    }

    #[allow(dead_code)]
    pub fn submit<A: Display + Debug>(
        year: u16,
        day: u8,
        part: u8,
        answer: A,
    ) -> Result<(), UtilsError> {
        init();
        super::submit(year, day, part, answer, SESSION.get().unwrap())?;
        Ok(())
    }
}
