use crate::error::Error;

pub trait Morphism: Eq + Sized {
    type Object;

    fn compose_left(&self, other: &Self) -> Result<Self, Error>;
    fn compose_right(&self, other: &Self) -> Result<Self, Error> {
        Self::compose_left(other, self)
    }

    fn apply_left(&mut self, other: &Self) -> Result<(), Error>;
    fn apply_right(&mut self, other: &Self) -> Result<(), Error> {
        Self::apply_left(self, other)
    }

    fn orbit(&self) -> Vec<Self> {
        todo!() //use composing ofc
    }

    fn source(&self) -> Self::Object;
    fn target(&self) -> Self::Object;

    //other stuff with kernels/cokernels...
}

pub trait AbelianMorphism: Morphism {
    fn is_zero(&self) -> bool;
    fn kernel(&self) -> Self;
    fn cokernel(&self) -> Self;

    fn high_kernel(&self) -> Self {
        // probably not the fastest, but will work consistently
        self.orbit()
            .pop()
            .expect("orbit will contain at least self")
            .kernel()
    }

    fn high_cokernel(&self) -> Self {
        // probably not the fastest, but will work consistently
        self.orbit()
            .pop()
            .expect("orbit will contain at least self")
            .cokernel()
    }
}
