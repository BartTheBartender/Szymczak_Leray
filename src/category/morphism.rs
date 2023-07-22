use crate::{error::Error, zmodule::ZModule};
use std::{collections::HashSet, hash::Hash, rc::Rc, sync::Arc};

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
    Sized + Hash + Eq + PartialEq + Morphism<Object, Object> + Compose<Object, Object, Object, Self>
{
    fn cycle(&self) -> Vec<Self> {
        let mut seen_iterations = HashSet::new();
        let self_rc = Rc::new(&self); //i dont know if this is allowed
        seen_iterations.insert(Rc::downgrade(&self_rc));

        let temporal_cycle =
            std::iter::successors(Some(Rc::clone(&self_rc)), |curr_iteration_rc| {
                let next_iteration = self_rc.compose_unchecked(curr_iteration_rc);

                if seen_iterations.contains(&next_iteration) {
                    None
                } else {
                    let next_iteration_rc = Rc::new(next_iteration);
                    seen_iterations.insert(Rc::downgrade(&next_iteration_rc));
                    Some(next_iteration_rc)
                }
            })
            .collect();

        //
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
