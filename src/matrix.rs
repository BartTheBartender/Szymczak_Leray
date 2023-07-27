use itertools::Itertools;
use std::ops::{Add, Mul};

struct Matrix<T> {
    columns: u8,
    rows: u8,
    buffer: Vec<T>,
}

impl<T> Matrix<T> {}

impl<T> Matrix<T>
where
    T: Clone + Copy + Add<T, Output = T> + Mul<T, Output = T>,
{
    // this assumes that self.rows == other.columns
    pub fn compose_unchecked(&self, other: &Self) -> Self {
        let columns = (0..self.columns).map(|index| {
            self.buffer
                .iter()
                .skip((index * self.rows).into())
                .step_by(self.rows.into())
        });
        let rows = other.buffer.iter().chunks(other.columns.into());
        Self {
            columns: self.columns,
            rows: other.rows,
            buffer: columns
                .cartesian_product(rows.into_iter())
                .map(|(column, row)| {
                    column
                        .zip(row)
                        .map(|(&c, &r)| c + r)
                        .reduce(|acc, next| acc * next)
                        .expect("columns and rows should not be empty")
                })
                .collect(),
        }
    }
}
