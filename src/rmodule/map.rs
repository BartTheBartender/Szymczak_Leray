#![allow(unused_imports)] // DELETE LATER
use crate::{
    category::morphism::{AbelianMorphism, Compose, Morphism, PreAbelianMorphism},
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
use std::{
    collections::HashMap,
    ops::{Add, Neg, Rem},
    sync::Arc,
};

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

impl<RC: Radix, R: SuperRing<RC>> PreAbelianMorphism<RC, R, CanonModule<RC, R>, CanonModule<RC, R>>
    for CanonToCanon<RC, R>
{
    fn is_zero(&self) -> bool {
        self.map.iter().all(|e| e.is_zero())
    }

    fn kernel(&self) -> Self {
        // need smiths for that
        todo!()
    }

    fn cokernel(&self) -> Self {
        // need smiths for that
        todo!()
    }
}

impl<RC: Radix, R: SuperRing<RC>> Add for &CanonToCanon<RC, R> {
    type Output = CanonToCanon<RC, R>;

    /**
    this assumes that both self and output have the same source and target.
    we could panic otherwise, but that would require checking
    and therefore slow us down
    */
    fn add(self, other: Self) -> Self::Output {
        Self::Output {
            source: Arc::clone(&self.source),
            target: Arc::clone(&other.target),
            map: &self.map + &other.map,
        }
    }
}

impl<RC: Radix, R: SuperRing<RC>> Neg for &CanonToCanon<RC, R> {
    type Output = CanonToCanon<RC, R>;

    fn neg(self) -> Self::Output {
        Self::Output {
            source: Arc::clone(&self.source),
            target: Arc::clone(&self.target),
            map: -&self.map,
        }
    }
}
