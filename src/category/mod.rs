//pub mod map;
pub mod morphism;
pub mod relation;
// pub mod torsion_coeff;

use crate::category::morphism::{EndoMorphism, Morphism};
use std::collections::HashMap;

pub struct Category<Object: Eq, M: Morphism<Object, Object>> {
    pub hom_sets: HashMap<(Object, Object), Vec<M>>,
}
