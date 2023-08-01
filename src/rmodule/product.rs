#![allow(unused_imports)] // DELETE LATER
use crate::{
    category::morphism::{AbelianMorphism, Compose, Morphism},
    matrix::Matrix,
    rmodule::{
        canon::CanonModule,
        map::CanonToCanon,
        ring::{Radix, Ring, SuperRing},
        Module,
    },
    util::number::versor,
};
use std::sync::Arc;

pub struct BiProductZModule<RC: Radix, R: SuperRing<RC>> {
    left_inclusion: CanonToCanon<RC, R>,
    right_inclusion: CanonToCanon<RC, R>,
    left_projection: CanonToCanon<RC, R>,
    right_projection: CanonToCanon<RC, R>,
}

impl<RC: Radix, R: SuperRing<RC>> BiProductZModule<RC, R> {
    pub fn left(&self) -> Arc<CanonModule<RC, R>> {
        // should be the same as left_projection.target()
        Arc::clone(&self.left_inclusion.source())
    }

    pub fn right(&self) -> Arc<CanonModule<RC, R>> {
        // should be the same as right_projection.target()
        Arc::clone(&self.right_inclusion.source())
    }

    pub fn submodules_goursat(&self) -> Vec<CanonToCanon<RC, R>> {
        self.left()
            .submodules()
            .iter()
            .zip(self.right().submodules())
            .flat_map(|(left_sub, right_sub)| {
                right_sub
                    .source()
                    .submodules()
                    .into_iter()
                    .map(|right_sub_sub| {
                        let right_quot = right_sub_sub.cokernel();
                        CanonToCanon::epis(left_sub.source(), right_quot.target())
                            .into_iter()
                            .map(|projection| {
                                let equa = equailiser(
                                    right_quot.compose(&smol.left_projection),
                                    projection.compose(&smol.right_projection),
                                );
                                equa.compose(subs_into_self)
                            })
                    })
            })
            .collect()
    }
}

impl<RC: Radix, R: SuperRing<RC>> From<CanonModule<RC, R>> for BiProductZModule<RC, R> {
    fn from(canon: CanonModule<RC, R>) -> Self {
        let canon_arc = Arc::new(canon);
        let (left_coeff, right_coeff) = canon_arc.coeff_tree().clone().split();
        let left_dim = left_coeff.len();
        let right_dim = right_coeff.len();
        let left = Arc::new(CanonModule::new(left_coeff));
        let right = Arc::new(CanonModule::new(right_coeff));
        Self {
            left_inclusion: CanonToCanon::new_unchecked(
                Arc::clone(&left),
                Arc::clone(&canon_arc),
                Matrix::from_cols(
                    left.coeff_tree()
                        .keys()
                        .map(|key| canon_arc.versor(key).into_values().collect()),
                    left_dim.try_into().expect("we're gonna need a bigger int"),
                ),
            ),
            right_inclusion: CanonToCanon::new_unchecked(
                Arc::clone(&right),
                Arc::clone(&canon_arc),
                Matrix::from_cols(
                    right
                        .coeff_tree()
                        .keys()
                        .map(|key| canon_arc.versor(key).into_values().collect()),
                    right_dim.try_into().expect("we're gonna need a bigger int"),
                ),
            ),
            left_projection: CanonToCanon::new_unchecked(
                Arc::clone(&canon_arc),
                Arc::clone(&left),
                Matrix::from_cols(
                    canon_arc.coeff_tree().keys().map(|key| {
                        match left.coeff_tree().contains_key(key) {
                            true => left.versor(key).into_values().collect(),
                            false => <CanonModule<RC, R> as Module<RC, R>>::zero(&left)
                                .into_values()
                                .collect(),
                        }
                    }),
                    left_dim.try_into().expect("we're gonna need a bigger int"),
                ),
            ),
            right_projection: CanonToCanon::new_unchecked(
                Arc::clone(&canon_arc),
                Arc::clone(&right),
                Matrix::from_cols(
                    canon_arc.coeff_tree().keys().map(|key| {
                        match right.coeff_tree().contains_key(key) {
                            true => right.versor(key).into_values().collect(),
                            false => <CanonModule<RC, R> as Module<RC, R>>::zero(&right)
                                .into_values()
                                .collect(),
                        }
                    }),
                    right_dim.try_into().expect("we're gonna need a bigger int"),
                ),
            ),
        }
    }
}
