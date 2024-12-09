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
    use crate::utils::tests_utils::get_input;
    use chrono::Utc;

    #[ignore]
    #[test]
    pub fn part1() -> Result<(), Error> {
        let input = get_input(2024, 11)?;
        let start = Utc::now();
        println!("Result: {}", super::part1(input.as_str())?);
        let duration = Utc::now() - start;
        println!("Runtime: {}", duration);
        Ok(())
    }

    #[ignore]
    #[test]
    pub fn part2() -> Result<(), Error> {
        let start = Utc::now();
        let input = get_input(2024, 11)?;
        println!("Result: {}", super::part2(input.as_str())?);
        let duration = Utc::now() - start;
        println!("Runtime: {}", duration);
        Ok(())
    }
}
