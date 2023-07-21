use crate::{
    category::morphism::{Compose, Morphism},
    error::Error,
    zmodule::canon::CanonZModule,
    TorsionCoeff,
};

use bitvector::BitVector;
use std::rc::Rc;

pub struct Relation {
    source: CanonZModule,
    target: CanonZModule,
    matrix_normal: BitVector,
    matrix_transpose: BitVector,
}

impl PartialEq for Relation {
    fn eq(&self, other: &Self) -> bool {
        self.matrix_normal == other.matrix_normal
    }
}

impl Morphism<CanonZModule, CanonZModule> for Relation {
    fn source(&self) -> Rc<CanonZModule> {
        Rc::new(&self.source);
        todo!()
    }

    fn target(&self) -> Rc<CanonZModule> {
        Rc::new(&self.target);
        todo!()
    }
}

impl Compose<CanonZModule, CanonZModule, CanonZModule, Relation> for Relation {
    type Output = Relation;

    fn compose_unchecked(&self, other: &Relation) -> Self::Output {
        todo!()
    }
}
