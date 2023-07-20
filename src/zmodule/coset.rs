#![allow(unused_imports)]

use crate::{
    util::{iterator::Dedup, number::versor},
    zmodule::{canon::CanonZModule, map::CosetToCanon, ZModule, Zahl},
};
use core::hash::Hash;
use itertools::Itertools;
use std::{collections::HashMap, rc::Rc};

/* # elements */

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Coset<Parent>
where
    Parent: ZModule + Clone,
    Parent::Element: Clone,
{
    // could this be a hash set ?
    pub set: Vec<Parent::Element>,
}

impl<Parent> Coset<Parent>
where
    Parent: ZModule + Clone,
    Parent::Element: Clone,
{
    pub fn new(elements: Vec<Parent::Element>) -> Self {
        Self { set: elements }
    }
}

/* # zmod */

#[derive(Clone, PartialEq, Eq)]
pub struct CosetZModule<Parent>
where
    Parent: ZModule + Clone + Eq,
    Parent::Element: Clone + Eq,
{
    buffer: Vec<Coset<Parent>>,
    parent: Rc<Parent>,
}

impl<Parent> CosetZModule<Parent>
where
    Parent: ZModule + Clone + Eq,
    Parent::Element: Clone + Eq,
{
    pub fn new(buffer: Vec<Coset<Parent>>, parent: Rc<Parent>) -> Self {
        Self { buffer, parent }
    }

    fn coset_with_element(&self, element: &Parent::Element) -> Option<Coset<Parent>> {
        self.buffer
            .iter()
            .find(|coset| coset.set.iter().any(|el| el == element))
            .cloned()
    }
}

impl CosetZModule<CanonZModule> {
    fn generators(&self) -> Vec<<Self as ZModule>::Element> {
        let mut gens: Vec<_> = self
            .parent
            .generators()
            .iter()
            .map(|generator| self.coset_with_element(generator).expect("TODO"))
            .collect();
        gens.clear_duplicates();
        gens
    }

    fn generate_cycle(
        &self,
        generator: <Self as ZModule>::Element,
    ) -> Vec<<Self as ZModule>::Element> {
        let mut next = generator.clone();
        let zero = self.zero(); // for comparisons, just in case it is expensive to build one
        let mut cycle = Vec::new();
        while next != zero {
            cycle.push(next.clone());
            next = self.add_unchecked(&next, &generator);
        }
        cycle
    }

    pub fn canonise(self) -> CosetToCanon {
        // this can surely be done better, since this is fucking naive

        let cycles = self
            .generators()
            .into_iter()
            .map(|generator| self.generate_cycle(generator))
            .sorted_by_key(|v| v.len())
            .rev()
            .collect::<Vec<_>>();
        let target = CanonZModule::new(
            cycles
                .iter()
                .map(|v| {
                    <usize as TryInto<u8>>::try_into(v.len())
                        .expect("we're gonna need a bigger int")
                        + 1
                })
                .collect(),
        );
        let dim = target.dimension();
        let mut map = HashMap::new();
        for (index, cycle) in cycles.iter().enumerate() {
            let v = versor(index, dim);
            for (jndex, el) in cycle.iter().enumerate() {
                map.insert(
                    el.clone(),
                    target.mul_by_scalar_unchecked(
                        jndex.try_into().expect("we're gonna need a bigger int"),
                        &v,
                    ),
                );
            }
        }
        map.insert(self.zero(), target.zero());
        CosetToCanon::new(Rc::new(self), Rc::new(target), map)
    }
}

impl<Parent> ZModule for CosetZModule<Parent>
where
    Parent: ZModule + Clone + Eq,
    Parent::Element: Clone + Eq,
{
    type Element = Coset<Parent>;

    fn zero(&self) -> Self::Element {
        self.coset_with_element(&self.parent.zero())
            .expect("zero will be in some coset")
    }

    fn is_element(&self, v: &Self::Element) -> bool {
        v.set.iter().all(|el| self.parent.as_ref().is_element(el))
    }

    fn add_unchecked(&self, v: &Self::Element, u: &Self::Element) -> Self::Element {
        self.coset_with_element(&self.parent.add_unchecked(
            v.set.first().expect("coset should not be empty"),
            u.set.first().expect("coset should not be empty"),
        ))
        .expect("will not go out of scope")
    }

    fn increment_unchecked(&self, v: &mut Self::Element, u: &Self::Element) {
        // this is fucking lazy, but i am tired
        // TODO fix later
        *v = self.add_unchecked(v, u)
    }

    fn mul_by_scalar_unchecked(&self, x: Zahl, v: &Self::Element) -> Self::Element {
        self.coset_with_element(
            &self
                .parent
                .mul_by_scalar_unchecked(x, v.set.first().expect("coset should not be empty")),
        )
        .expect("will not go out of scope")
    }
}
