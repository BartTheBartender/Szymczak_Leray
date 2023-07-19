//imports from external sources
use bitvector::BitVector;
use std::io::{Error, ErrorKind};

//imports form the crate
use crate::endocategory::morphism::*;
use crate::TorsionCoeff;

#[derive(PartialEq)]
pub struct Relation {
    source: TorsionCoeff,
    target: TorsionCoeff,
    matrix_normal: BitVector,
    matrix_transpose: BitVector,
}

impl Eq for Relation {}

impl Morphism for Relation {
    fn compose_left(&self, other: &Self) -> Result<Self, Error> {
        if self.source == other.target {
            todo!()
        } else {
            Err(Error::new(
                ErrorKind::InvalidInput,
                "Cannot compose morphisms.",
            ))
        }
    }

    fn apply_left(&mut self, other: &Self) -> Result<&mut Self, Error> {
        todo!()
    }
}
