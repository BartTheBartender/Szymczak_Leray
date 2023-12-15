use crate::ralg::{
    module::{canon::element::Element as CanonElement, Module},
    ring::{
        ideal::Ideal, AdditivePartialGroup, AdditivePartialMonoid, Bezout as BezoutRing, Demesne,
        MultiplicativePartialMonoid, Ring,
    },
};
use itertools::Itertools;
use std::{
    cmp,
    collections::{BTreeSet, VecDeque},
    fmt,
};

/* # two dimensional container */

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct VecD2<T> {
    pub nof_cols: usize,
    pub nof_rows: usize,
    buffer: Vec<T>,
}

/* ## debug and display */

impl<T: fmt::Debug> fmt::Debug for VecD2<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "V2({:?}x{:?}){:?}",
            self.nof_cols, self.nof_rows, self.buffer
        )
    }
}

impl<T: fmt::Display> fmt::Display for VecD2<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "V2 [\n{}\n]",
            self.rows()
                .map(|row| {
                    let mut rstr = "  [".to_owned();
                    rstr.push_str(&row.map(std::string::ToString::to_string).join(", "));
                    rstr.push(']');
                    rstr
                })
                .join("\n")
        )
    }
}

/* ## contianer operations */

impl<T> VecD2<T> {
    /* # constructors */

    pub fn from_buffer<I>(buffer: I, nof_cols: usize, nof_rows: usize) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        Self {
            nof_cols,
            nof_rows,
            buffer: Vec::from_iter(buffer),
        }
    }

    pub fn from_rows<I>(rows: I, nof_rows: usize) -> Self
    where
        I: IntoIterator<Item = Vec<T>>,
    {
        let buffer = rows.into_iter().concat();
        let nof_cols = match nof_rows {
            0 => 0,
            x => buffer.len().div_euclid(x),
        };
        Self::from_buffer(buffer, nof_cols, nof_rows)
    }

    pub fn from_rows_custom<I>(rows: I, nof_cols: usize, nof_rows: usize) -> Self
    where
        I: IntoIterator<Item = Vec<T>>,
    {
        Self::from_buffer(rows.into_iter().concat(), nof_cols, nof_rows)
    }

    pub fn from_cols<I>(cols: I, nof_cols: usize) -> Self
    where
        I: IntoIterator<Item = Vec<T>>,
    {
        Self::from_rows(cols, nof_cols).transpose()
    }

    pub fn from_cols_custom<I>(cols: I, nof_cols: usize, nof_rows: usize) -> Self
    where
        I: IntoIterator<Item = Vec<T>>,
    {
        Self::from_rows_custom(cols, nof_rows, nof_cols).transpose()
    }

    /* # getters */

    pub fn get(&self, col: usize, row: usize) -> Option<&T> {
        self.buffer
            .get(col.wrapping_add(self.nof_cols.wrapping_mul(row)))
    }

    pub fn get_mut(&mut self, col: usize, row: usize) -> Option<&mut T> {
        self.buffer
            .get_mut(col.wrapping_add(self.nof_cols.wrapping_mul(row)))
    }

    pub fn get_minor<I, J>(&self, cols: I, rows: J) -> VecD2<&T>
    where
        I: IntoIterator<Item = usize> + Clone,
        J: IntoIterator<Item = usize> + Clone,
        <I as IntoIterator>::IntoIter: Clone,
    {
        VecD2 {
            nof_cols: cols
                .clone()
                .into_iter()
                .filter(|&col| col < self.nof_cols)
                .count(),
            nof_rows: rows
                .clone()
                .into_iter()
                .filter(|&row| row < self.nof_rows)
                .count(),
            buffer: rows
                .into_iter()
                .cartesian_product(cols)
                .filter_map(|(row, col)| self.get(col, row))
                .collect(),
        }
    }

    /*
    pub fn get_minor_mut<I, J>(&mut self, cols: I, rows: J) -> VecD2<&mut T>
    where
        I: IntoIterator<Item = usize> + Clone,
        J: IntoIterator<Item = usize> + Clone,
        <I as IntoIterator>::IntoIter: Clone,
    {
        VecD2 {
            nof_cols: cols
                .clone()
                .into_iter()
                .filter(|&col| col < self.nof_cols)
                .count(),
            nof_rows: rows
                .clone()
                .into_iter()
                .filter(|&row| row < self.nof_rows)
                .count(),
            buffer: rows
                .into_iter()
                .cartesian_product(cols)
                .filter_map(|(row, col)| self.get_mut(col, row))
                .collect(),
        }
    }
    */

    /* ## iterators */

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.buffer.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.buffer.iter_mut()
    }

    pub fn into_iter(self) -> impl Iterator<Item = T> {
        self.buffer.into_iter()
    }

    fn row(&self, row: usize) -> impl Iterator<Item = &T> {
        self.buffer
            .iter()
            .skip(row.wrapping_mul(self.nof_cols))
            .take(self.nof_cols)
    }

    fn into_row(self, row: usize) -> impl Iterator<Item = T> {
        self.buffer
            .into_iter()
            .skip(row.wrapping_mul(self.nof_cols))
            .take(self.nof_cols)
    }

    pub fn row_mut(&mut self, row: usize) -> impl Iterator<Item = &mut T> {
        self.buffer
            .iter_mut()
            .skip(row.wrapping_mul(self.nof_cols))
            .take(self.nof_cols)
    }

    pub fn rows(&self) -> impl Iterator<Item = impl Iterator<Item = &T>> {
        (0..self.nof_rows).map(|row| self.row(row))
    }

    fn col(&self, col: usize) -> impl Iterator<Item = &T> {
        self.buffer
            .iter()
            .skip(col)
            .step_by(self.nof_cols)
            .take(self.nof_rows)
    }

    fn into_col(self, col: usize) -> impl Iterator<Item = T> {
        self.buffer
            .into_iter()
            .skip(col)
            .step_by(self.nof_cols)
            .take(self.nof_rows)
    }

    pub fn col_mut(&mut self, col: usize) -> impl Iterator<Item = &mut T> {
        self.buffer
            .iter_mut()
            .skip(col)
            .step_by(self.nof_cols)
            .take(self.nof_rows)
    }

    pub fn cols(&self) -> impl Iterator<Item = impl Iterator<Item = &T>> {
        (0..self.nof_cols).map(|col| self.col(col))
    }

    /* # properties */

    pub const fn shape(&self) -> (usize, usize) {
        (self.nof_cols, self.nof_rows)
    }

    pub const fn is_square(&self) -> bool {
        self.nof_cols == self.nof_rows
    }

    /* # transformations */

    pub fn transpose(self) -> Self {
        Self {
            nof_cols: self.nof_rows,
            nof_rows: self.nof_cols,
            buffer: self
                .buffer
                .into_iter()
                .zip((0..self.nof_rows).cartesian_product(0..self.nof_cols))
                .sorted_by_key(|&(ref _t, (row, col))| (col, row))
                .map(|(t, _crd)| t)
                .collect(),
        }
    }
}

impl<T: Clone> VecD2<T> {
    fn cloned(&self) -> Self {
        Self {
            nof_cols: self.nof_cols,
            nof_rows: self.nof_rows,
            buffer: self.buffer.clone(),
        }
    }
}

impl<T: Copy> VecD2<T> {
    fn copied(&self) -> Self {
        Self {
            nof_cols: self.nof_cols,
            nof_rows: self.nof_rows,
            buffer: self.buffer.clone(),
        }
    }

    pub fn get_copied(&self, col: usize, row: usize) -> Option<T> {
        self.buffer
            .get(col.wrapping_add(self.nof_cols.wrapping_mul(row)))
            .copied()
    }

    pub fn get_minor_copied<I, J>(&self, cols: I, rows: J) -> Self
    where
        I: IntoIterator<Item = usize> + Clone,
        J: IntoIterator<Item = usize> + Clone,
        <I as IntoIterator>::IntoIter: Clone,
    {
        Self {
            nof_cols: cols
                .clone()
                .into_iter()
                .filter(|&col| col < self.nof_cols)
                .count(),
            nof_rows: rows
                .clone()
                .into_iter()
                .filter(|&row| row < self.nof_rows)
                .count(),
            buffer: rows
                .into_iter()
                .cartesian_product(cols)
                .filter_map(|(row, col)| self.get_copied(col, row))
                .collect(),
        }
    }

    fn row_copied(&self, row: usize) -> impl Iterator<Item = T> + '_ {
        self.buffer
            .iter()
            .skip(row.wrapping_mul(self.nof_cols))
            .take(self.nof_cols)
            .copied()
    }

    fn col_copied(&self, col: usize) -> impl Iterator<Item = T> + '_ {
        self.buffer
            .iter()
            .skip(col)
            .step_by(self.nof_cols)
            .take(self.nof_rows)
            .copied()
    }
}

/* # matrix */

#[allow(type_alias_bounds, reason = "waiting on feature `lazy_type_alias`")]
pub type Matrix<R: Ring> = VecD2<R>;

/* ## conversion */
// TODO : move this as Into to CanonElement

impl<R: Ring, I: Ideal<Parent = R> + Ord> From<CanonElement<R, I>> for Matrix<R> {
    fn from(element: CanonElement<R, I>) -> Self {
        Self::from_cols([element.into_values().collect()], 1)
    }
}

/* ## additive structure */

impl<R: Ring> Demesne for Matrix<R> {}

impl<R: Ring> AdditivePartialMonoid for Matrix<R> {
    fn try_add(self, other: Self) -> Option<Self> {
        (self.shape() == other.shape()).then_some(Self::from_buffer(
            self.buffer
                .into_iter()
                .zip(other.buffer)
                .map(|(x, y)| x.add(y)),
            self.nof_cols,
            other.nof_rows,
        ))
    }

    fn own_zero(&self) -> Self {
        Self::from_buffer(
            (0..self.nof_cols.saturating_mul(self.nof_rows)).map(|_| R::zero()),
            self.nof_cols,
            self.nof_rows,
        )
    }

    fn is_zero(&self) -> bool {
        self.buffer.iter().all(R::is_zero)
    }

    fn is_negable(&self) -> bool {
        true
    }

    fn try_neg(self) -> Option<Self> {
        Some(self.neg())
    }
}

impl<R: Ring> AdditivePartialGroup for Matrix<R> {
    fn neg(self) -> Self {
        Self::from_buffer(
            self.buffer.into_iter().map(R::neg),
            self.nof_cols,
            self.nof_rows,
        )
    }

    fn neg_inplace(&mut self) {
        self.buffer.iter_mut().for_each(R::neg_inplace);
    }
}

/* ## multiplicative structure */

impl<R: Ring + Copy> Matrix<R> {
    pub fn identity(nof_cols: usize, nof_rows: usize) -> Self {
        Self::from_buffer(
            (0..nof_rows).flat_map(|r| {
                (0..nof_cols).map(move |c| match r == c {
                    true => R::one(),
                    false => R::zero(),
                })
            }),
            nof_cols,
            nof_rows,
        )
    }

    /// helper mostly to check how much time is spent on this
    const fn are_composable(left: &Self, right: &Self) -> bool {
        left.nof_rows == right.nof_cols
    }

    /**
    returns other * self.
    will crash if the dimensions are invalid
    */
    pub unsafe fn compose(&self, other: &Self) -> Self {
        Self {
            nof_cols: self.nof_cols,
            nof_rows: other.nof_rows,
            buffer: (0..other.nof_rows)
                .flat_map(|row| {
                    (0..self.nof_cols).map(move |col| {
                        other
                            .row_copied(row)
                            .zip(self.col_copied(col))
                            .map(|(r, c)| r.mul(c))
                            .reduce(R::add)
                            .unwrap_or_else(R::zero)
                    })
                })
                .collect(),
        }
    }

    /// returns other * self if composable
    pub fn try_compose(&self, other: &Self) -> Option<Self> {
        // safe since we first check the necessay condition
        Self::are_composable(self, other).then_some(unsafe { self.compose(other) })
    }

    #[allow(
        clippy::multiple_unsafe_ops_per_block,
        reason = "both have the same safety condition"
    )]
    pub fn try_left_inverse(&self) -> Option<Self> {
        // (A^T * A)^(−1) * A^T
        let transpose = self.clone().transpose();
        // matrix is always composable with its transpose
        unsafe {
            self.compose(&transpose)
                .try_inv()
                .map(|square| transpose.compose(&square))
        }
    }

    #[allow(
        clippy::multiple_unsafe_ops_per_block,
        reason = "both have the same safety condition"
    )]
    pub fn try_right_inverse(&self) -> Option<Self> {
        // A^T * (A * A^T)^(−1)
        let transpose = self.clone().transpose();
        // matrix is always composable with its transpose
        unsafe {
            transpose
                .compose(self)
                .try_inv()
                .map(|square| square.compose(&transpose))
        }
    }
}

impl<R: Ring + Copy> MultiplicativePartialMonoid for Matrix<R> {
    fn try_mul(self, other: Self) -> Option<Self> {
        self.try_compose(&other)
    }

    fn own_one(&self) -> Self {
        let (nof_cols, nof_rows) = self.shape();
        Self::identity(nof_cols, nof_rows)
    }

    fn is_one(&self) -> bool {
        *self == self.own_one()
    }

    fn is_invable(&self) -> bool {
        self.try_characteristic_polynomial()
            .and_then(|poly| poly.first().map(MultiplicativePartialMonoid::is_invable))
            .is_some()
    }

    fn try_inv(self) -> Option<Self> {
        self.try_characteristic_polynomial().and_then(|p| {
            let size = self.shape().0; // if char_poly returns Some, then self is square
            let (determinant, coeffs) = p
                .split_first()
                .expect("char polynomial will have at least one term");
            determinant.try_inv().map(|dinv| {
                coeffs
                    .iter()
                    .enumerate()
                    .map(|(j, c)| {
                        self.clone()
                            .try_pow(j)
                            .expect("square matrix will be self composable")
                            .mul(*c)
                    })
                    .reduce(|acc, next| {
                        acc.try_add(next)
                            .expect("all matrices have the same shape as self")
                    })
                    .unwrap_or_else(|| self.own_one())
                    .mul(dinv.mul(R::one().neg().pow(size - 1)))
            })
        })
    }
}

impl<R: Ring + Copy> Module<R> for Matrix<R> {
    fn mul(self, r: R) -> Self {
        Self {
            nof_cols: self.nof_cols,
            nof_rows: self.nof_rows,
            buffer: self
                .buffer
                .into_iter()
                .map(|element| element.mul(r))
                .collect(),
        }
    }

    fn mul_assign(&mut self, r: R) {
        self.buffer = self.buffer.iter().map(|element| element.mul(r)).collect();
    }
}

/* ## matrix operations */

impl<R: Ring + Copy> Matrix<R> {
    fn mul_row_by(&mut self, row: usize, r: R) {
        for v in self.row_mut(row) {
            v.mul_assign(r);
        }
    }

    fn mul_col_by(&mut self, col: usize, r: R) {
        for v in self.col_mut(col) {
            v.mul_assign(r);
        }
    }

    fn add_muled_row_to_row(&mut self, muled_row: usize, to_row: usize, r: R) {
        let mrow: Vec<_> = self.row(muled_row).copied().collect();
        for (t, m) in self.row_mut(to_row).zip(mrow) {
            t.add_assign(m.mul(r));
        }
    }

    fn add_muled_col_to_col(&mut self, muled_col: usize, to_col: usize, r: R) {
        let mcol: Vec<_> = self.col(muled_col).copied().collect();
        for (t, m) in self.col_mut(to_col).zip(mcol) {
            t.add_assign(m.mul(r));
        }
    }
}

/* ## characteristic polynomial */

impl<R: Ring + Copy> Matrix<R> {
    /**
    produces a toeplitz matrix with zeros above the diagonal.
    toeplitz means every diagonal is constant
    */
    pub fn toeplitz_lower_triangular<I: Iterator<Item = R>>(first_col: I, nof_cols: usize) -> Self {
        let first_col_deq: VecDeque<R> = first_col.collect();
        Self::from_cols(
            (0..nof_cols).map(|col_index| {
                let mut col = first_col_deq.clone();
                for _j in 0..col_index {
                    col.pop_back();
                    col.push_front(R::zero());
                }
                col.into()
            }),
            nof_cols,
        )
    }

    /// this assumes the matrix is square, otherwise may produce undefined behaviour or panic
    #[allow(
        clippy::arithmetic_side_effects,
        reason = "i am to lazy to do this correctly now"
    )]
    unsafe fn characteristic_polynomial_unchecked(&self) -> Vec<R> {
        let size = self.nof_cols;
        let matrix_m = |j: usize| -> Self { self.get_minor_copied((j + 1)..size, (j + 1)..size) };
        let matrix_r = |j: usize| -> Self { self.get_minor_copied((j + 1)..=size, [j]) };
        let matrix_s = |j: usize| -> Self { self.get_minor_copied([j], (j + 1)..=size) };
        let matrix_c = |j: usize| -> Self {
            Self::toeplitz_lower_triangular(
                (0..(size + 1 - j)).map(|i| match i {
                    0 => R::one(),
                    1 => (*self.get(j, j).expect("j will be smaller than size")).neg(),
                    ith => matrix_s(j)
                        .compose(
                            &matrix_m(j)
                                .try_pow(ith - 2)
                                .expect("square matrix will be self composable"),
                        )
                        .compose(&matrix_r(j))
                        .get(0, 0)
                        .expect("the resulting matrix should have shape (1,1)")
                        .neg(),
                }),
                size - j,
            )
        };
        // this is rahter ugly, maybe it can be done better
        (0..size)
            .rev()
            .map(matrix_c)
            .reduce(|acc, next| acc.compose(&next))
            .unwrap()
            .buffer
            // we want the degree zero coefficient to be on the front of the iterator
            .into_iter()
            .rev()
            .collect()
    }

    /**
    will only produce result for square matrices.
    uses Berkowitz's algorithm
    */
    pub fn try_characteristic_polynomial(&self) -> Option<Vec<R>> {
        // safe since we first check the necessary condition
        unsafe {
            self.is_square()
                .then_some(self.characteristic_polynomial_unchecked())
        }
    }
}

/* ## smithing */

impl<R: Copy + BezoutRing + Into<u16>> Matrix<R> {
    fn find_smallest_nonzero_entry(
        &self,
        done_cols: &BTreeSet<usize>,
        done_rows: &BTreeSet<usize>,
    ) -> Option<(R, usize, usize)> {
        (0..self.nof_cols)
            .filter(|col| !done_cols.contains(col))
            .cartesian_product(
                (0..self.nof_rows)
                    .rev()
                    .filter(|row| !done_rows.contains(row)),
            )
            .map(|(col, row)| (*self.get(col, row).unwrap_or(&R::zero()), col, row))
            .filter(|&(v, _, _)| !v.is_zero())
            .sorted_by_key(|&(v, _, _)| {
                <R as Into<u16>>::into(v).min(<R as Into<u16>>::into(v.neg()))
            })
            .next()
    }

    /**
    fn: A -> (U,S,V)
    should return a matrix with at most one nonzero entry
    in every row and column, such that UA = SV.
    psuedo, because it should never switch any columns or rows,
    nor will the non zero entries be divisors of one another.
    */
    #[allow(clippy::panic, reason = "structural guarantees")]
    pub fn pseudo_smith(&self) -> (Self, Self, Self) {
        let mut smith = self.clone();
        let mut u = Self::identity(self.nof_rows, self.nof_rows);
        let mut v = Self::identity(self.nof_cols, self.nof_cols);
        let mut done_cols = BTreeSet::new();
        let mut done_rows = BTreeSet::new();

        for _ in 0..cmp::min(smith.nof_rows, smith.nof_cols) {
            if let Some((minx, mincol, minrow)) =
                smith.find_smallest_nonzero_entry(&done_cols, &done_rows)
            {
                for row in (0..smith.nof_rows).filter(|&i| i != minrow) {
                    if let Some(&x) = smith.get(mincol, row) {
                        if x.is_zero() {
                            continue;
                        }
                        let (gcd, _, _) = R::gcd(x, minx);
                        if let Some(muland) = minx.try_divide(gcd).next() {
                            smith.mul_row_by(row, muland);
                            u.mul_row_by(row, muland);
                        }
                        if let Some(muland) = x.try_divide(gcd).next().map(R::neg) {
                            smith.add_muled_row_to_row(minrow, row, muland);
                            u.add_muled_row_to_row(minrow, row, muland);
                        }
                    }
                }
                for col in (0..smith.nof_cols).filter(|&i| i != mincol) {
                    if let Some(&x) = smith.get(col, minrow) {
                        if x.is_zero() {
                            continue;
                        }
                        let (gcd, _, _) = R::gcd(x, minx);
                        if let Some(muland) = minx.try_divide(gcd).next() {
                            smith.mul_col_by(col, muland);
                            v.mul_col_by(col, muland);
                        }
                        if let Some(muland) = x.try_divide(gcd).next().map(R::neg) {
                            smith.add_muled_col_to_col(mincol, col, muland);
                            v.add_muled_col_to_col(mincol, col, muland);
                        }
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

// - - -

use std::ops;
impl<T> VecD2<T>
where
    T: Copy + ops::BitOr<T, Output = T> + ops::BitAnd<T, Output = T>,
{
    // this assumes that self.rows == other.cols
    pub fn compose_unchecked_bool(&self, other: &Self) -> Self {
        Self {
            nof_cols: self.nof_cols,
            nof_rows: other.nof_rows,
            buffer: (0..other.nof_rows)
                .flat_map(|row| {
                    (0..self.nof_cols).map(move |col| {
                        other
                            .row(row)
                            .zip(self.col(col))
                            .map(|(r, c)| *r & *c)
                            .reduce(|acc, nxt| acc | nxt)
                            // this is not neccesarily true and could be replaced by an `unwrap_or_else` call
                            .expect("matrices are not empty")
                    })
                })
                .collect(),
        }
    }

    pub fn buffer(&self) -> Vec<T> {
        self.buffer.clone()
    }
}

// - - -

/* # tests */
#[cfg(test)]
mod test {
    #![allow(
        clippy::shadow_unrelated,
        reason = "two tests in one unit need not two different names"
    )]

    use super::*;
    use crate::ralg::cgroup::C;
    use typenum::{U32, U4, U54, U6, U7};

    /* # 2d container */

    #[test]
    fn transposing() {
        let a = VecD2::<u8>::from_buffer([0, 1, 2, 3, 4, 5], 3, 2);
        let b = VecD2::<u8>::from_buffer([0, 3, 1, 4, 2, 5], 2, 3);
        assert_eq!(a.transpose(), b);

        let a = VecD2::<u8>::from_buffer([0, 1, 2, 3, 4, 5], 3, 2);
        assert_eq!(b.transpose(), a);
    }

    #[test]
    fn creating_from_rows() {
        assert_eq!(
            VecD2::<u8>::from_buffer([0, 1, 2, 3, 4, 5], 3, 2),
            VecD2::<u8>::from_rows(vec![vec![0, 1, 2], vec![3, 4, 5]], 2)
        );
        assert_eq!(
            VecD2::<u8>::from_buffer([0, 1, 2, 3, 4, 5], 2, 3),
            VecD2::<u8>::from_rows(vec![vec![0, 1], vec![2, 3], vec![4, 5]], 3)
        );
    }

    #[test]
    fn creating_from_cols() {
        assert_eq!(
            VecD2::<u8>::from_buffer([0, 1, 2, 3, 4, 5], 3, 2),
            VecD2::<u8>::from_cols(vec![vec![0, 3], vec![1, 4], vec![2, 5]], 3)
        );
        assert_eq!(
            VecD2::<u8>::from_buffer([0, 1, 2, 3, 4, 5], 2, 3),
            VecD2::<u8>::from_cols(vec![vec![0, 2, 4], vec![1, 3, 5]], 2)
        );
    }

    #[test]
    fn reading_rows() {
        use std::iter::Iterator;

        let m = VecD2::<u8>::from_buffer([1, 2], 1, 2);
        let mut rows = m.rows();
        assert_eq!(rows.next().map(Iterator::collect), Some(vec![&1]));
        assert_eq!(rows.next().map(Iterator::collect), Some(vec![&2]));
        assert!(rows.next().is_none());

        let m = VecD2::<u8>::from_buffer([1, 2], 2, 1);
        let mut rows = m.rows();
        assert_eq!(rows.next().map(Iterator::collect), Some(vec![&1, &2]));
        assert!(rows.next().is_none());

        let m = VecD2::<u8>::from_buffer([3, 4, 5, 9, 14, 19, 15, 24, 33], 3, 3);
        let mut rows = m.rows();
        assert_eq!(rows.next().map(Iterator::collect), Some(vec![&3, &4, &5]));
        assert_eq!(rows.next().map(Iterator::collect), Some(vec![&9, &14, &19]));
        assert_eq!(
            rows.next().map(Iterator::collect),
            Some(vec![&15, &24, &33])
        );
        assert!(rows.next().is_none());
    }

    #[test]
    fn reading_cols() {
        let m = VecD2::<u8>::from_buffer([1, 2], 1, 2);
        let mut cols = m.cols();
        assert_eq!(cols.next().map(Iterator::collect), Some(vec![&1, &2]));
        assert!(cols.next().is_none());

        let m = VecD2::<u8>::from_buffer([1, 2], 2, 1);
        let mut cols = m.cols();
        assert_eq!(cols.next().map(Iterator::collect), Some(vec![&1]));
        assert_eq!(cols.next().map(Iterator::collect), Some(vec![&2]));
        assert!(cols.next().is_none());

        let m = VecD2::<u8>::from_buffer([3, 4, 5, 9, 14, 19, 15, 24, 33], 3, 3);
        let mut cols = m.cols();
        assert_eq!(cols.next().map(Iterator::collect), Some(vec![&3, &9, &15]));
        assert_eq!(cols.next().map(Iterator::collect), Some(vec![&4, &14, &24]));
        assert_eq!(cols.next().map(Iterator::collect), Some(vec![&5, &19, &33]));
        assert!(cols.next().is_none());
    }

    #[test]
    fn reading_minors() {
        let m = VecD2::<u8>::from_buffer([3, 4, 5, 9, 14, 19, 15, 24, 33], 3, 3);
        assert_eq!(
            m.get_minor([0], [0, 2]),
            VecD2::<&u8>::from_buffer([&3, &15], 1, 2)
        );
        assert_eq!(
            m.get_minor([0, 1], [1, 2]),
            VecD2::<&u8>::from_buffer([&9, &14, &15, &24], 2, 2)
        );
        assert_eq!(
            m.get_minor([3], [4, 6]),
            VecD2::<&u8>::from_buffer([], 0, 0)
        );
    }

    /* # matrices */

    /* ## additive structure */

    #[test]
    fn zeros() {
        type R = C<U6>;

        let a = Matrix::<R>::from_buffer([1, 0, 0, 1].map(R::from), 2, 2);
        let z = a.own_zero();
        assert_eq!(z, Matrix::<R>::from_buffer([0, 0, 0, 0].map(R::from), 2, 2));
        assert!(z.is_zero());
    }

    #[test]
    fn addition() {
        type R = C<U6>;

        let a = Matrix::<R>::from_buffer([0, 1, 0, 1].map(R::from), 2, 2);
        assert_eq!(a.clone().try_add(a.own_zero()), Some(a.clone()));

        let b = Matrix::<R>::from_buffer([1, 0, 0, 1].map(R::from), 2, 2);
        let c = Matrix::<R>::from_buffer([1, 1, 0, 2].map(R::from), 2, 2);
        assert_eq!(a.clone().try_add(b), Some(c));

        let b = Matrix::<R>::from_buffer([1, 0, 0, 1, 0, 0].map(R::from), 3, 2);
        assert_eq!(a.try_add(b), None);
    }

    #[test]
    fn negation() {
        type R = C<U6>;

        let a = Matrix::<R>::from_buffer([0, 1, 0, 1].map(R::from), 2, 2);
        let b = Matrix::<R>::from_buffer([0, 5, 0, 5].map(R::from), 2, 2);
        assert_eq!(a.clone().neg(), b);
        assert_eq!(Some(a.own_zero()), a.clone().try_add(a.neg()));
    }

    /* ## multiplicative structure */

    #[test]
    fn identities() {
        type R = C<U6>;
        assert_eq!(
            Matrix::<R>::identity(2, 2),
            Matrix::<R>::from_buffer([1, 0, 0, 1].map(R::from), 2, 2)
        );
        assert_eq!(
            Matrix::<R>::identity(3, 2),
            Matrix::<R>::from_buffer([1, 0, 0, 0, 1, 0].map(R::from), 3, 2)
        );
        assert_eq!(
            Matrix::<R>::identity(2, 3),
            Matrix::<R>::from_buffer([1, 0, 0, 1, 0, 0].map(R::from), 2, 3)
        );
        assert_eq!(
            Matrix::<R>::identity(3, 3),
            Matrix::<R>::from_buffer([1, 0, 0, 0, 1, 0, 0, 0, 1].map(R::from), 3, 3)
        );
    }

    #[test]
    fn composing() {
        type R = C<U54>;
        let a = Matrix::<R>::from_buffer([0, 1, 1, 0].map(R::from), 2, 2);
        let b = Matrix::<R>::from_buffer([1, 0, 0, 1].map(R::from), 2, 2);
        assert_eq!(a.try_compose(&a), Some(b));

        let a = Matrix::<R>::from_buffer([0, 1, 2, 0, 1, 2].map(R::from), 3, 2);
        let b = Matrix::<R>::from_buffer([0, 1, 1].map(R::from), 1, 3);
        let c = Matrix::<R>::from_buffer([3, 3].map(R::from), 1, 2);
        assert_eq!(b.try_compose(&a), Some(c));

        let a = Matrix::<R>::from_buffer([0, 1, 2, 3, 4, 5].map(R::from), 3, 2);
        let b = Matrix::<R>::from_buffer([0, 1, 2, 3, 4, 5].map(R::from), 2, 3);
        let c = Matrix::<R>::from_buffer([3, 4, 5, 9, 14, 19, 15, 24, 33].map(R::from), 3, 3);
        let d = Matrix::<R>::from_buffer([10, 13, 28, 40].map(R::from), 2, 2);
        assert_eq!(a.try_compose(&b), Some(c));
        assert_eq!(b.try_compose(&a), Some(d));
    }

    /* ## elementary operations */

    #[test]
    fn muling_row_by_element() {
        type R = C<U6>;
        let mut m = Matrix::<R>::from_buffer([1, 2, 0, 1, 0, 0].map(R::from), 3, 2);
        let res = Matrix::<R>::from_buffer([3, 0, 0, 1, 0, 0].map(R::from), 3, 2);
        m.mul_row_by(0, R::from(3));
        assert_eq!(m, res);
    }

    #[test]
    fn muling_col_by_element() {
        type R = C<U6>;
        let mut m = Matrix::<R>::from_buffer([1, 2, 0, 1, 0, 0].map(R::from), 3, 2);
        let res = Matrix::<R>::from_buffer([1, 4, 0, 1, 0, 0].map(R::from), 3, 2);
        m.mul_col_by(1, R::from(2));
        assert_eq!(m, res);
    }

    #[test]
    fn adding_row_muled_by_element() {
        type R = C<U6>;
        let mut m = Matrix::<R>::from_buffer([1, 2, 0, 1, 0, 0].map(R::from), 3, 2);
        m.add_muled_row_to_row(0, 1, R::from(2));
        let res = Matrix::<R>::from_buffer([1, 2, 0, 3, 4, 0].map(R::from), 3, 2);
        assert_eq!(m, res);
    }

    #[test]
    fn adding_col_muled_by_element() {
        type R = C<U6>;
        let mut m = Matrix::<R>::from_buffer([1, 2, 0, 1, 0, 0].map(R::from), 3, 2);
        m.add_muled_col_to_col(1, 0, R::from(2));
        let res = Matrix::<R>::from_buffer([5, 2, 0, 1, 0, 0].map(R::from), 3, 2);
        assert_eq!(m, res);
    }

    /* ## characteristic polynomial */

    #[test]
    fn toeplitz() {
        type R = C<U32>;
        assert_eq!(
            Matrix::<R>::toeplitz_lower_triangular([1, 2, 3].map(R::from).into_iter(), 3),
            Matrix::<R>::from_buffer([1, 0, 0, 2, 1, 0, 3, 2, 1].map(R::from), 3, 3),
        );

        assert_eq!(
            Matrix::<R>::toeplitz_lower_triangular([1, 2, 3].map(R::from).into_iter(), 2),
            Matrix::<R>::from_buffer([1, 0, 2, 1, 3, 2].map(R::from), 2, 3),
        );
    }

    #[test]
    fn characteristic_polynomial() {
        type R = C<U32>;
        assert_eq!(
            Matrix::<R>::from_buffer([1, 0, 0, 1].map(R::from), 2, 2)
                .try_characteristic_polynomial(),
            Some([1, 30, 1].map(R::from).to_vec())
        );
        assert_eq!(
            Matrix::<R>::from_buffer([1, 2, 3, 4].map(R::from), 2, 2)
                .try_characteristic_polynomial(),
            Some([30, 27, 1].map(R::from).to_vec())
        );
        assert_eq!(
            Matrix::<R>::from_buffer([1, 0, 0, 0, 1, 0, 0, 0, 1].map(R::from), 3, 3)
                .try_characteristic_polynomial(),
            Some([31, 3, 29, 1].map(R::from).to_vec())
        );
        assert_eq!(
            Matrix::<R>::from_buffer([1, 2, 3, 2, 5, 8, 3, 5, 7].map(R::from), 3, 3)
                .try_characteristic_polynomial(),
            Some([0, 26, 19, 1].map(R::from).to_vec())
        );
        assert_eq!(
            Matrix::<R>::from_buffer(
                [1, 2, 3, 4, 2, 3, 4, 1, 3, 4, 1, 2, 4, 1, 2, 3].map(R::from),
                4,
                4
            )
            .try_characteristic_polynomial(),
            Some([0, 0, 4, 24, 1].map(R::from).to_vec())
        );
    }

    /* ## smithing */

    #[test]
    fn finding_smallest_nonzero_entry() {
        type R = C<U6>;
        let m = Matrix::<R>::from_buffer([2, 4, 5, 1, 3, 6].map(R::from), 3, 2);
        assert_eq!(
            m.find_smallest_nonzero_entry(&BTreeSet::new(), &BTreeSet::new()),
            Some((R::from(1), 0, 1))
        );
        assert_eq!(
            m.find_smallest_nonzero_entry(&BTreeSet::from_iter([0]), &BTreeSet::new()),
            Some((R::from(5), 2, 0))
        );
        assert_eq!(
            m.find_smallest_nonzero_entry(&BTreeSet::from_iter([0]), &BTreeSet::from_iter([0])),
            Some((R::from(3), 1, 1))
        );
        assert_eq!(
            m.find_smallest_nonzero_entry(&BTreeSet::from_iter([0, 1]), &BTreeSet::from_iter([0])),
            None
        );
    }

    #[test]
    fn smithing_nonexample() {
        type R = C<U6>;
        let m = Matrix::<R>::from_buffer([0, 2, 0, 3, 0, 0].map(R::from), 3, 2);
        let (u, s, v) = m.pseudo_smith();
        assert_eq!(s, m);
        assert_eq!(u, Matrix::<R>::from_buffer([1, 0, 0, 1].map(R::from), 2, 2));
        assert_eq!(
            v,
            Matrix::<R>::from_buffer([1, 0, 0, 0, 1, 0, 0, 0, 1].map(R::from), 3, 3),
        );
    }

    #[test]
    fn smithing_easy() {
        type R = C<U6>;
        let m = Matrix::<R>::from_buffer([1, 1, 1, 1].map(R::from), 2, 2);
        let (u, s, v) = m.pseudo_smith();
        assert_eq!(s, Matrix::<R>::from_buffer([0, 0, 1, 0].map(R::from), 2, 2));
        assert_eq!(u, Matrix::<R>::from_buffer([1, 5, 0, 1].map(R::from), 2, 2));
        assert_eq!(v, Matrix::<R>::from_buffer([1, 5, 0, 1].map(R::from), 2, 2));
    }

    #[test]
    fn smithing_apparently_hard() {
        type R = C<U7>;
        let m = Matrix::<R>::from_buffer([6, 4].map(R::from), 2, 1);
        let (u, s, v) = m.pseudo_smith();
        assert_eq!(s, Matrix::<R>::from_buffer([6, 0].map(R::from), 2, 1));
        assert_eq!(u, Matrix::<R>::from_buffer([1].map(R::from), 1, 1));
        assert_eq!(v, Matrix::<R>::from_buffer([1, 5, 0, 3].map(R::from), 2, 2));
    }

    #[test]
    fn smithing_ordering_really_matters_one() {
        type R = C<U4>;
        let m = Matrix::<R>::from_buffer([3, 3, 0, 0, 1, 0, 1, 3].map(R::from), 4, 2);
        let (u, s, v) = m.pseudo_smith();
        assert_eq!(
            s,
            Matrix::<R>::from_buffer([0, 3, 0, 0, 1, 0, 0, 0].map(R::from), 4, 2)
        );
        assert_eq!(u, Matrix::<R>::from_buffer([1, 1, 0, 1].map(R::from), 2, 2));
        assert_eq!(
            v,
            Matrix::<R>::from_buffer(
                [1, 0, 1, 1, 0, 1, 3, 3, 0, 0, 3, 0, 0, 0, 0, 1].map(R::from),
                4,
                4
            )
        );
    }

    #[test]
    fn smithing_ordering_really_matters_two() {
        type R = C<U4>;
        let m = Matrix::<R>::from_buffer([3, 1, 1, 0, 0, 3, 2, 3].map(R::from), 4, 2);
        let (u, s, v) = m.pseudo_smith();
        assert_eq!(
            s,
            Matrix::<R>::from_buffer([3, 0, 0, 0, 0, 1, 0, 0].map(R::from), 4, 2)
        );
        assert_eq!(u, Matrix::<R>::from_buffer([1, 0, 0, 1].map(R::from), 2, 2));
        assert_eq!(
            v,
            Matrix::<R>::from_buffer(
                [1, 3, 1, 3, 0, 3, 2, 3, 0, 0, 3, 0, 0, 0, 0, 1].map(R::from),
                4,
                4
            )
        );
    }

    #[test]
    fn smithing_medium() {
        type R = C<U32>;
        let m = Matrix::<R>::from_buffer([2, 5, 6, 4, 3, 7].map(R::from), 3, 2);
        let (u, s, v) = m.pseudo_smith();
        assert_eq!(
            s,
            Matrix::<R>::from_buffer([2, 0, 0, 0, 0, 27].map(R::from), 3, 2)
        );
        assert_eq!(
            u,
            Matrix::<R>::from_buffer([1, 0, 30, 1].map(R::from), 2, 2)
        );
        assert_eq!(
            v,
            Matrix::<R>::from_buffer([1, 23, 29, 0, 6, 0, 0, 30, 1].map(R::from), 3, 3),
        );
    }
}
