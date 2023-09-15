#![allow(unused_imports, reason = "DELETE LATER, refactoring")]
use crate::{
    category::object::{Concrete as ConcreteObject, Object},
    ralg::ring::{AdditivePartialGroup, AdditivePartialMonoid, Ring},
};
use dedup::noncon::DedupNonConAdapter;
// use gcd::Gcd;
use std::{
    borrow,
    cmp::Eq,
    collections::HashSet,
    hash::{Hash, Hasher},
    ops::{Add, Neg},
    sync::Arc,
};

/**
all the following traits should bre prefixed with Partial,
but since we are unable to provide any types which would not be partial,
this seems like a lot of work for no actual benefit other than pedantry.
*/

/*
if we ever need this trait to work between different types,
source and target can be split,
then Compose and Apply become separate traits.
however, right now my problem is that too many things have the same type,
not the other way around.
*/
pub trait Morphism<O: Object, B: borrow::Borrow<O>>: Sized {
    fn source(&self) -> B;
    fn target(&self) -> B;

    fn try_compose(self, other: Self) -> Option<Self>;
}

pub trait Enumerable<O: Object, B: borrow::Borrow<O>>: Morphism<O, B> {
    fn hom(source: B, target: B) -> impl Iterator<Item = Self> + Clone;
}

pub trait Concrete<O: ConcreteObject, B: borrow::Borrow<O>>: Morphism<O, B>
where
    O::Element: Clone,
{
    fn try_evaluate(&self, element: O::Element) -> Option<O::Element>;

    fn image(&self) -> impl Iterator<Item = O::Element> + Clone {
        self.source()
            .borrow()
            .elements()
            .filter_map(|element| self.try_evaluate(element))
            .dedup_non_con()
            // this forces references to be returned and makes liftime managfement easier
            .collect::<Vec<_>>()
            .into_iter()
    }
}

/*
pub trait EndoMorphism<Object: Eq>:
    Sized
    + Clone
    + Hash
    + PartialEq
    + Eq
    + Morphism<Object, Object>
    + Compose<Object, Object, Object, Self, Output = Self>
{
    /**
    there is a possibility, that this hash is not perfect
    which can be a huge problem if uncaught
    implementators of this trait should make sure that their hash is perfect
    */
    fn perfect_hash(&self) -> u64 {
        let mut s = std::collections::hash_map::DefaultHasher::new();
        self.hash(&mut s);
        s.finsh()
    }

    // jeśli naprawdę potrzebujesz Rc
    /*
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
    */

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
*/

pub trait PreAbelian<O: Object, B: borrow::Borrow<O>>:
    Morphism<O, B> + AdditivePartialMonoid
{
    fn kernel(&self) -> Self;
    fn cokernel(&self) -> Self;

    fn image(&self) -> Self {
        self.cokernel().kernel()
    }

    fn coimage(&self) -> Self {
        self.kernel().cokernel()
    }
}

pub trait Abelian<O: Object, B: borrow::Borrow<O>>:
    PreAbelian<O, B> + AdditivePartialGroup
{
    fn try_equaliser(self, other: Self) -> Option<Self> {
        self.try_sub(other).map(|x| x.kernel())
    }

    fn try_coequaliser(self, other: Self) -> Option<Self> {
        self.try_sub(other).map(|x| x.cokernel())
    }
}

/*
pub trait AbelianEndoMorphism<R: Ring, Object: Module<R> + Eq>:
    EndoMorphism<Object> + AbelianMorphism<R, Object, Object>
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
*/
