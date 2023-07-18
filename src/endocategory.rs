//submodules
pub mod map;
pub mod morphism;
//pub mod relation;
pub mod torsion_coeff;

//imports from external sources
use rayon::prelude::*;
use std::collections::HashMap;

//imports from the crate
use crate::endocategory::morphism::*;
use crate::TorsionCoeff;

pub struct Endocategory<M: Morphism> {
    hom_sets: HashMap<(TorsionCoeff, TorsionCoeff), Vec<M>>,
}

impl<M: Morphism> Endocategory<M> {
    pub fn generate_orbits(&self) -> HashMap<TorsionCoeff, Vec<HashMap<M, Vec<M>>>> {
        todo!()
    }

    fn generate_orbits_hom_set(hom_set: &Vec<M>) -> Vec<M> {
        todo!()
    }
}
