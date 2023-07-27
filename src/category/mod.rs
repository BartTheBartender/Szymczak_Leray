//pub mod map;
pub mod morphism;
pub mod relation;
// pub mod torsion_coeff;

use crate::category::morphism::{Endomorphism, Morphism};
use std::collections::HashMap;

pub type HomSet<Object, M> = HashMap<Object, HashMap<Object, Vec<M>>>;

pub struct Category<Object: Eq, M: Morphism<Object, Object>> {
    pub hom_sets: HomSet<Object, M>,
}
