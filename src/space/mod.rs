use std::ptr::from_mut;
pub mod space2d;

pub trait Space<T, C, const N: usize> {
    fn get(&self, idx: &[C; N]) -> Option<&T>;
    fn set(&mut self, idx: &[C; N], val: T) -> Option<T>;
    fn get_mut(&mut self, idx: &[C; N]) -> Option<&mut T>;
    fn swap(&mut self, idx1: &[C; N], idx2: &[C; N]) -> bool {
        if let Some(ptr1) = self.get_mut(idx1).map(from_mut) {
            if let Some(ptr2) = self.get_mut(idx2).map(from_mut) {
                unsafe {
                    if ptr1 != ptr2 {
                        std::ptr::swap(ptr1, ptr2);
                    }
                }
                return true;
            }
        }

        false
    }
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
            .try_fold([0; N], |mut v, (idx, val)| {
                v[idx] = val.checked_add_signed(diff[idx])?;
                Some(v)
            })
    }

    #[inline]
    fn shift_dimension(&self, idx: usize, diff: isize) -> Option<[usize; N]> {
        let mut res = *self;
        res[idx] = res[idx].checked_add_signed(diff)?;
        Some(res)
    }
}
