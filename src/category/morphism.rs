use crate::{error::Error, zmodule::ZModule};
use std::{
    collections::HashSet,
    hash::{Hash, Hasher},
    rc::Rc,
    sync::Arc,
};

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
    Sized
    + Clone
    + Hash
    + PartialEq
    + Eq
    + Morphism<Object, Object>
    + Compose<Object, Object, Object, Self, Output = Self>
{
    fn perfect_hash(&self) -> u64 {
        // there is a possibility, that this hash is not perfect
        // which can be a huge problem if uncaught
        // implementators of this trait should make sure that their hash is perfect
        let mut s = std::collections::hash_map::DefaultHasher::new();
        self.hash(&mut s);
        s.finish()
    }

    // jeśli naprawdę potrzebujesz Rc
    fn cycle_rc(&self) -> Vec<Rc<Self>> {
        let mut seen_iterations = HashSet::new();

        seen_iterations.insert(self.perfect_hash());
        std::iter::successors(Some(Rc::new(self.clone())), |current_iteration| {
            let next_iteration = current_iteration.compose_unchecked(self);
            let next_iteration_hash = next_iteration.perfect_hash();
            match seen_iterations.contains(&next_iteration_hash) {
                true => None,
                false => {
                    seen_iterations.insert(next_iteration_hash);
                    Some(Rc::new(next_iteration))
                }
            }
        })
        .collect()
    }

    fn cycle(&self) -> Vec<Self> {
        // nie ma potrzeby trzymać całego morfizmu, wystarczy perfekcyjny hash
        let mut seen_iterations = HashSet::new();

        seen_iterations.insert(self.perfect_hash());
        std::iter::successors(Some(self.clone()), |current_iteration| {
            let next_iteration = current_iteration.compose_unchecked(self);
            let next_iteration_hash = next_iteration.perfect_hash();
            match seen_iterations.contains(&next_iteration_hash) {
                true => None,
                false => {
                    seen_iterations.insert(next_iteration_hash);
                    Some(next_iteration)
                }
            }
        })
        .collect()
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
