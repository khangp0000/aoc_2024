use bit_set::BitSet;
use bit_vec::BitBlock;
use std::borrow::Cow;
use std::mem::replace;

pub trait Space<T, C, const N: usize> {
    fn get(&self, idx: &[C; N]) -> Option<&T>;
    fn set(&mut self, idx: &[C; N], val: T) -> Option<T>;
    fn get_mut(&mut self, idx: &[C; N]) -> Option<&mut T>;
}

pub trait IterMutSpace<T, C, const N: usize> {
    #[allow(dead_code)]
    fn iter_mut(&mut self) -> impl Iterator<Item = ([usize; 2], &mut T)>
    where
        T: 'static;
}

pub trait IterSpace<T, C, const N: usize> {
    fn iter(&self) -> impl Iterator<Item = ([usize; 2], &T)>
    where
        T: 'static;
}

#[derive(Clone)]
pub struct Board2d<T> {
    inner: Vec<Vec<T>>,
}

#[derive(Clone)]
pub struct RefBoard2d<'a, T>
where
    [T]: ToOwned<Owned = Vec<T>>,
{
    inner: Vec<Cow<'a, [T]>>,
}

impl<T> Space<T, usize, 2> for Board2d<T> {
    fn get(&self, idx: &[usize; 2]) -> Option<&T> {
        let [x, y] = idx;
        self.inner.get(*y).and_then(|v| v.get(*x))
    }

    fn set(&mut self, idx: &[usize; 2], val: T) -> Option<T> {
        let [x, y] = idx;
        self.inner
            .get_mut(*y)
            .and_then(|v| v.get_mut(*x))
            .map(|v| replace(v, val))
    }

    fn get_mut(&mut self, idx: &[usize; 2]) -> Option<&mut T> {
        let [x, y] = idx;
        self.inner.get_mut(*y).and_then(|v| v.get_mut(*x))
    }
}

impl<T> Space<T, usize, 2> for RefBoard2d<'_, T>
where
    [T]: ToOwned<Owned = Vec<T>>,
{
    fn get(&self, idx: &[usize; 2]) -> Option<&T> {
        let [x, y] = idx;
        self.inner.get(*y).and_then(|v| v.get(*x))
    }

    fn set(&mut self, idx: &[usize; 2], val: T) -> Option<T> {
        let [x, y] = idx;
        self.inner
            .get_mut(*y)
            .and_then(|v| v.to_mut().get_mut(*x))
            .map(|v| replace(v, val))
    }

    fn get_mut(&mut self, idx: &[usize; 2]) -> Option<&mut T> {
        let [x, y] = idx;
        self.inner.get_mut(*y).and_then(|v| v.to_mut().get_mut(*x))
    }
}

impl<T> IterSpace<T, usize, 2> for Board2d<T> {
    fn iter(&self) -> impl Iterator<Item = ([usize; 2], &T)> {
        self.inner
            .iter()
            .enumerate()
            .flat_map(move |(y, v)| v.iter().enumerate().map(move |(x, val)| ([x, y], val)))
    }
}

impl<T> IterSpace<T, usize, 2> for RefBoard2d<'_, T>
where
    [T]: ToOwned<Owned = Vec<T>>,
{
    fn iter(&self) -> impl Iterator<Item = ([usize; 2], &T)> {
        self.inner
            .iter()
            .enumerate()
            .flat_map(move |(y, v)| v.iter().enumerate().map(move |(x, val)| ([x, y], val)))
    }
}

impl<T> IterMutSpace<T, usize, 2> for Board2d<T> {
    fn iter_mut(&mut self) -> impl Iterator<Item = ([usize; 2], &mut T)> {
        self.inner
            .iter_mut()
            .enumerate()
            .flat_map(move |(y, v)| v.iter_mut().enumerate().map(move |(x, val)| ([x, y], val)))
    }
}

impl<T> IterMutSpace<T, usize, 2> for RefBoard2d<'_, T>
where
    [T]: ToOwned<Owned = Vec<T>>,
{
    fn iter_mut(&mut self) -> impl Iterator<Item = ([usize; 2], &mut T)> {
        self.inner.iter_mut().enumerate().flat_map(move |(y, v)| {
            v.to_mut()
                .iter_mut()
                .enumerate()
                .map(move |(x, val)| ([x, y], val))
        })
    }
}

impl<T> From<Vec<Vec<T>>> for Board2d<T> {
    fn from(value: Vec<Vec<T>>) -> Self {
        Board2d { inner: value }
    }
}

impl<'a, T> From<Vec<Cow<'a, [T]>>> for RefBoard2d<'a, T>
where
    [T]: ToOwned<Owned = Vec<T>>,
{
    fn from(value: Vec<Cow<'a, [T]>>) -> Self {
        RefBoard2d { inner: value }
    }
}

impl<T> AsRef<Vec<Vec<T>>> for Board2d<T> {
    fn as_ref(&self) -> &Vec<Vec<T>> {
        &self.inner
    }
}

#[allow(dead_code)]
impl<T> RefBoard2d<'_, T>
where
    [T]: ToOwned<Owned = Vec<T>>,
{
    pub fn height(&self) -> usize {
        self.inner.len()
    }

    pub fn width(&self, row: usize) -> Option<usize> {
        self.inner.get(row).map(|s| s.len())
    }

    pub fn map_ref<U, F: Fn([usize; 2], &T) -> U>(&self, f: F) -> Board2d<U> {
        self.inner
            .iter()
            .enumerate()
            .map(|(y, row)| {
                row.iter()
                    .enumerate()
                    .map(|(x, val)| f([x, y], val))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
            .into()
    }
}

#[allow(dead_code)]
impl<T> Board2d<T> {
    pub fn height(&self) -> usize {
        self.inner.len()
    }

    pub fn width(&self, row: usize) -> Option<usize> {
        self.inner.get(row).map(Vec::len)
    }
}

#[derive(Clone)]
pub struct BitBoard2d<B: BitBlock = usize> {
    inner: Vec<BitSet<B>>,
}

impl<B: BitBlock> BitBoard2d<B> {
    pub fn with_height(height: usize) -> Self {
        BitBoard2d {
            inner: vec![BitSet::<B>::default(); height],
        }
    }
}

impl<B: BitBlock> Space<bool, usize, 2> for BitBoard2d<B> {
    fn get(&self, idx: &[usize; 2]) -> Option<&bool> {
        let [x, y] = idx;
        self.inner
            .get(*y)
            .map(|v| if v.contains(*x) { &true } else { &false })
    }

    fn set(&mut self, idx: &[usize; 2], val: bool) -> Option<bool> {
        let [x, y] = idx;
        self.inner.get_mut(*y).map(|v| match val {
            true => !v.insert(*x),
            false => v.remove(*x),
        })
    }

    fn get_mut(&mut self, _idx: &[usize; 2]) -> Option<&mut bool> {
        unimplemented!()
    }
}

#[allow(dead_code)]
impl<B: BitBlock> BitBoard2d<B> {
    pub fn set_force(&mut self, idx: &[usize; 2], val: bool) -> Option<bool> {
        let [x, y] = idx;
        if *y >= self.inner.len() {
            self.inner.resize_with(y.checked_add(1)?, BitSet::default);
        }
        self.inner.get_mut(*y).map(|v| match val {
            true => !v.insert(*x),
            false => v.remove(*x),
        })
    }
}

#[allow(dead_code)]
pub trait Pos<const N: usize> {
    fn shift(&self, idx: &[isize; N]) -> Option<[usize; N]>;
    fn shift_dimension(&self, idx: usize, diff: isize) -> Option<[usize; N]>;
}

impl<const N: usize> Pos<N> for [usize; N] {
    #[inline]
    fn shift(&self, diff: &[isize; N]) -> Option<[usize; N]> {
        self.iter()
            .enumerate()
            .try_fold(Vec::with_capacity(N), |mut v, (idx, val)| {
                v.push(val.checked_add_signed(diff[idx])?);
                Some(v)
            })
            .map(|v| v.try_into().unwrap())
    }

    #[inline]
    fn shift_dimension(&self, idx: usize, diff: isize) -> Option<[usize; N]> {
        let mut res = *self;
        res[idx] = res[idx].checked_add_signed(diff)?;
        Some(res)
    }
}
