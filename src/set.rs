use crate::error::Error;
use std::collections::HashSet;
use std::hash::Hash;

#[allow(dead_code)]
pub trait Set<T> {
    fn contains(&self, elem: &T) -> Result<bool, Error>;
    fn insert(&mut self, elem: T) -> Result<bool, Error>;
}

impl<T> Set<T> for HashSet<T>
where
    T: Eq + Hash,
{
    fn contains(&self, elem: &T) -> Result<bool, Error> {
        Ok(self.contains(elem))
    }

    fn insert(&mut self, elem: T) -> Result<bool, Error> {
        Ok(self.insert(elem))
    }
}
