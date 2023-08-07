#![allow(unused_imports)] // DELETE LATER
use crate::{
    category::morphism::{AbelianMorphism, Compose, Morphism, PreAbelianMorphism},
    error::Error,
    matrix::Matrix,
    rmodule::{
        canon::CanonModule,
        ring::{Radix, Ring, SuperRing},
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

#[derive(Debug, PartialEq, Eq)]
pub struct CanonToCanon<R: SuperRing> {
    source: Arc<CanonModule<R>>,
    target: Arc<CanonModule<R>>,
    map: Matrix<R>,
}

impl<R: SuperRing> CanonToCanon<R> {
    pub fn new_unchecked(
        source: Arc<CanonModule<R>>,
        target: Arc<CanonModule<R>>,
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
        v: &<CanonModule<R> as Module<R>>::Element,
    ) -> <CanonModule<R> as Module<R>>::Element {
        self.target
            .element_from_matrix(v.as_matrix().compose_unchecked(&self.map))
    }

    /*
    pub fn evaluate(
        &self,
        v: &<CanonModule<R> as Module<R>>::Element,
    ) -> Result<<CanonModule<R> as Module<R>>::Element, Error> {
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

impl<R: SuperRing> Morphism<CanonModule<R>, CanonModule<R>> for CanonToCanon<R> {
    fn source(&self) -> Arc<CanonModule<R>> {
        Arc::clone(&self.source)
    }

    fn target(&self) -> Arc<CanonModule<R>> {
        Arc::clone(&self.target)
    }
}

impl<R: SuperRing> Compose<CanonModule<R>, CanonModule<R>, CanonModule<R>, Self>
    for CanonToCanon<R>
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

impl<R: SuperRing> PreAbelianMorphism<R, CanonModule<R>, CanonModule<R>> for CanonToCanon<R> {
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

impl<R: SuperRing> Add for CanonToCanon<R> {
    type Output = CanonToCanon<R>;

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

impl<R: SuperRing> Neg for CanonToCanon<R> {
    type Output = CanonToCanon<R>;

    fn neg(self) -> Self::Output {
        Self::Output {
            source: Arc::clone(&self.source),
            target: Arc::clone(&self.target),
            map: -&self.map,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::rmodule::{
        ring::{Fin, Set},
        torsion::CoeffTree,
    };
    use typenum::U6;

    #[test]
    fn kernels() {
        type R = Fin<U6>;

        let z2 = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(2)])));
        let z6 = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(0)])));
        assert_eq!(
            CanonToCanon::new_unchecked(
                Arc::clone(&z6),
                Arc::clone(&z6),
                Matrix::from_buffer([R::new(2)], 1, 1),
            )
            .kernel(),
            CanonToCanon::new_unchecked(
                Arc::clone(&z2),
                Arc::clone(&z6),
                Matrix::from_buffer([R::new(3)], 1, 1),
            )
        );

        let z6sq = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([
            R::new(0),
            R::new(0),
        ])));
        assert_eq!(
            CanonToCanon::new_unchecked(
                Arc::clone(&z6sq),
                Arc::clone(&z6sq),
                Matrix::from_buffer([R::new(2), R::new(2), R::new(3), R::new(0)], 2, 2),
            )
            .kernel(),
            CanonToCanon::new_unchecked(
                Arc::clone(&z6),
                Arc::clone(&z6sq),
                Matrix::from_buffer([R::new(2), R::new(1)], 1, 2),
            )
        );
    }

    #[test]
    fn cokernels() {
        type R = Fin<U6>;

        let z2 = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(2)])));
        let z6 = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(0)])));
        assert_eq!(
            CanonToCanon::new_unchecked(
                Arc::clone(&z6),
                Arc::clone(&z6),
                Matrix::from_buffer([R::new(2)], 1, 1),
            )
            .cokernel(),
            CanonToCanon::new_unchecked(
                Arc::clone(&z6),
                Arc::clone(&z2),
                Matrix::from_buffer([R::new(1)], 1, 1),
            )
        );

        let z6sq = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([
            R::new(0),
            R::new(0),
        ])));
        assert_eq!(
            CanonToCanon::new_unchecked(
                Arc::clone(&z6sq),
                Arc::clone(&z6sq),
                Matrix::from_buffer([R::new(2), R::new(2), R::new(3), R::new(0)], 2, 2),
            )
            .cokernel(),
            CanonToCanon::new_unchecked(
                Arc::clone(&z6sq),
                Arc::clone(&z6),
                // 90% sure this this the cokernel
                Matrix::from_buffer([R::new(1), R::new(2)], 2, 1),
            )
        );
    }
}
