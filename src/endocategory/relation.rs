use crate::endocategory::morphism::*;
use std::io::Error;

#[derive(Eq, PartialEq)] //i dont really understand this magical solution
pub struct Relation {
    //adjacency matrix FAST AS FUCK
}

impl Morphism for Relation {
    fn compose_left(&self, other: &Self) -> Result<Self, Error> {
        todo!() //FAST AS FUCK
    }

    fn apply_left(&mut self, other: &Self) -> Result<&mut Self, Error> {
        todo!()
    }
}
