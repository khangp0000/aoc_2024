use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap};
use std::ops::Deref;
use crate::error::Error;
use crate::part_solver;
use crate::utils::ures;

part_solver!();

pub fn part1(input: &str) -> Result<ures, Error> {
    let data: Vec<u8> = parse_input(input)?;
    Ok(CompactDataIter::new_borrow(data.as_slice())
        .enumerate()
        .map(|(id, val)| (id as ures) * (val as ures))
        .sum())
}

pub fn part2(input: &str) -> Result<ures, Error> {

    let data: Vec<u8> = parse_input(input)?;
    Ok(process_disk_part_2(data.into_iter())
        .into_iter()
        .enumerate()
        .skip(1) // skipping zero value
        .map(|(val, (pos, count))| {
            let count = count as usize;
            (2*pos + count - 1)*count / 2 * val
        }).map(|v| v as ures)
        .sum())
}

fn parse_input(input: &str) -> Result<Vec<u8>, Error> {
    let input = input.trim();
        input .as_bytes()
        .iter()
        .map(|b| {
            if b.is_ascii_digit() {
                Ok(*b - b'0')
            } else {
                Err(Error::ParseError(format!("invalid input, expect digit: {:?}", *b as char)))
            }
        })
        .try_fold(Vec::with_capacity(input.len()), |mut v, b| {
           b.map(|b| {
               v.push(b);
               v
           })
        })
}
fn process_disk_part_2<T: ExactSizeIterator<Item = u8>>(disk: T) -> Vec<(usize, u8)> {
    let (_, mut loc_map, mut free_space_size_map) = disk.enumerate()
        .fold((0, Vec::new(), BTreeMap::new()), |(mut pos,mut loc_map, mut free_space_size_map), (idx, size)| {
            if (idx % 2) == 0 {
                loc_map.push((pos, size));
            } else {
                free_space_size_map.entry(size)
                    .or_insert(BTreeSet::new())
                    .insert(pos);
            };
            pos += size as usize;
            (pos, loc_map, free_space_size_map)
    });

    loc_map.iter_mut()
        .rev()
        .for_each(|(pos, size)| {
            if let Some(new_pos) = disk_realloc(*pos, *size, &mut free_space_size_map) {
                *pos = new_pos
            }
        });

    loc_map
}

fn disk_realloc(pos: usize, size: u8, free_space_size_map: &mut BTreeMap<u8, BTreeSet<usize>>) -> Option<usize> {
    let new_pos = free_space_size_map.range_mut(size..)
        .filter_map(|(free_size, tree)|
            tree.first().cloned().filter(|loc| *loc < pos).map(|loc| (*free_size, tree, loc))
        ).fold(None, |min_loc_tree, (free_size, tree, loc)| {
        match min_loc_tree {
            None => {Some((free_size, tree, loc))}
            Some((current_min_free_size, current_min_tree, current_min_loc)) => {
                if loc < current_min_loc {
                    Some((free_size, tree, loc))
                } else {
                    Some((current_min_free_size, current_min_tree, current_min_loc))
                }
            }
        }
    });

    if let Some((free_size, tree, new_pos)) = new_pos {
        let _ = tree.pop_first();
        let new_size = free_size - size;
        if new_size > 0 {
            let new_free_pos = new_pos + size as usize;
            free_space_size_map.entry(new_size).or_default().insert(new_free_pos);
        }
        Some(new_pos)
    } else {
        None
    }
}

fn process_disk_part_2_2<T: ExactSizeIterator<Item = u8>>(disk: T) -> Vec<(usize, u8)> {
    let len = disk.len();
    let (_, mut loc_map, mut free_space_size_map) = disk.enumerate()
        .fold((0, Vec::with_capacity(len), vec![None; 9]), |(mut pos,mut loc_map, mut free_space_size_map), (idx, size)| {
            if (idx % 2) == 0 {
                loc_map.push((pos, size));
            } else {
                if size > 0 {
                    let free_space_size_idx = size as usize - 1;
                    free_space_size_map[free_space_size_idx].get_or_insert_with(BinaryHeap::new)
                        .push(Reverse(pos));
                }
                // free_space_size_map.entry(size)
                //     .or_insert(BTreeSet::new())
                //     .insert(pos);
            };
            pos += size as usize;
            (pos, loc_map, free_space_size_map)
        });

    loc_map.iter_mut()
        .rev()
        .for_each(|(pos, size)| {
            if let Some(new_pos) = disk_realloc_2(*pos, *size, &mut free_space_size_map) {
                *pos = new_pos
            }
        });

    loc_map
}

fn disk_realloc_2(pos: usize, size: u8, free_space_size_map: &mut Vec<Option<BinaryHeap<Reverse<usize>>>>) -> Option<usize> {
    if size == 0 {
        None
    } else {
        let min_free_pos_and_idx = free_space_size_map[size as usize - 1..]
            .iter()
            .enumerate()
            .filter_map(|(new_free_size, heap)| {
                heap.as_ref().and_then(|heap| heap.peek())
                    .map(|&Reverse(new_pos)| (new_pos, new_free_size))
            })
            .filter(|(new_pos, _)| *new_pos < pos)
            .min();

        if let Some((new_pos, new_free_size)) = min_free_pos_and_idx {
            free_space_size_map[size as usize + new_free_size - 1].as_mut().unwrap().pop().unwrap();
            if new_free_size > 0 {
                free_space_size_map[new_free_size - 1].get_or_insert_default().push(Reverse(new_pos + size as usize));
            }
            Some(new_pos)
        } else {
            None
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
        let current_val_count = disk.get(0).cloned().unwrap_or(0);
        let rev_left_over_count = disk.get(rev_idx).cloned().unwrap_or(0);
        CompactDataIter {
            disk: disk,
            current_idx,
            rev_idx,
            current_val,
            current_val_count,
            rev_left_over_count,
        }
    }
}

impl<'a> Iterator for CompactDataIter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_idx > self.rev_idx {
            return None
        }
        while self.current_val_count == 0 {
            self.current_idx += 1;
            if self.current_idx > self.rev_idx {
                return None
            } else if self.current_idx == self.rev_idx {
                self.current_val = self.rev_idx / 2;
                self.current_val_count = self.rev_left_over_count;
                self.rev_left_over_count = 0;
            } else {
                if (self.current_idx % 2) == 1 {
                    self.current_val = self.rev_idx / 2;
                } else {
                    self.current_val = self.current_idx / 2;
                }
                self.current_val_count = self.disk[self.current_idx];
            }
        }
        if (self.current_idx % 2) == 1 {
            while self.rev_left_over_count == 0 {
                self.rev_idx -= 2;
                if self.current_idx > self.rev_idx {
                    return None
                }
                self.current_val = self.rev_idx / 2;
                self.rev_left_over_count = self.disk[self.rev_idx];
            }
            self.rev_left_over_count -= 1;
        }

        self.current_val_count -= 1;
        Some(self.current_val)
    }
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::utils::tests_utils::get_input;
    use chrono::Utc;
    use crate::y2024::day9::CompactDataIter;

    #[test]
    pub fn part_none() -> Result<(), Error> {
        let v = vec![2,3,3,3,1,3,3,1,2,1,4,1,4,1,3,1,4,0,2];
        CompactDataIter::new_borrow(&v)
            .for_each(|v| println!("{}", v));
        Ok(())
    }

    #[ignore]
    #[test]
    pub fn part1() -> Result<(), Error> {
        let input = get_input(2024, 9)?;
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
        let input = get_input(2024, 9)?;
        println!("Result: {}", super::part2(input.as_str())?);
        let duration = Utc::now() - start;
        println!("Runtime: {}", duration);
        Ok(())
    }
}
