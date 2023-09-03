use crate::rmodule::ring::Ring;
use gcd::Gcd;
use itertools::Itertools;
use std::{
    cmp::min,
    collections::BTreeSet,
    fmt,
    ops::{Add, BitAnd, BitOr, Mul, Neg, Rem},
};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Matrix<T> {
    cols: u8,
    rows: u8,
    buffer: Vec<T>,
}

impl<T: fmt::Debug> fmt::Debug for Matrix<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Mtx({:?}x{:?}){:?}", self.cols, self.rows, self.buffer)
    }
}

impl<T> Matrix<T> {
    pub fn num_of_rows(&self) -> u8 {
        self.rows
    }
    pub fn num_of_cols(&self) -> u8 {
        self.cols
    }

    pub fn from_buffer<I>(buffer: I, cols: u8, rows: u8) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        Self {
            cols,
            rows,
            buffer: Vec::from_iter(buffer),
        }
    }

    pub fn from_rows<I>(rows: I, nof_rows: u8) -> Self
    where
        I: IntoIterator<Item = Vec<T>>,
    {
        let buffer = rows.into_iter().concat();
        Self {
            cols: match nof_rows {
                0 => 0,
                x => u8::try_from(buffer.len()).expect("we're gonna need a bigger int") / x,
            },
            rows: nof_rows,
            buffer,
        }
    }

    pub fn transpose_inplace(&mut self) {
        for row in 0..self.rows {
            for col in row + 1..self.cols {
                self.buffer.swap(
                    (row * self.cols + col).into(),
                    (col * self.rows + row).into(),
                );
            }
        }
    }

    pub fn get(&self, col: u8, row: u8) -> Option<&T> {
        self.buffer.get(usize::from(col + self.cols * row))
    }

    pub fn set(&mut self, col: u8, row: u8, value: T) {
        self.buffer[usize::from(col + self.cols + row)] = value;
    }

    /* # iters */

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.buffer.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.buffer.iter_mut()
    }

    pub fn into_iter(self) -> impl Iterator<Item = T> {
        self.buffer.into_iter()
    }

    fn row(&self, row: u8) -> impl Iterator<Item = &T> {
        self.buffer
            .iter()
            .skip(usize::from(row * self.cols))
            .take(usize::from(self.cols))
    }

    pub fn row_mut(&mut self, row: u8) -> impl Iterator<Item = &mut T> {
        self.buffer
            .iter_mut()
            .skip(usize::from(row * self.cols))
            .take(usize::from(self.cols))
    }

    fn col(&self, col: u8) -> impl Iterator<Item = &T> {
        self.buffer
            .iter()
            .skip(usize::from(col))
            .step_by(usize::from(self.cols))
            .take(usize::from(self.rows))
    }

    pub fn col_mut(&mut self, col: u8) -> impl Iterator<Item = &mut T> {
        self.buffer
            .iter_mut()
            .skip(usize::from(col))
            .step_by(usize::from(self.cols))
            .take(usize::from(self.rows))
    }
}

impl<T> Matrix<T>
where
    T: Copy,
{
    pub fn from_cols<I>(cols: I, nof_cols: u8) -> Self
    where
        I: IntoIterator<Item = Vec<T>>,
    {
        Self::from_rows(cols, nof_cols).transpose()
    }

    pub fn transpose(&self) -> Self {
        Self {
            cols: self.rows,
            rows: self.cols,
            buffer: (0..self.cols)
                .flat_map(|col| {
                    (0..self.rows).map(move |row| self.buffer[usize::from(row * self.cols + col)])
                })
                .collect(),
        }
    }

    pub fn rows(&self) -> impl Iterator<Item = Vec<T>> + '_ {
        (0..self.rows).map(|row| self.row(row).copied().collect())
    }

    pub fn cols(&self) -> impl Iterator<Item = Vec<T>> + '_ {
        (0..self.cols).map(|col| self.col(col).copied().collect())
    }

    pub fn buffer(&self) -> Vec<T> {
        self.buffer.clone()
    }
}

impl<T> Matrix<T>
where
    T: Copy + Add<T, Output = T> + Mul<T, Output = T>,
{
    // this assumes that self.rows == other.cols
    pub fn compose_unchecked(&self, other: &Self) -> Self {
        Self {
            cols: self.cols,
            rows: other.rows,
            buffer: (0..other.rows)
                .flat_map(|row| {
                    (0..self.cols).map(move |col| {
                        other
                            .row(row)
                            .zip(self.col(col))
                            .map(|(r, c)| *r * *c)
                            .reduce(|acc, nxt| acc + nxt)
                            .expect("matrices are not empty")
                    })
                })
                .collect(),
        }
    }
}

impl<T> Matrix<T>
where
    T: Copy + BitOr<T, Output = T> + BitAnd<T, Output = T>,
{
    // this assumes that self.rows == other.cols
    pub fn compose_unchecked_bool(&self, other: &Self) -> Self {
        Self {
            cols: self.cols,
            rows: other.rows,
            buffer: (0..other.rows)
                .flat_map(|row| {
                    (0..self.cols).map(move |col| {
                        other
                            .row(row)
                            .zip(self.col(col))
                            .map(|(r, c)| *r & *c)
                            .reduce(|acc, nxt| acc | nxt)
                            .expect("matrices are not empty")
                    })
                })
                .collect(),
        }
    }
}

impl<T> Add for &Matrix<T>
where
    T: Copy + Add<Output = T>,
{
    type Output = Matrix<T>;

    /**
    this assumes that the shapes of the matrices are the same
    we could panic otherwise, but that would reqiure checking
    and therefore slow us down
    */
    fn add(self, other: Self) -> Self::Output {
        Matrix {
            cols: self.cols,
            rows: other.rows,
            buffer: self
                .buffer
                .iter()
                .zip(other.buffer.iter())
                .map(|(x, y)| *x + *y)
                .collect(),
        }
    }
}

impl<T> Neg for &Matrix<T>
where
    T: Copy + Neg<Output = T>,
{
    type Output = Matrix<T>;

    fn neg(self) -> Self::Output {
        Matrix {
            cols: self.cols,
            rows: self.rows,
            buffer: self.buffer.iter().map(|x| -*x).collect(),
        }
    }
}

impl<R: Ring + Rem<Output = R> + Ord + Gcd> Matrix<R>
where
    R: fmt::Debug,
{
    fn identity(cols: u8, rows: u8) -> Self {
        Self::from_buffer(
            (0..rows).flat_map(|r| {
                (0..cols).map(move |c| match r == c {
                    true => R::one(),
                    false => R::zero(),
                })
            }),
            cols,
            rows,
        )
    }

    fn mul_row_by(&mut self, row: u8, r: R) {
        for v in self.row_mut(row) {
            *v = *v * r;
        }
    }

    fn mul_col_by(&mut self, col: u8, r: R) {
        for v in self.col_mut(col) {
            *v = *v * r;
        }
    }

    fn add_muled_row_to_row(&mut self, muled_row: u8, to_row: u8, r: R) {
        let mrow: Vec<_> = self.row(muled_row).copied().collect();
        for (t, m) in self.row_mut(to_row).zip(mrow) {
            *t = *t + m * r
        }
    }

    fn add_muled_col_to_col(&mut self, muled_col: u8, to_col: u8, r: R) {
        let mcol: Vec<_> = self.col(muled_col).copied().collect();
        for (t, m) in self.col_mut(to_col).zip(mcol) {
            *t = *t + m * r
        }
    }

    fn smallest_nonzero_entry(
        &self,
        done_cols: &BTreeSet<u8>,
        done_rows: &BTreeSet<u8>,
    ) -> Option<(u8, u8)> {
        (0..self.cols)
            .filter(|col| !done_cols.contains(col))
            .cartesian_product((0..self.rows).filter(|row| !done_rows.contains(row)))
            .map(|(col, row)| {
                (
                    col,
                    row,
                    *self.get(col, row).expect("index will not be out of range"),
                )
            })
            .filter(|(_, _, v)| !v.is_zero())
            .sorted_by_key(|(_, _, v)| *v)
            .map(|(col, row, _)| (col, row))
            .next()
    }

    /**
    fn: A -> (U,S,V)
    should return a matrix with at most one nonzero entry
    in every row and column, such that UA = SV.
    psuedo, because it should never switch any columns or rows,
    nor will the non zero entries be divisors of one another.
    */
    pub fn pseudo_smith(&self) -> (Self, Self, Self) {
        let mut smith = self.clone();
        let mut u = Self::identity(self.rows, self.rows);
        let mut v = Self::identity(self.cols, self.cols);
        let mut done_cols = BTreeSet::new();
        let mut done_rows = BTreeSet::new();
        for _ in 0..min(smith.rows, smith.cols) {
            if let Some((mincol, minrow)) = smith.smallest_nonzero_entry(&done_cols, &done_rows) {
                let minx = *smith.get(mincol, minrow).expect("indices in range");
                for row in (0..smith.rows).filter(|&i| i != minrow) {
                    if let Some(&x) = smith.get(mincol, row) {
                        if x.is_zero() {
                            continue;
                        }
                        let gcd = x.gcd(minx);
                        if !(x % minx).is_zero() {
                            smith.mul_row_by(row, minx.divide_by(&gcd));
                            u.mul_row_by(row, minx);
                        }
                        smith.add_muled_row_to_row(minrow, row, -x.divide_by(&gcd));
                        u.add_muled_row_to_row(minrow, row, -x.divide_by(&gcd));
                    }
                }
                for col in (0..smith.cols).filter(|&i| i != mincol) {
                    if let Some(&x) = smith.get(col, minrow) {
                        if x.is_zero() {
                            continue;
                        }
                        let gcd = x.gcd(minx);
                        if !(x % minx).is_zero() {
                            smith.mul_col_by(col, minx.divide_by(&gcd));
                            v.mul_col_by(col, minx);
                        }
                        smith.add_muled_col_to_col(mincol, col, -x.divide_by(&gcd));
                        v.add_muled_col_to_col(mincol, col, -x.divide_by(&gcd));
                    }
                }
                done_cols.insert(mincol);
                done_rows.insert(minrow);
            } else {
                break;
            }
        }
        (u, smith, v)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::rmodule::ring::{Fin, Set};
    use typenum::{U32, U6};

    #[test]
    fn transposition() {
        let a = Matrix::<u8>::from_buffer([0, 1, 2, 3, 4, 5], 3, 2);
        let b = Matrix::<u8>::from_buffer([0, 3, 1, 4, 2, 5], 2, 3);
        assert_eq!(a.transpose(), b);
        assert_eq!(b.transpose(), a);
    }

    #[test]
    fn composition() {
        let a = Matrix::<u8>::from_buffer([0, 1, 1, 0], 2, 2);
        let b = Matrix::<u8>::from_buffer([1, 0, 0, 1], 2, 2);
        assert_eq!(a.compose_unchecked(&a), b);

        let a = Matrix::<u8>::from_buffer([0, 1, 2, 0, 1, 2], 3, 2);
        let b = Matrix::<u8>::from_buffer([0, 1, 1], 1, 3);
        let c = Matrix::<u8>::from_buffer([3, 3], 1, 2);
        assert_eq!(b.compose_unchecked(&a), c);

        let a = Matrix::<u8>::from_buffer([0, 1, 2, 3, 4, 5], 3, 2);
        let b = Matrix::<u8>::from_buffer([0, 1, 2, 3, 4, 5], 2, 3);
        let c = Matrix::<u8>::from_buffer([3, 4, 5, 9, 14, 19, 15, 24, 33], 3, 3);
        let d = Matrix::<u8>::from_buffer([10, 13, 28, 40], 2, 2);
        assert_eq!(a.compose_unchecked(&b), c);
        assert_eq!(b.compose_unchecked(&a), d);
    }

    #[test]
    fn composing_from_rows() {
        assert_eq!(
            Matrix::<u8>::from_buffer([0, 1, 2, 3, 4, 5], 3, 2),
            Matrix::<u8>::from_rows(vec![vec![0, 1, 2], vec![3, 4, 5]], 2)
        );
        assert_eq!(
            Matrix::<u8>::from_buffer([0, 1, 2, 3, 4, 5], 2, 3),
            Matrix::<u8>::from_rows(vec![vec![0, 1], vec![2, 3], vec![4, 5]], 3)
        );
    }

    #[test]
    fn composing_from_cols() {
        assert_eq!(
            Matrix::<u8>::from_buffer([0, 1, 2, 3, 4, 5], 3, 2),
            Matrix::<u8>::from_cols(vec![vec![0, 3], vec![1, 4], vec![2, 5]], 3)
        );
        assert_eq!(
            Matrix::<u8>::from_buffer([0, 1, 2, 3, 4, 5], 2, 3),
            Matrix::<u8>::from_cols(vec![vec![0, 2, 4], vec![1, 3, 5]], 2)
        );
    }

    #[test]
    fn rows() {
        let m = Matrix::<u8>::from_buffer([1, 2], 1, 2);
        let mut rows = m.rows();
        assert_eq!(rows.next(), Some(vec![1]));
        assert_eq!(rows.next(), Some(vec![2]));
        assert_eq!(rows.next(), None);

        let m = Matrix::<u8>::from_buffer([1, 2], 2, 1);
        let mut rows = m.rows();
        assert_eq!(rows.next(), Some(vec![1, 2]));
        assert_eq!(rows.next(), None);

        let m = Matrix::<u8>::from_buffer([3, 4, 5, 9, 14, 19, 15, 24, 33], 3, 3);
        let mut rows = m.rows();
        assert_eq!(rows.next(), Some(vec![3, 4, 5]));
        assert_eq!(rows.next(), Some(vec![9, 14, 19]));
        assert_eq!(rows.next(), Some(vec![15, 24, 33]));
        assert_eq!(rows.next(), None);
    }

    #[test]
    fn cols() {
        let m = Matrix::<u8>::from_buffer([1, 2], 1, 2);
        let mut cols = m.cols();
        assert_eq!(cols.next(), Some(vec![1, 2]));
        assert_eq!(cols.next(), None);

        let m = Matrix::<u8>::from_buffer([1, 2], 2, 1);
        let mut cols = m.cols();
        assert_eq!(cols.next(), Some(vec![1]));
        assert_eq!(cols.next(), Some(vec![2]));
        assert_eq!(cols.next(), None);

        let m = Matrix::<u8>::from_buffer([3, 4, 5, 9, 14, 19, 15, 24, 33], 3, 3);
        let mut cols = m.cols();
        assert_eq!(cols.next(), Some(vec![3, 9, 15]));
        assert_eq!(cols.next(), Some(vec![4, 14, 24]));
        assert_eq!(cols.next(), Some(vec![5, 19, 33]));
        assert_eq!(cols.next(), None);
    }

    #[test]
    fn identities() {
        type R = Fin<U6>;
        assert_eq!(
            Matrix::<R>::identity(2, 2),
            Matrix::<R>::from_buffer([R::new(1), R::new(0), R::new(0), R::new(1)], 2, 2)
        );
        assert_eq!(
            Matrix::<R>::identity(3, 2),
            Matrix::<R>::from_buffer(
                [
                    R::new(1),
                    R::new(0),
                    R::new(0),
                    R::new(0),
                    R::new(1),
                    R::new(0)
                ],
                3,
                2
            )
        );
        assert_eq!(
            Matrix::<R>::identity(2, 3),
            Matrix::<R>::from_buffer(
                [
                    R::new(1),
                    R::new(0),
                    R::new(0),
                    R::new(1),
                    R::new(0),
                    R::new(0)
                ],
                2,
                3
            )
        );
        assert_eq!(
            Matrix::<R>::identity(3, 3),
            Matrix::<R>::from_buffer(
                [
                    R::new(1),
                    R::new(0),
                    R::new(0),
                    R::new(0),
                    R::new(1),
                    R::new(0),
                    R::new(0),
                    R::new(0),
                    R::new(1)
                ],
                3,
                3
            )
        );
    }

    #[test]
    fn muling_row_by_element() {
        type R = Fin<U6>;
        let mut m = Matrix::<R>::from_buffer(
            [
                R::new(1),
                R::new(2),
                R::new(0),
                R::new(1),
                R::new(0),
                R::new(0),
            ],
            3,
            2,
        );
        m.mul_row_by(0, R::new(3));
        assert_eq!(
            m,
            Matrix::<R>::from_buffer(
                [
                    R::new(3),
                    R::new(0),
                    R::new(0),
                    R::new(1),
                    R::new(0),
                    R::new(0),
                ],
                3,
                2,
            )
        )
    }

    #[test]
    fn muling_col_by_element() {
        type R = Fin<U6>;
        let mut m = Matrix::<R>::from_buffer(
            [
                R::new(1),
                R::new(2),
                R::new(0),
                R::new(1),
                R::new(0),
                R::new(0),
            ],
            3,
            2,
        );
        m.mul_col_by(1, R::new(2));
        assert_eq!(
            m,
            Matrix::<R>::from_buffer(
                [
                    R::new(1),
                    R::new(4),
                    R::new(0),
                    R::new(1),
                    R::new(0),
                    R::new(0),
                ],
                3,
                2,
            )
        )
    }

    #[test]
    fn adding_row_muled_by_element() {
        type R = Fin<U6>;
        let mut m = Matrix::<R>::from_buffer(
            [
                R::new(1),
                R::new(2),
                R::new(0),
                R::new(1),
                R::new(0),
                R::new(0),
            ],
            3,
            2,
        );
        m.add_muled_row_to_row(0, 1, R::new(2));
        assert_eq!(
            m,
            Matrix::<R>::from_buffer(
                [
                    R::new(1),
                    R::new(2),
                    R::new(0),
                    R::new(3),
                    R::new(4),
                    R::new(0),
                ],
                3,
                2,
            )
        )
    }

    #[test]
    fn adding_col_muled_by_element() {
        type R = Fin<U6>;
        let mut m = Matrix::<R>::from_buffer(
            [
                R::new(1),
                R::new(2),
                R::new(0),
                R::new(1),
                R::new(0),
                R::new(0),
            ],
            3,
            2,
        );
        m.add_muled_col_to_col(1, 0, R::new(2));
        assert_eq!(
            m,
            Matrix::<R>::from_buffer(
                [
                    R::new(5),
                    R::new(2),
                    R::new(0),
                    R::new(1),
                    R::new(0),
                    R::new(0),
                ],
                3,
                2,
            )
        )
    }

    #[test]
    fn finding_smallest_nonzero_entry() {
        type R = Fin<U6>;
        let m = Matrix::<R>::from_buffer(
            [
                R::new(2),
                R::new(4),
                R::new(5),
                R::new(1),
                R::new(3),
                R::new(6),
            ],
            3,
            2,
        );
        assert_eq!(
            m.smallest_nonzero_entry(&BTreeSet::new(), &BTreeSet::new()),
            Some((0, 1))
        );
        assert_eq!(
            m.smallest_nonzero_entry(&BTreeSet::from_iter([0]), &BTreeSet::new()),
            Some((1, 1))
        );
        assert_eq!(
            m.smallest_nonzero_entry(&BTreeSet::from_iter([0]), &BTreeSet::from_iter([0])),
            Some((1, 1))
        );
        assert_eq!(
            m.smallest_nonzero_entry(&BTreeSet::from_iter([0, 1]), &BTreeSet::from_iter([0])),
            None
        );
    }

    #[test]
    fn smithing_nonexample() {
        type R = Fin<U6>;
        let m = Matrix::<R>::from_buffer(
            [
                R::new(0),
                R::new(2),
                R::new(0),
                R::new(3),
                R::new(0),
                R::new(0),
            ],
            3,
            2,
        );
        let (u, s, v) = m.pseudo_smith();
        assert_eq!(s, m);
        assert_eq!(
            u,
            Matrix::<R>::from_buffer([R::new(1), R::new(0), R::new(0), R::new(1)], 2, 2)
        );
        assert_eq!(
            v,
            Matrix::<R>::from_buffer(
                [
                    R::new(1),
                    R::new(0),
                    R::new(0),
                    R::new(0),
                    R::new(1),
                    R::new(0),
                    R::new(0),
                    R::new(0),
                    R::new(1)
                ],
                3,
                3
            ),
        );
    }

    #[test]
    fn smithing() {
        type R = Fin<U32>;
        let m = Matrix::<R>::from_buffer(
            [
                R::new(2),
                R::new(5),
                R::new(6),
                R::new(4),
                R::new(3),
                R::new(7),
            ],
            3,
            2,
        );
        let (u, s, v) = m.pseudo_smith();
        assert_eq!(
            s,
            Matrix::<R>::from_buffer(
                [
                    R::new(2),
                    R::new(0),
                    R::new(0),
                    R::new(0),
                    R::new(18),
                    R::new(0)
                ],
                3,
                2
            )
        );
        assert_eq!(
            u,
            Matrix::<R>::from_buffer([R::new(1), R::new(0), R::new(30), R::new(1)], 2, 2)
        );
        assert_eq!(
            v,
            Matrix::<R>::from_buffer(
                [
                    R::new(1),
                    R::new(27),
                    R::new(25),
                    R::new(0),
                    R::new(2),
                    R::new(26),
                    R::new(0),
                    R::new(0),
                    R::new(18)
                ],
                3,
                3
            ),
        );
    }
}
