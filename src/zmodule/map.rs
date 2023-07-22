use crate::{
    category::morphism::{Compose, Morphism},
    error::Error,
    util::iterator::Dedup,
    zmodule::{
        canon::CanonZModule,
        coset::{Coset, CosetZModule},
        ZModule,
    },
};
use std::{collections::HashMap, sync::Arc};

/* # Canon to Canon */

#[derive(PartialEq, Eq)]
pub struct CanonToCanon {
    source: Arc<CanonZModule>,
    target: Arc<CanonZModule>,
    map: Vec<<CanonZModule as ZModule>::Element>,
}

impl CanonToCanon {
    pub fn new_unchecked(
        source: Arc<CanonZModule>,
        target: Arc<CanonZModule>,
        map: Vec<<CanonZModule as ZModule>::Element>,
    ) -> Self {
        Self {
            source,
            target,
            map,
        }
    }

    pub fn new(
        source: Arc<CanonZModule>,
        target: Arc<CanonZModule>,
        map: Vec<<CanonZModule as ZModule>::Element>,
    ) -> Result<Self, Error> {
        match map.iter().all(|el| target.is_element(el)) {
            true => Ok(Self::new_unchecked(source, target, map)),
            false => Err(Error::InvalidElement),
        }
    }

    pub fn evaluate_unchecked(
        &self,
        v: &<CanonZModule as ZModule>::Element,
    ) -> <CanonZModule as ZModule>::Element {
        self.map
            .iter()
            .zip(v.iter())
            .map(|(output, gen)| self.source.as_ref().mul_by_scalar_unchecked(*gen, output))
            .reduce(|acc, next| self.source.as_ref().add_unchecked(&acc, &next))
            .expect("element will not be empty")
    }

    pub fn evaluate(
        &self,
        v: &<CanonZModule as ZModule>::Element,
    ) -> Result<<CanonZModule as ZModule>::Element, Error> {
        match self.source.as_ref().is_element(v) {
            true => Ok(self.evaluate_unchecked(v)),
            false => Err(Error::InvalidElement),
        }
    }

    pub fn image(&self) -> Vec<<CanonZModule as ZModule>::Element> {
        let mut im: Vec<_> = self
            .source()
            .all_elements()
            .iter()
            .map(|element| self.evaluate_unchecked(element))
            .collect();
        im.clear_duplicates();
        im
    }
}

impl Morphism<CanonZModule, CanonZModule> for CanonToCanon {
    fn source(&self) -> Arc<CanonZModule> {
        Arc::clone(&self.source)
    }

    fn target(&self) -> Arc<CanonZModule> {
        Arc::clone(&self.target)
    }
}

impl Compose<CanonZModule, CanonZModule, CanonZModule, Self> for CanonToCanon {
    type Output = Self;

    fn compose_unchecked(&self, other: &Self) -> Self {
        Self::new_unchecked(
            Arc::clone(&self.source),
            Arc::clone(&other.target),
            self.map
                .iter()
                .map(|element| other.evaluate_unchecked(element))
                .collect(),
        )
    }
}

/* # Coset to Canon */

#[derive(PartialEq, Eq)]
pub struct CosetToCanon {
    source: Arc<CosetZModule<CanonZModule>>,
    target: Arc<CanonZModule>,
    map: HashMap<Coset<CanonZModule>, <CanonZModule as ZModule>::Element>,
}

impl CosetToCanon {
    pub fn new(
        source: Arc<CosetZModule<CanonZModule>>,
        target: Arc<CanonZModule>,
        map: HashMap<Coset<CanonZModule>, <CanonZModule as ZModule>::Element>,
    ) -> Self {
        Self {
            source,
            target,
            map,
        }
    }

    fn evaluate(
        &self,
        element: &Coset<CanonZModule>,
    ) -> Result<<CanonZModule as ZModule>::Element, Error> {
        self.map.get(element).ok_or(Error::PartialMap).cloned()
    }
}

impl Morphism<CosetZModule<CanonZModule>, CanonZModule> for CosetToCanon {
    fn source(&self) -> Arc<CosetZModule<CanonZModule>> {
        Arc::clone(&self.source)
    }

    fn target(&self) -> Arc<CanonZModule> {
        Arc::clone(&self.target)
    }
}

/* # Canon to Coset */

pub struct CanonToCoset {
    source: Arc<CanonZModule>,
    target: Arc<CosetZModule<CanonZModule>>,
    map: HashMap<<CanonZModule as ZModule>::Element, Coset<CanonZModule>>,
}

impl CanonToCoset {
    pub fn new(
        source: Arc<CanonZModule>,
        target: Arc<CosetZModule<CanonZModule>>,
        map: HashMap<<CanonZModule as ZModule>::Element, Coset<CanonZModule>>,
    ) -> Self {
        Self {
            source,
            target,
            map,
        }
    }

    fn evaluate(
        &self,
        element: &<CanonZModule as ZModule>::Element,
    ) -> Result<Coset<CanonZModule>, Error> {
        self.map.get(element).ok_or(Error::PartialMap).cloned()
    }
}

impl Morphism<CanonZModule, CosetZModule<CanonZModule>> for CanonToCoset {
    fn source(&self) -> Arc<CanonZModule> {
        Arc::clone(&self.source)
    }

    fn target(&self) -> Arc<CosetZModule<CanonZModule>> {
        Arc::clone(&self.target)
    }
}

/* # Canon to Coset to Canon */

impl Compose<CanonZModule, CosetZModule<CanonZModule>, CanonZModule, CosetToCanon>
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
