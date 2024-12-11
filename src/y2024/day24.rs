use crate::error::Error;
use crate::part_solver;
use crate::utils::ures;

part_solver!();

pub fn part1(_input: &str) -> Result<ures, Error> {
    todo!()
}

pub fn part2(_input: &str) -> Result<ures, Error> {
    todo!()
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::utils::tests_utils::{get_input, human_text_duration};
    use chrono::Utc;

    #[ignore]
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

    #[ignore]
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
