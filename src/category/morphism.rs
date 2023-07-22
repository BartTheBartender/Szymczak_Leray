use crate::{error::Error, zmodule::ZModule};
use std::{collections::HashSet, hash::Hash, sync::Arc};

pub trait Morphism<Source, Target> {
    fn source(&self) -> Arc<Source>;
    fn target(&self) -> Arc<Target>;
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
    Sized + Hash + Eq + Clone + Morphism<Object, Object> + Compose<Object, Object, Object, Self>
{
    fn cycle(&self) -> Vec<Self> {
        /*
        let mut seen_iterations: HashSet<Self> = HashSet::new();

        seen_iterations.insert(self.clone());

        std::iter::successors(Some(self.clone()), |curr_iteration| {
            let next_iteration = self.compose_unchecked(&curr_iteration);
            if seen_iterations.contains(&next_iteration) {
                None
            } else {
                seen_iterations.insert(next_iteration.clone());
                Some(next_iteration)
            }
        })
        .collect()
        */
        todo!()
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
        self.cycle()
            .pop()
            .expect("cycle will contain at least one iteration")
            .kernel()
    }

    fn high_cokernel(&self) -> Self {
        // probably not the fastest, but will work consistently
        self.cycle()
            .pop()
            .expect("cycle will contain at least one iteration")
            .cokernel()
    }
}
