use itertools::Itertools;
use std::ops::{Add, Mul, Neg};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Matrix<T> {
    cols: u8,
    rows: u8,
    buffer: Vec<T>,
}

impl<T> Matrix<T> {
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
            cols: u8::try_from(buffer.len()).expect("we're gonna need a bigger int") / nof_rows,
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
        (0..self.rows).map(|row| {
            self.buffer
                .iter()
                .skip(usize::from(row * self.cols))
                .take(usize::from(self.cols))
                .copied()
                .collect()
        })
    }

    pub fn cols(&self) -> impl Iterator<Item = Vec<T>> + '_ {
        (0..self.cols).map(|col| {
            self.buffer
                .iter()
                .skip(usize::from(col))
                .step_by(usize::from(self.cols))
                .take(usize::from(self.rows))
                .copied()
                .collect()
        })
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
                            .buffer
                            .iter()
                            .skip(usize::from(row * other.cols))
                            .take(usize::from(other.cols))
                            .zip(
                                self.buffer
                                    .iter()
                                    .skip(usize::from(col))
                                    .step_by(usize::from(self.cols))
                                    .take(usize::from(self.rows)),
                            )
                            .map(|(r, c)| *r * *c)
                            .reduce(|acc, nxt| acc + nxt)
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

impl<T> Matrix<T> {
    /**
    A -> (U,S,V)
    should return a matrix with at most one nonzero entry in every column,
    such that UA = SV.
    psuedo, because it should never switch any columns or rows
    */
    pub fn pseudo_smith(&self) -> (Self, Self, Self) {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;

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
    fn rows() {
        let m = Matrix::<u8>::from_buffer([3, 4, 5, 9, 14, 19, 15, 24, 33], 3, 3);
        let mut rows = m.rows();
        assert_eq!(rows.next(), Some(vec![3, 4, 5]));
        assert_eq!(rows.next(), Some(vec![9, 14, 19]));
        assert_eq!(rows.next(), Some(vec![15, 24, 33]));
        assert_eq!(rows.next(), None);
    }

    #[test]
    fn cols() {
        let m = Matrix::<u8>::from_buffer([3, 4, 5, 9, 14, 19, 15, 24, 33], 3, 3);
        let mut cols = m.cols();
        assert_eq!(cols.next(), Some(vec![3, 9, 15]));
        assert_eq!(cols.next(), Some(vec![4, 14, 24]));
        assert_eq!(cols.next(), Some(vec![5, 19, 33]));
        assert_eq!(cols.next(), None);
    }
}
