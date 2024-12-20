use crate::error::Error;
use std::borrow::Borrow;
use std::iter::Peekable;

pub trait TrieNode<T>: Default {
    fn get<B: Borrow<T>>(&self, t: B) -> Result<Option<&Self>, Error>;
    fn get_mut_or_add_default<B: Borrow<T>>(&mut self, t: B) -> Result<&mut Self, Error>;
    fn depth(&self) -> usize;
    fn is_valid(&self) -> bool;
    fn set_valid(&mut self, b: bool);

    fn add<I: IntoIterator<Item = T>>(&mut self, iter: I) -> Result<(), Error> {
        let iter = iter.into_iter();
        let mut current = self;
        for val in iter {
            current = current.get_mut_or_add_default(val)?;
        }
        current.set_valid(true);
        Ok(())
    }

    fn find_prefix<Item: Borrow<T>, I: Iterator<Item = Item>>(
        &self,
        iter: &mut Peekable<I>,
    ) -> Result<Option<&Self>, Error> {
        let mut current = self;
        while let Some(val) = iter.peek() {
            if let Some(next_node) = current.get(val.borrow())? {
                iter.next();
                if next_node.is_valid() {
                    return Ok(Some(next_node));
                } else {
                    current = next_node;
                }
            } else {
                return Ok(None);
            }
        }
        Ok(None)
    }
}

#[derive(Debug)]
pub struct ArrayTrie<const N: usize> {
    children: [Option<Box<ArrayTrie<N>>>; N],
    len: usize,
    valid: bool,
}

impl<const N: usize> Default for ArrayTrie<N> {
    fn default() -> Self {
        let mut v = Vec::new();
        v.reserve_exact(N);
        v.resize_with(N, Option::default);
        Self {
            children: v.try_into().unwrap(),
            len: 0,
            valid: false,
        }
    }
}

impl<const N: usize> TrieNode<usize> for ArrayTrie<N> {
    fn get<B: Borrow<usize>>(&self, t: B) -> Result<Option<&Self>, Error> {
        let idx = *t.borrow();
        let res = self
            .children
            .get(idx)
            .ok_or_else(|| {
                Error::InvalidState(format!("out of bound, index {} bound {}", idx, N).into())
            })?
            .as_ref()
            .map(|b| b.as_ref());
        Ok(res)
    }

    fn get_mut_or_add_default<B: Borrow<usize>>(&mut self, t: B) -> Result<&mut Self, Error> {
        let idx = *t.borrow();
        let res = self
            .children
            .get_mut(idx)
            .ok_or_else(|| {
                Error::InvalidState(format!("out of bound, index {} bound {}", idx, N).into())
            })?
            .get_or_insert_with(|| {
                Self {
                    len: self.len + 1,
                    ..Default::default()
                }
                .into()
            })
            .as_mut();
        Ok(res)
    }

    fn depth(&self) -> usize {
        self.len
    }

    fn is_valid(&self) -> bool {
        self.valid
    }

    fn set_valid(&mut self, b: bool) {
        self.valid = b;
    }
}
