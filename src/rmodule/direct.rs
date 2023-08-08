use crate::{
    category::morphism::{AbelianMorphism, Compose, Morphism, PreAbelianMorphism},
    matrix::Matrix,
    rmodule::{canon::CanonModule, map::CanonToCanon, ring::SuperRing, torsion::Coeff, Module},
};
use std::sync::Arc;

enum Side {
    Left,
    Right,
}

pub struct DirectModule<R: SuperRing> {
    left_inclusion: CanonToCanon<R>,
    right_inclusion: CanonToCanon<R>,
    left_projection: CanonToCanon<R>,
    right_projection: CanonToCanon<R>,
}

impl<R: SuperRing> DirectModule<R> {
    fn left(&self) -> Arc<CanonModule<R>> {
        // should be the same as left_projection.target()
        Arc::clone(&self.left_inclusion.source())
    }

    fn right(&self) -> Arc<CanonModule<R>> {
        // should be the same as right_projection.target()
        Arc::clone(&self.right_inclusion.source())
    }

    fn module(&self) -> Arc<CanonModule<R>> {
        // should be the same as right_inclusion.source()
        // should be the same as left_projection.target()
        // should be the same as right_projection.target()
        Arc::clone(&self.left_inclusion.source())
    }

    fn locate_key(&self, key: &Coeff<R>) -> Side {
        match self.left_inclusion.source().coeff_tree().contains_key(key) {
            true => Side::Left,
            false => Side::Right,
        }
    }

    pub fn submodules_goursat(&self) -> Vec<CanonToCanon<R>> {
        Arc::unwrap_or_clone(self.left())
            .submodules()
            .into_iter()
            .zip(Arc::unwrap_or_clone(self.right()).submodules())
            .flat_map(|(left_sub, right_sub)| {
                let mut phi_epis = Arc::unwrap_or_clone(self.left()).quotients();
                // this unfortunately is rather necessary
                let smol = DirectModule::biproduct(left_sub.source(), right_sub.source());
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

    pub fn quotients_goursat(&self) -> Vec<CanonToCanon<R>> {
        Arc::unwrap_or_clone(self.left())
            .quotients()
            .into_iter()
            .zip(Arc::unwrap_or_clone(self.right()).quotients())
            .flat_map(|(left_quot, right_quot)| {
                let mut phi_monos = Arc::unwrap_or_clone(self.right()).submodules();
                let smol = DirectModule::biproduct(left_quot.target(), right_quot.target());
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

    fn biproduct(left: Arc<CanonModule<R>>, right: Arc<CanonModule<R>>) -> Self {
        let mut coeff_tree = left.coeff_tree().clone();
        coeff_tree.join(right.coeff_tree().clone());
        let direct = Arc::new(CanonModule::new(coeff_tree));
        let left_dim: u8 = left
            .coeff_tree()
            .len()
            .try_into()
            .expect("we're gonna need a bigger int");
        let right_dim: u8 = right
            .coeff_tree()
            .len()
            .try_into()
            .expect("we're gonna need a bigger int");
        Self {
            left_inclusion: CanonToCanon::new_unchecked(
                Arc::clone(&left),
                Arc::clone(&direct),
                Matrix::from_cols(
                    left.coeff_tree()
                        .keys()
                        .map(|key| direct.versor(key).into_values().collect()),
                    left_dim,
                ),
            ),
            right_inclusion: CanonToCanon::new_unchecked(
                Arc::clone(&right),
                Arc::clone(&direct),
                Matrix::from_cols(
                    right
                        .coeff_tree()
                        .keys()
                        .map(|key| direct.versor(key).into_values().collect()),
                    right_dim,
                ),
            ),
            left_projection: CanonToCanon::new_unchecked(
                Arc::clone(&direct),
                Arc::clone(&left),
                Matrix::from_cols(
                    direct.coeff_tree().keys().map(|key| {
                        match left.coeff_tree().contains_key(key) {
                            true => left.versor(key).into_values().collect(),
                            false => <CanonModule<R> as Module<R>>::zero(&left)
                                .into_values()
                                .collect(),
                        }
                    }),
                    left_dim,
                ),
            ),
            right_projection: CanonToCanon::new_unchecked(
                Arc::clone(&direct),
                Arc::clone(&right),
                Matrix::from_cols(
                    direct.coeff_tree().keys().map(|key| {
                        match right.coeff_tree().contains_key(key) {
                            true => right.versor(key).into_values().collect(),
                            false => <CanonModule<R> as Module<R>>::zero(&right)
                                .into_values()
                                .collect(),
                        }
                    }),
                    right_dim,
                ),
            ),
        }
    }

    /**
    given two functions with the same source
    and whose target is self,
    construct the universal morphism from self to their source
    */
    fn universal_in(
        &self,
        left_par: CanonToCanon<R>,
        right_par: CanonToCanon<R>,
    ) -> CanonToCanon<R> {
        let mut cols = Vec::new();
        let mut cols_left = left_par.cols();
        let mut cols_right = right_par.cols();
        for key in self.module().coeff_tree().keys() {
            match self.locate_key(key) {
                Side::Left => cols.push(cols_left.next().expect("the number of keys should match")),
                Side::Right => {
                    cols.push(cols_right.next().expect("the number of keys should match"))
                }
            }
        }
        CanonToCanon::new_unchecked(
            left_par.source(),
            self.module(),
            Matrix::from_cols(
                cols,
                self.module()
                    .dimension()
                    .try_into()
                    .expect("we're gonna need a bigger int"),
            ),
        )
    }

    /**
    given two functions with the same target
    and whose source is self,
    construct the universal morphism from self to their target
    */
    fn universal_out(
        &self,
        left_par: CanonToCanon<R>,
        right_par: CanonToCanon<R>,
    ) -> CanonToCanon<R> {
        let mut rows = Vec::new();
        let mut rows_left = left_par.rows();
        let mut rows_right = right_par.rows();
        for key in self.module().coeff_tree().keys() {
            match self.locate_key(key) {
                Side::Left => rows.push(rows_left.next().expect("the number of keys should match")),
                Side::Right => {
                    rows.push(rows_right.next().expect("the number of keys should match"))
                }
            }
        }
        CanonToCanon::new_unchecked(
            self.module(),
            left_par.target(),
            Matrix::from_rows(
                rows,
                self.module()
                    .dimension()
                    .try_into()
                    .expect("we're gonna need a bigger int"),
            ),
        )
    }
}

impl<R: SuperRing> From<CanonModule<R>> for DirectModule<R> {
    fn from(canon: CanonModule<R>) -> Self {
        let canon_arc = Arc::new(canon);
        let (left_coeff, right_coeff) = canon_arc.coeff_tree().clone().split();
        let left_dim: u8 = left_coeff
            .len()
            .try_into()
            .expect("we're gonna need a bigger int");
        let right_dim: u8 = right_coeff
            .len()
            .try_into()
            .expect("we're gonna need a bigger int");
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
                    left_dim,
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
                    right_dim,
                ),
            ),
            left_projection: CanonToCanon::new_unchecked(
                Arc::clone(&canon_arc),
                Arc::clone(&left),
                Matrix::from_cols(
                    canon_arc.coeff_tree().keys().map(|key| {
                        match left.coeff_tree().contains_key(key) {
                            true => left.versor(key).into_values().collect(),
                            false => <CanonModule<R> as Module<R>>::zero(&left)
                                .into_values()
                                .collect(),
                        }
                    }),
                    left_dim,
                ),
            ),
            right_projection: CanonToCanon::new_unchecked(
                Arc::clone(&canon_arc),
                Arc::clone(&right),
                Matrix::from_cols(
                    canon_arc.coeff_tree().keys().map(|key| {
                        match right.coeff_tree().contains_key(key) {
                            true => right.versor(key).into_values().collect(),
                            false => <CanonModule<R> as Module<R>>::zero(&right)
                                .into_values()
                                .collect(),
                        }
                    }),
                    right_dim,
                ),
            ),
        }
    }
}
