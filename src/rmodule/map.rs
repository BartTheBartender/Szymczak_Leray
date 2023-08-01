#![allow(unused_imports)] // DELETE LATER
use crate::{
    category::morphism::{AbelianMorphism, Compose, Morphism},
    error::Error,
    matrix::Matrix,
    rmodule::{
        canon::CanonModule,
        ring::{Radix, Ring, SuperRing},
        // coset::{Coset, CosetModule<R>},
        Module,
    },
    util::iterator::Dedup,
};
use std::{collections::HashMap, ops::Rem, sync::Arc};

/* # Canon to Canon */

#[derive(PartialEq, Eq)]
pub struct CanonToCanon<RC: Radix, R: SuperRing<RC>> {
    source: Arc<CanonModule<RC, R>>,
    target: Arc<CanonModule<RC, R>>,
    map: Matrix<R>,
}

impl<RC: Radix, R: SuperRing<RC>> CanonToCanon<RC, R> {
    pub fn new_unchecked(
        source: Arc<CanonModule<RC, R>>,
        target: Arc<CanonModule<RC, R>>,
        map: Matrix<R>,
    ) -> Self {
        Self {
            source,
            target,
            map,
        }
    }

    /*
    pub fn new(
        source: Arc<CanonModule<R>>,
        target: Arc<CanonModule<R>>,
        map: Matrix<<CanonModule<R> as Module<R>>::Element>,
    ) -> Result<Self, Error> {
        match map.iter().all(|el| target.is_element(el)) {
            true => Ok(Self::new_unchecked(source, target, map)),
            false => Err(Error::InvalidElement),
        }
    }
    */

    pub fn evaluate_unchecked(
        &self,
        v: &<CanonModule<RC, R> as Module<RC, R>>::Element,
    ) -> <CanonModule<RC, R> as Module<RC, R>>::Element {
        self.target
            .element_from_matrix(v.as_matrix().compose_unchecked(&self.map))
    }

    /*
    pub fn evaluate(
        &self,
        v: &<CanonModule<RC, R> as Module<RC, R>>::Element,
    ) -> Result<<CanonModule<RC, R> as Module<RC, R>>::Element, Error> {
        match self.source.as_ref().is_element(v) {
            true => Ok(self.evaluate_unchecked(v)),
            false => Err(Error::InvalidElement),
        }
    }
    */

    /*
    pub fn image(&self) -> Vec<<CanonModule<R> as Module<R>>::Element> {
        let mut im: Vec<_> = self
            .source()
            .all_elements()
            .iter()
            .map(|element| self.evaluate_unchecked(element))
            .collect();
        im.clear_duplicates();
        im
    }
    */

    /*
    pub fn epis(source: Arc<CanonModule<R>>, target: Arc<CanonModule<R>>) -> Vec<CanonToCanon<R>> {
        if target.dimension() > source.dimension() {
            return Vec::new();
        }
        // find all generating sets of the target
        // for every such set, create all maps from the canonical generators of the source onto this set

        // maybe we can do this by dual goursat
        todo!()
    }
    */
}

impl<RC: Radix, R: SuperRing<RC>> Morphism<CanonModule<RC, R>, CanonModule<RC, R>>
    for CanonToCanon<RC, R>
{
    fn source(&self) -> Arc<CanonModule<RC, R>> {
        Arc::clone(&self.source)
    }

    fn target(&self) -> Arc<CanonModule<RC, R>> {
        Arc::clone(&self.target)
    }
}

impl<RC: Radix, R: SuperRing<RC>>
    Compose<CanonModule<RC, R>, CanonModule<RC, R>, CanonModule<RC, R>, Self>
    for CanonToCanon<RC, R>
{
    type Output = Self;

    fn compose_unchecked(&self, other: &Self) -> Self {
        Self::new_unchecked(
            Arc::clone(&self.source),
            Arc::clone(&other.target),
            self.map.compose_unchecked(&other.map),
        )
    }
}

impl<RC: Radix, R: SuperRing<RC>> AbelianMorphism<RC, R, CanonModule<RC, R>, CanonModule<RC, R>>
    for CanonToCanon<RC, R>
{
    fn is_zero(&self) -> bool {
        self.map.iter().all(|e| e.is_zero())
    }

    fn kernel(&self) -> Self {
        // we can probably do this with smithing
        todo!()
    }

    fn cokernel(&self) -> Self {
        todo!()
    }
}

/* # Coset to Canon */

/*
#[derive(PartialEq, Eq)]
pub struct CosetToCanon {
    source: Arc<CosetModule<R><CanonModule<R>>>,
    target: Arc<CanonModule<R>>,
    map: HashMap<Coset<CanonModule<R>>, <CanonModule<R> as Module<R>>::Element>,
}

impl CosetToCanon {
    pub fn new(
        source: Arc<CosetModule<R><CanonModule<R>>>,
        target: Arc<CanonModule<R>>,
        map: HashMap<Coset<CanonModule<R>>, <CanonModule<R> as Module<R>>::Element>,
    ) -> Self {
        Self {
            source,
            target,
            map,
        }
    }

    fn evaluate(
        &self,
        element: &Coset<CanonModule<R>>,
    ) -> Result<<CanonModule<R> as Module<R>>::Element, Error> {
        self.map.get(element).ok_or(Error::PartialMap).cloned()
    }
}

impl Morphism<CosetModule<R><CanonModule<R>>, CanonModule<R>> for CosetToCanon {
    fn source(&self) -> Arc<CosetModule<R><CanonModule<R>>> {
        Arc::clone(&self.source)
    }

    fn target(&self) -> Arc<CanonModule<R>> {
        Arc::clone(&self.target)
    }
}
*/

/* # Canon to Coset */

/*
pub struct CanonToCoset {
    source: Arc<CanonModule<R>>,
    target: Arc<CosetModule<R><CanonModule<R>>>,
    map: HashMap<<CanonModule<R> as Module<R>>::Element, Coset<CanonModule<R>>>,
}

impl CanonToCoset {
    pub fn new(
        source: Arc<CanonModule<R>>,
        target: Arc<CosetModule<R><CanonModule<R>>>,
        map: HashMap<<CanonModule<R> as Module<R>>::Element, Coset<CanonModule<R>>>,
    ) -> Self {
        Self {
            source,
            target,
            map,
        }
    }

    fn evaluate(
        &self,
        element: &<CanonModule<R> as Module<R>>::Element,
    ) -> Result<Coset<CanonModule<R>>, Error> {
        self.map.get(element).ok_or(Error::PartialMap).cloned()
    }
}

impl Morphism<CanonModule<R>, CosetModule<R><CanonModule<R>>> for CanonToCoset {
    fn source(&self) -> Arc<CanonModule<R>> {
        Arc::clone(&self.source)
    }

    fn target(&self) -> Arc<CosetModule<R><CanonModule<R>>> {
        Arc::clone(&self.target)
    }
}
*/

/* # Canon to Coset to Canon */

/*
impl Compose<CanonModule<R>, CosetModule<R><CanonModule<R>>, CanonModule<R>, CosetToCanon>
    for CanonToCoset
{
    type Output = CanonToCanon;

    fn compose_unchecked(&self, other: &CosetToCanon) -> Self::Output {
        CanonToCanon::new_unchecked(
            Arc::clone(&self.source),
            Arc::clone(&other.target),
            self.source()
                .generators()
                .iter()
                .flat_map(|generator| {
                    other.evaluate(
                        &self
                            .evaluate(generator)
                            .expect("this not really unchecked but it is wayyy too late"),
                    )
                })
                .collect(),
        )
    }
}
*/
