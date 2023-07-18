//imports from external sources
use std::collections::HashMap;

//imports from the crate
use crate::endocategory;
use crate::endocategory::morphism::Morphism;
use crate::TorsionCoeff;

pub struct SzymczakCategory<M: Morphism> {
    classes: Vec<HashMap<TorsionCoeff, Vec<M>>>,
}

impl<M: Morphism> SzymczakCategory<M> {
    pub fn new(endocategory: endocategory::Endocategory<M>) -> Self {
        todo!()
    }
}
