use crate::endocategory::morphism::*;

#[derive(Eq, PartialEq)] //i dont really understand this magical solution
pub struct Relation {
    //adjacency matrix FAST AS FUCK
}

impl Morphism for Relation {
    fn compose(&self, other: &Self) -> Self {
        todo!() //FAST AS FUCK
    }
}
