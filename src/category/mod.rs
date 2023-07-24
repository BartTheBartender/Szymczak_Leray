//pub mod map;
pub mod morphism;
pub mod relation;
// pub mod torsion_coeff;

use crate::category::morphism::{EndoMorphism, Morphism};
use std::collections;

pub struct Category<Object: Eq, M: Morphism<Object, Object>> {
    pub hom_sets: collections::HashMap<(Object, Object), collections::HashSet<M>>,
}

impl<Object: Eq, M: Morphism<Object, Object>> Category<Object, M> {
    pub fn all_endomorphisms<E: EndoMorphism<Object>>(
        &self,
    ) -> collections::HashMap<Object, collections::HashSet<E>> {
        todo!()
    }
}
