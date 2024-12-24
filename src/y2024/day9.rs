use crate::error::Error;
use crate::part_solver;
use crate::utils::ures;
use std::cmp::{min, Ordering, Reverse};
use std::collections::BinaryHeap;

part_solver!();

pub fn part1(input: &str) -> Result<ures, Error> {
    let data: Vec<u8> = parse_input(input)?;
    let (_, sum) = CompactDataIter::new_borrow(data.as_slice()).fold(
        (0, 0),
        |(mut pos, mut sum): (ures, ures), (val, count)| {
            let count = count as ures;
            let val = val as ures;
            if val != 0 {
                sum += (2 * pos + count - 1) * count / 2 * val;
            };
            pos += count;
            (pos, sum)
        },
    );

    Ok(sum)
}

pub fn part2(input: &str) -> Result<ures, Error> {
    let data: Vec<u8> = parse_input(input)?;
    Ok(process_disk_part_2(data.into_iter())?
        .into_iter()
        .enumerate()
        .skip(1) // skipping zero value
        .map(|(val, (pos, count))| {
            let count = count as ures;
            let val = val as ures;
            let pos = pos as ures;
            (2 * pos + count - 1) * count / 2 * val
        })
        .sum())
}

fn parse_input(input: &str) -> Result<Vec<u8>, Error> {
    let input = input.trim();
    input
        .as_bytes()
        .iter()
        .map(|b| {
            if b.is_ascii_digit() {
                Ok(*b - b'0')
            } else {
                Err(Error::ParseError(
                    format!("invalid input, expect digit: {:?}", *b as char).into(),
                ))
            }
        })
        .try_fold(Vec::with_capacity(input.len()), |mut v, b| {
            b.map(|b| {
                v.push(b);
                v
            })
        })
}

fn process_disk_part_2<T: ExactSizeIterator<Item = u8>>(
    disk: T,
) -> Result<Vec<(usize, u8)>, Error> {
    let len = disk.len();
    let (_, mut loc_map, mut free_space_size_map) = disk.enumerate().fold(
        (0, Vec::with_capacity(len), vec![None; 9]),
        |(mut pos, mut loc_map, mut free_space_size_map), (idx, size)| {
            if (idx % 2) == 0 {
                loc_map.push((pos, size));
            } else if size > 0 {
                let free_space_size_idx = size as usize - 1;
                free_space_size_map[free_space_size_idx]
                    .get_or_insert_with(BinaryHeap::new)
                    .push(Reverse(pos));
            };
            pos += size as usize;
            (pos, loc_map, free_space_size_map)
        },
    );

    loc_map.iter_mut().rev().try_for_each(|(pos, size)| {
        if let Some(new_pos) = disk_realloc_part_2(*pos, *size, &mut free_space_size_map)? {
            *pos = new_pos
        }
        Ok::<_, Error>(())
    })?;

    Ok(loc_map)
}

fn disk_realloc_part_2(
    pos: usize,
    size: u8,
    free_space_size_map: &mut [Option<BinaryHeap<Reverse<usize>>>],
) -> Result<Option<usize>, Error> {
    if size == 0 {
        Ok(None)
    } else {
        let min_free_pos_and_idx = free_space_size_map[size as usize - 1..]
            .iter()
            .enumerate()
            .filter_map(|(new_free_size, heap)| {
                heap.as_ref()
                    .and_then(|heap| heap.peek())
                    .map(|&Reverse(new_pos)| (new_pos, new_free_size))
            })
            .min()
            .filter(|(new_pos, _)| *new_pos < pos);

        if let Some((new_pos, new_free_size)) = min_free_pos_and_idx {
            free_space_size_map[size as usize + new_free_size - 1]
                .as_mut()
                .ok_or_else(|| {
                    Error::InvalidState(
                        "free space size map at found free size should not be none".into(),
                    )
                })?
                .pop()
                .ok_or_else(|| {
                    Error::InvalidState(
                        "free space size map at found free size should not be empty".into(),
                    )
                })?;
            if new_free_size > 0 {
                free_space_size_map[new_free_size - 1]
                    .get_or_insert_default()
                    .push(Reverse(new_pos + size as usize));
            }
            Ok(Some(new_pos))
        } else {
            Ok(None)
        }
    }
}

struct CompactDataIter<'a> {
    disk: &'a [u8],
    current_idx: usize,
    rev_idx: usize,
    current_val: usize,
    current_val_count: u8,
    rev_left_over_count: u8,
}

impl<'a> CompactDataIter<'a> {
    fn new_borrow(disk: &'a [u8]) -> Self {
        let current_idx = 0;
        let rev_idx = disk.len() & (usize::MAX - 1);
        let current_val = 0;
        let current_val_count = disk.first().copied().unwrap_or(0);
        let rev_left_over_count = disk.get(rev_idx).copied().unwrap_or(0);
        CompactDataIter {
            disk,
            current_idx,
            rev_idx,
            current_val,
            current_val_count,
            rev_left_over_count,
        }
    }
}

impl Iterator for CompactDataIter<'_> {
    type Item = (usize, u8);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_idx > self.rev_idx {
            return None;
        }
        while self.current_val_count == 0 {
            self.current_idx += 1;
            match self.current_idx.cmp(&self.rev_idx) {
                Ordering::Less => {
                    if (self.current_idx % 2) == 1 {
                        self.current_val = self.rev_idx / 2;
                    } else {
                        self.current_val = self.current_idx / 2;
                    }
                    self.current_val_count = self.disk[self.current_idx];
                }
                Ordering::Equal => {
                    self.current_val = self.rev_idx / 2;
                    self.current_val_count = self.rev_left_over_count;
                    self.rev_left_over_count = 0;
                }
                Ordering::Greater => {
                    return None;
                }
            }
        }
        if (self.current_idx % 2) == 1 {
            while self.rev_left_over_count == 0 {
                self.rev_idx -= 2;
                if self.current_idx > self.rev_idx {
                    return None;
                }
                self.current_val = self.rev_idx / 2;
                self.rev_left_over_count = self.disk[self.rev_idx];
            }
            let min_count = min(self.rev_left_over_count, self.current_val_count);
            self.rev_left_over_count -= min_count;
            self.current_val_count -= min_count;
            Some((self.current_val, min_count))
        } else {
            let count = self.current_val_count;
            self.current_val_count = 0;
            Some((self.current_val, count))
        }
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
        let input = get_input(2024, 9)?;
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
        let input = get_input(2024, 9)?;
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
