use crate::{error::Error, zmodule::ZModule};
use std::rc::Rc;

pub trait Morphism<Source, Target> {
    fn source(&self) -> Rc<Source>;
    fn target(&self) -> Rc<Target>;
}

pub trait Compose<Source, Middle: Eq, Target, Lhs: Morphism<Middle, Target>>:
    Morphism<Source, Middle>
{
    type Output: Morphism<Source, Target>;

    fn compose_unchecked(&self, other: &Lhs) -> Self::Output;

    fn compose(&self, other: &Lhs) -> Result<Self::Output, Error> {
        match self.target() == other.source() {
            true => Ok(self.compose_unchecked(other)),
            false => Err(Error::SourceTargetMismatch),
        }
    }

    // musiałem pozbyć się `apply`, bo się genericsy nie zgadzały
    // ale można dopisać nowego traita na to gdzie `Middle` i `Target` są tym samym
}

pub trait EndoMorphism<Object: Eq>:
    Sized + Morphism<Object, Object> + Compose<Object, Object, Object, Self>
{
    fn orbit(&self) -> Vec<Self> {
        todo!() // use composing ofc
    }
}

pub trait AbelianMorphism<Source: ZModule, Target: ZModule>: Morphism<Source, Target> {
    fn is_zero(&self) -> bool;
    fn kernel(&self) -> Self;
    fn cokernel(&self) -> Self;
}

pub trait AbelianEndoMorphism<Object: ZModule + Eq>:
    EndoMorphism<Object> + AbelianMorphism<Object, Object>
{
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
