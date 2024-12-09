pub trait Space<T, C, const N: usize> {
    fn get(&self, idx: &[C; N]) -> Option<&T>;
    fn get_mut(&mut self, idx: &[C; N]) -> Option<&mut T>;
    fn iter(&self) -> impl Iterator<Item = ([usize; 2], &T)>
    where
        T: 'static;
    fn iter_mut(&mut self) -> impl Iterator<Item = ([usize; 2], &mut T)>
    where
        T: 'static;
}

#[derive(Clone)]
pub struct Board2d<T> {
    inner: Vec<Vec<T>>,
}

impl<T> Space<T, usize, 2> for Board2d<T> {
    fn get(&self, idx: &[usize; 2]) -> Option<&T> {
        let [x, y] = idx;
        self.inner.get(*y).and_then(|v| v.get(*x))
    }

    fn get_mut(&mut self, idx: &[usize; 2]) -> Option<&mut T> {
        let [x, y] = idx;
        self.inner.get_mut(*y).and_then(|v| v.get_mut(*x))
    }

    fn iter(&self) -> impl Iterator<Item = ([usize; 2], &T)> {
        self.inner
            .iter()
            .enumerate()
            .flat_map(move |(y, v)| v.iter().enumerate().map(move |(x, val)| ([x, y], val)))
    }

    #[allow(dead_code)]
    fn iter_mut(&mut self) -> impl Iterator<Item = ([usize; 2], &mut T)> {
        self.inner
            .iter_mut()
            .enumerate()
            .flat_map(move |(y, v)| v.iter_mut().enumerate().map(move |(x, val)| ([x, y], val)))
    }
}

impl<T> From<Vec<Vec<T>>> for Board2d<T> {
    fn from(value: Vec<Vec<T>>) -> Self {
        Board2d { inner: value }
    }
}

impl<T> AsRef<Vec<Vec<T>>> for Board2d<T> {
    fn as_ref(&self) -> &Vec<Vec<T>> {
        &self.inner
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
