//submodules
mod morphism;
mod relation;

//imports from external sources
use std::collections::HashMap;

//imports from crate
use crate::endocategory::morphism::*;
use crate::TorsionCoeff;

pub struct Endocategory<M: Morphism> {
    hom_set: HashMap<(TorsionCoeff, TorsionCoeff), Vec<M>>,
    //do you think we should add all possible pairs of TorsionCoeffs here?
}

impl<M: Morphism> Endocategory<M> {
    pub fn generate_orbit_endocategory(&self) -> HashMap<TorsionCoeff, Vec<(M, Vec<M>)>> {
        todo!()
        //for torsion_coeff in some Vec<TorsionCoeff> paralelly call generate_orbits_hom_set and insert into output
    }

    pub fn generate_orbit_hom_set(&self, torsion_coeff: TorsionCoeff) -> Vec<(M, Vec<M>)> {
        todo!()

        //for every morphism generate_orbit_hom_set calls generate_orbit in Morphism trait (i wish i had function overloading... ;D )
    }
}
