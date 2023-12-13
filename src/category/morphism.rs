use crate::{
    category::object::{Concrete as ConcreteObject, Object},
    ralg::ring::{AdditivePartialGroup, AdditivePartialMonoid},
};
use dedup::noncon::DedupNonConAdapter;
use std::borrow::Borrow;

/**
all the following traits should bre prefixed with Partial,
but since we are unable to provide any types which would not be partial,
this seems like a lot of work for no actual benefit other than pedantry.
*/

/*
if we ever need this trait to work between different types,
source and target can be split,
then Compose becomes a separate trait.
however, right now my problem is that too many things have the same type,
not the other way around.
*/
pub trait Morphism<O: Object>: Sized + Clone + Eq {
    type B: Borrow<O>;

    fn source(&self) -> Self::B;
    fn target(&self) -> Self::B;

    fn identity(object: Self::B) -> Self;
    fn is_iso(&self) -> bool;

    unsafe fn compose_unchecked(&self, other: &Self) -> Self;
    fn try_compose(&self, other: &Self) -> Option<Self> {
        (self.target().borrow() == other.source().borrow())
            .then_some(unsafe { self.compose_unchecked(other) }) // safe since we just checked if composable
    }

    fn is_endo(&self) -> bool {
        self.source().borrow() == self.target().borrow()
    }

    fn try_cycle(&self) -> Option<Vec<Self>> {
        self.is_endo().then_some({
            let mut seen_iterations = Vec::new(); // this could be a HashSet or a BTreeSet if Morphism implemented Hash or Ord

            seen_iterations.push(Self::identity(self.source()));
            std::iter::successors(Some(Self::identity(self.source())), |current_iteration| {
                // compose safe, since endo is self composable
                let next_iteration = unsafe { current_iteration.clone().compose_unchecked(self) };
                match seen_iterations.contains(&next_iteration) {
                    true => None,
                    false => {
                        seen_iterations.push(next_iteration.clone());
                        Some(next_iteration)
                    }
                }
            })
            .collect()
        })
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

// these should be only defined for relations
pub trait IsMap<O: Object>: Morphism<O> {
    fn is_a_map(&self) -> bool;
}

pub trait IsMatching<O: Object>: Morphism<O> {
    fn is_a_matching(&self) -> bool;
}

pub trait IsWide<O: Object>: Morphism<O> {
    fn is_wide(&self) -> bool;
}
