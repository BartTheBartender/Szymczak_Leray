use std::io::Error;

pub trait Morphism: PartialEq + Eq + Sized {
    fn compose_left(&self, other: &Self) -> Result<Self, Error>;
    fn compose_right(&self, other: &Self) -> Result<Self, Error> {
        Self::compose_left(other, self)
    }

    fn apply_left(&mut self, other: &Self) -> Result<&mut Self, Error>;
    fn apply_right(&mut self, other: &Self) -> Result<&mut Self, Error> {
        Self::apply_left(self, other)
    }

    fn orbit(&self) -> Vec<Self> {
        todo!() //use composing ofc
    }

    //other stuff with kernels/cokernels...
}
