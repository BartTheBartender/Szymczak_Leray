#![allow(unused_imports)] // DELETE LATER
use crate::{
    category::morphism::{AbelianMorphism, Compose, Morphism, PreAbelianMorphism},
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

pub struct DirectModule<RC: Radix, R: SuperRing<RC>> {
    // left: Arc<CanonModule<RC, R>>,
    // right: Arc<CanonModule<RC, R>>,
    left_inclusion: CanonToCanon<RC, R>,
    right_inclusion: CanonToCanon<RC, R>,
    left_projection: CanonToCanon<RC, R>,
    right_projection: CanonToCanon<RC, R>,
}

impl<RC: Radix, R: SuperRing<RC>> DirectModule<RC, R> {
    pub fn left(&self) -> Arc<CanonModule<RC, R>> {
        // should be the same as left_projection.target()
        Arc::clone(&self.left_inclusion.source())
    }

    pub fn right(&self) -> Arc<CanonModule<RC, R>> {
        // should be the same as right_projection.target()
        Arc::clone(&self.right_inclusion.source())
    }

    pub fn submodules_goursat(&self) -> Vec<CanonToCanon<RC, R>> {
        Arc::unwrap_or_clone(self.left())
            .submodules()
            .into_iter()
            .zip(Arc::unwrap_or_clone(self.right()).submodules())
            .flat_map(|(left_sub, right_sub)| {
                let mut phi_epis = Arc::unwrap_or_clone(self.left()).quotients();
                // this unfortunately is rather necessary
                let smol = DirectModule::biproduct(&left_sub.source(), &right_sub.source());
                Arc::unwrap_or_clone(right_sub.source())
                    .submodules()
                    .into_iter()
                    .map(|sub| sub.cokernel())
                    .flat_map(|right_quot| {
                        phi_epis
                            .extract_if(|phi| phi.target() == right_quot.target())
                            .map(|phi| {
                                let equa = smol.left_projection.compose_unchecked(&phi).equaliser(
                                    smol.right_projection.compose_unchecked(&right_quot),
                                );
                                equa.compose_unchecked(&smol.universal_out(
                                    left_sub.compose_unchecked(&self.left_inclusion),
                                    right_sub.compose_unchecked(&self.right_inclusion),
                                ))
                            })
                            .collect::<Vec<_>>() // necessary to force the closure to release borrowed variables
                    })
                    .collect::<Vec<_>>() // necessary to force the closure to release borrowed variables
            })
            .collect()
    }

    pub fn quotients_goursat(&self) -> Vec<CanonToCanon<RC, R>> {
        Arc::unwrap_or_clone(self.left())
            .quotients()
            .into_iter()
            .zip(Arc::unwrap_or_clone(self.right()).quotients())
            .flat_map(|(left_quot, right_quot)| {
                let mut phi_monos = Arc::unwrap_or_clone(self.right()).submodules();
                let smol = DirectModule::biproduct(&left_quot.target(), &right_quot.target());
                Arc::unwrap_or_clone(left_quot.target())
                    .quotients()
                    .into_iter()
                    .map(|quot| quot.kernel())
                    .flat_map(|left_sub| {
                        phi_monos
                            .extract_if(|phi| phi.source() == left_sub.source())
                            .map(|phi| {
                                let coequa = phi
                                    .compose_unchecked(&smol.right_inclusion)
                                    .coequaliser(left_sub.compose_unchecked(&smol.left_inclusion));
                                smol.universal_in(
                                    self.left_projection.compose_unchecked(&left_quot),
                                    self.right_projection.compose_unchecked(&right_quot),
                                )
                                .compose_unchecked(&coequa)
                            })
                            .collect::<Vec<_>>() // necessary to force the closure to release borrowed variables
                    })
                    .collect::<Vec<_>>() // necessary to force the closure to release borrowed variables
            })
            .collect()
    }

    pub fn biproduct(left: &CanonModule<RC, R>, right: &CanonModule<RC, R>) -> Self {
        let mut coeff_tree = left.coeff_tree().clone();
        coeff_tree.join(right.coeff_tree().clone());
        todo!() // this is more difficult than i previously envisioned
                // since the join may combine some coefficients
                // so the inclusions and projections may be whack
    }

    /**
    given two functions with the same source
    and whose target is self,
    construct the universal morphism from self to their source
    */
    pub fn universal_in(
        &self,
        _left_par: CanonToCanon<RC, R>,
        _right_par: CanonToCanon<RC, R>,
    ) -> CanonToCanon<RC, R> {
        todo!()
    }

    /**
    given two functions with the same target
    and whose source is self,
    construct the universal morphism from self to their target
    */
    pub fn universal_out(
        &self,
        _left_par: CanonToCanon<RC, R>,
        _right_par: CanonToCanon<RC, R>,
    ) -> CanonToCanon<RC, R> {
        todo!()
    }
}

impl<RC: Radix, R: SuperRing<RC>> From<CanonModule<RC, R>> for DirectModule<RC, R> {
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
