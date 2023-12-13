use crate::{
    category::object::{Concrete as ConcreteObject, Object},
    ralg::ring::{AdditivePartialGroup, AdditivePartialMonoid},
};
use dedup::noncon::DedupNonConAdapter;
use std::{borrow::Borrow, collections::HashSet, hash::Hash};

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
pub trait Morphism<O: Object>: Sized {
    type B: Borrow<O>;

    fn source(&self) -> Self::B;
    fn target(&self) -> Self::B;

    fn compose(&self, other: &Self) -> Self;
    fn try_compose(&self, other: &Self) -> Option<Self> {
        (self.target().borrow() == other.source().borrow()).then_some(self.compose(other))
    }
}

pub trait Enumerable<O: Object>: Morphism<O> {
    fn hom(source: Self::B, target: Self::B) -> impl Iterator<Item = Self> + Clone;
}

pub trait Concrete<O: ConcreteObject>: Morphism<O>
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

pub trait Endo<O: Object>: Morphism<O> + Clone + Eq + Hash {
    // get rid of this trait, incorporate into Morphism
    fn identity(object: Self::B) -> Self;

    fn try_cycle(&self) -> Option<Vec<Self>> {
        // nie ma potrzeby trzymać całego morfizmu, wystarczy perfekcyjny hash
        (self.source().borrow() == self.target().borrow()).then_some({
            let mut seen_iterations = HashSet::new();

            seen_iterations.insert(Self::identity(self.source()));
            std::iter::successors(Some(Self::identity(self.source())), |current_iteration| {
                let next_iteration = current_iteration
                    .clone()
                    .try_compose(self)
                    .expect("endo should be self composable");
                match seen_iterations.contains(&next_iteration) {
                    true => None,
                    false => {
                        seen_iterations.insert(next_iteration.clone());
                        Some(next_iteration)
                    }
                }
            })
            .collect()
        })
    }
}

pub trait PreAbelian<O: Object>: Morphism<O> + AdditivePartialMonoid {
    fn kernel(&self) -> Self;
    fn cokernel(&self) -> Self;

    fn image(&self) -> Self {
        self.cokernel().kernel()
    }

    fn coimage(&self) -> Self {
        self.kernel().cokernel()
    }
}

pub trait Abelian<O: Object>: PreAbelian<O> + AdditivePartialGroup {
    fn try_equaliser(self, other: Self) -> Option<Self> {
        self.try_sub(other).map(|x| x.kernel())
    }

    fn try_coequaliser(self, other: Self) -> Option<Self> {
        self.try_sub(other).map(|x| x.cokernel())
    }
}

pub trait IsMap<O: Object>: Morphism<O> {
    fn is_a_map(&self) -> bool;
}

pub trait IsMatching<O: Object>: Morphism<O> {
    fn is_a_matching(&self) -> bool;
}

pub trait IsWide<O: Object>: Morphism<O> {
    fn is_wide(&self) -> bool;
}

pub trait IsBij<O: Object>: Morphism<O> {
    fn is_a_bijection(&self) -> bool;
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
