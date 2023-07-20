//imports from external sources
use bitvector::BitVector;

//imports form the crate
use crate::endocategory::morphism::*;
use crate::error::Error;
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
        if self.target == other.source {
            todo!()
        } else {
            Err(Error::SourceTargetMismatch)
        }
    }

    fn apply_left(&mut self, other: &Self) -> Result<&mut Self, Error> {
        todo!()
    }
}
