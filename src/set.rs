use crate::error::Error;
use crate::space::Space;
use derive_more::{Deref, DerefMut};
use std::collections::HashSet;
use std::hash::Hash;
use std::marker::PhantomData;

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

#[derive(Clone, Debug, Deref, DerefMut)]
pub struct BoolSpace<I, B: Space<bool, I, 2>> {
    inner: B,
    #[deref(ignore)]
    #[deref_mut(ignore)]
    phantom_data_i: PhantomData<I>,
}

impl<I, B: Space<bool, I, 2>> From<B> for BoolSpace<I, B> {
    fn from(value: B) -> Self {
        Self {
            inner: value,
            phantom_data_i: PhantomData,
        }
    }
}

impl<I, B: Space<bool, I, 2>> Set<[I; 2]> for BoolSpace<I, B> {
    fn contains(&self, elem: &[I; 2]) -> Result<bool, Error> {
        self.get(elem)
            .ok_or_else(|| Error::InvalidState("out of bound".into()))
            .cloned()
    }
    fn insert(&mut self, elem: [I; 2]) -> Result<bool, Error> {
        let val = self
            .get_mut(&elem)
            .ok_or_else(|| Error::InvalidState("out of bound".into()))?;
        let previous_inserted = *val;
        *val = true;
        Ok(!previous_inserted)
    }
}
