pub mod morphism;
// pub mod relation; UNCOMMENT THIS

//imports from external sources
// use rayon::prelude::*;
use std::collections::{HashMap, HashSet};

//imports from the crate

use crate::category::morphism::*;
use crate::TorsionCoeff;

pub struct Endocategory<Source, Target, M: Morphism<Source, Target>> {
    hom_sets: HashMap<(Source, Target), HashSet<M>>,
}

/*
impl<Source, Target, M: Morphism<Source, Target>> Endocategory<Source, Target, M> {
    pub fn generate_orbits(&self) -> HashMap<TorsionCoeff, Vec<HashMap<M, Vec<M>>>> {
        todo!()
    }

    fn generate_orbits_hom_set(hom_set: &Vec<M>) -> Vec<M> {
        todo!()
    }
}
*/
