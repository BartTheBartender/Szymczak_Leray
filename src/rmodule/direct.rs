use crate::{
    category::{
        morphism::{AbelianMorphism, Compose, Morphism, PreAbelianMorphism},
        Duplicate,
    },
    rmodule::{canon::CanonModule, map::CanonToCanon, ring::SuperRing, Module},
    util::matrix::Matrix,
};
use itertools::iproduct;
use std::{collections::BTreeSet, sync::Arc};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct DirectModule<R: SuperRing> {
    pub left_inclusion: CanonToCanon<R>,
    pub right_inclusion: CanonToCanon<R>,
    pub left_projection: CanonToCanon<R>,
    pub right_projection: CanonToCanon<R>,
}

impl<R: SuperRing> DirectModule<R> {
    pub fn left(&self) -> Arc<CanonModule<R>> {
        // should be the same as left_projection.target()
        Arc::clone(&self.left_inclusion.source())
    }

    pub fn right(&self) -> Arc<CanonModule<R>> {
        // should be the same as right_projection.target()
        Arc::clone(&self.right_inclusion.source())
    }

    pub fn module(&self) -> Arc<CanonModule<R>> {
        // should be the same as right_inclusion.target()
        // should be the same as left_projection.source()
        // should be the same as right_projection.source()
        Arc::clone(&self.left_inclusion.target())
    }

    pub fn submodules_goursat(&self) -> impl Iterator<Item = CanonToCanon<R>> + '_ {
        iproduct!(
            Arc::unwrap_or_clone(self.left()).submodules(),
            Arc::unwrap_or_clone(self.right()).submodules()
        )
        .flat_map(|(left_sub, right_sub)| {
            let smol = Self::sumproduct(&left_sub.source(), &right_sub.source());
            Arc::unwrap_or_clone(right_sub.source())
                .submodules()
                .into_iter()
                .map(|sub| sub.cokernel())
                .flat_map(|right_quot| {
                    CanonToCanon::hom(smol.left(), right_quot.target())
                        .filter(|map| map.cokernel().is_zero())
                        .map(|phi| {
                            smol.left_projection
                                .compose_unchecked(&phi)
                                .equaliser(smol.right_projection.compose_unchecked(&right_quot))
                                .compose_unchecked(&smol.universal_out(
                                    &left_sub.compose_unchecked(&self.left_inclusion),
                                    &right_sub.compose_unchecked(&self.right_inclusion),
                                ))
                        })
                        .collect::<Vec<_>>() // necessary to force the closure to release borrowed variables
                })
                .collect::<Vec<_>>() // necessary to force the closure to release borrowed variables
        })
    }

    pub fn quotients_goursat(&self) -> impl Iterator<Item = CanonToCanon<R>> + '_ {
        iproduct!(
            Arc::unwrap_or_clone(self.left()).quotients(),
            Arc::unwrap_or_clone(self.right()).quotients()
        )
        .flat_map(|(left_quot, right_quot)| {
            let smol = Self::sumproduct(&left_quot.target(), &right_quot.target());
            Arc::unwrap_or_clone(left_quot.target())
                .quotients()
                .into_iter()
                .map(|quot| quot.kernel())
                .flat_map(|left_sub| {
                    CanonToCanon::hom(smol.right(), left_sub.source())
                        .filter(|map| map.kernel().is_zero())
                        .map(|phi| {
                            smol.universal_in(
                                &self.left_projection.compose_unchecked(&left_quot),
                                &self.right_projection.compose_unchecked(&right_quot),
                            )
                            .compose_unchecked(
                                &phi.compose_unchecked(&smol.right_inclusion)
                                    .coequaliser(left_sub.compose_unchecked(&smol.left_inclusion)),
                            )
                        })
                        .collect::<Vec<_>>() // necessary to force the closure to release borrowed variables
                })
                .collect::<Vec<_>>() // necessary to force the closure to release borrowed variables
        })
    }

    pub fn sumproduct(left: &Arc<CanonModule<R>>, right: &Arc<CanonModule<R>>) -> Self {
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
            left_inclusion: CanonToCanon::new(
                Arc::clone(left),
                Arc::clone(&direct),
                Matrix::from_cols(
                    left.coeff_tree()
                        .keys()
                        .map(|key| direct.versor(key).into_values().collect()),
                    left_dim,
                ),
            ),
            right_inclusion: CanonToCanon::new(
                Arc::clone(right),
                Arc::clone(&direct),
                Matrix::from_cols(
                    right
                        .coeff_tree()
                        .keys()
                        .map(|key| direct.versor(key).into_values().collect()),
                    right_dim,
                ),
            ),
            left_projection: CanonToCanon::new(
                Arc::clone(&direct),
                Arc::clone(left),
                Matrix::from_rows(
                    direct.coeff_tree().keys().map(|key| {
                        match left.coeff_tree().contains_key(key) {
                            true => left.versor(key).into_values().collect(),
                            false => <CanonModule<R> as Module<R>>::zero(left)
                                .into_values()
                                .collect(),
                        }
                    }),
                    left_dim,
                ),
            ),
            right_projection: CanonToCanon::new(
                Arc::clone(&direct),
                Arc::clone(right),
                Matrix::from_rows(
                    direct.coeff_tree().keys().map(|key| {
                        match right.coeff_tree().contains_key(key) {
                            true => right.versor(key).into_values().collect(),
                            false => <CanonModule<R> as Module<R>>::zero(right)
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
        left_par: &CanonToCanon<R>,
        right_par: &CanonToCanon<R>,
    ) -> CanonToCanon<R> {
        let mut rows = Vec::new();
        let mut rows_left = left_par.rows();
        let mut rows_right = right_par.rows();
        let mut coeffs_left = self.left().torsion_coeffs().collect::<BTreeSet<_>>();
        let mut coeffs_right = self.right().torsion_coeffs().collect::<BTreeSet<_>>();
        for coeff in self.module().torsion_coeffs() {
            if coeffs_left.remove(&coeff) {
                rows.push(rows_left.next().expect("the number of keys should match"));
            } else if coeffs_right.remove(&coeff) {
                rows.push(rows_right.next().expect("the number of keys should match"));
            }
        }
        CanonToCanon::new(
            left_par.source(),
            self.module(),
            Matrix::from_rows(
                rows,
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
        left_par: &CanonToCanon<R>,
        right_par: &CanonToCanon<R>,
    ) -> CanonToCanon<R> {
        let mut cols = Vec::new();
        let mut cols_left = left_par.cols();
        let mut cols_right = right_par.cols();
        let mut coeffs_left = self.left().torsion_coeffs().collect::<BTreeSet<_>>();
        let mut coeffs_right = self.right().torsion_coeffs().collect::<BTreeSet<_>>();
        for coeff in self.module().torsion_coeffs() {
            if coeffs_left.remove(&coeff) {
                cols.push(cols_left.next().expect("the number of keys should match"));
            } else if coeffs_right.remove(&coeff) {
                cols.push(cols_right.next().expect("the number of keys should match"));
            }
        }
        CanonToCanon::new(
            self.module(),
            left_par.target(),
            Matrix::from_cols(
                cols,
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
        let (left_coeff, right_coeff) = canon.into_coeff_tree().split();
        Self::sumproduct(
            &Arc::new(CanonModule::new(left_coeff)),
            &Arc::new(CanonModule::new(right_coeff)),
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::rmodule::{
        ring::{Fin, Set},
        torsion::CoeffTree,
    };
    use typenum::{U12, U4};

    #[test]
    fn universal_morphism_in_easy() {
        type R = Fin<U12>;
        let z2 = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(2)])));
        let z3 = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(3)])));
        let z4 = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(4)])));
        let z4xz3_canon = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([
            R::new(4),
            R::new(3),
        ])));
        let z4xz3_direct = DirectModule::sumproduct(&z4, &z3);
        assert_eq!(
            z4xz3_direct.universal_in(
                &CanonToCanon::new(
                    Arc::clone(&z2),
                    Arc::clone(&z4),
                    Matrix::from_buffer([R::new(2)], 1, 2),
                ),
                &CanonToCanon::new(
                    Arc::clone(&z2),
                    Arc::clone(&z3),
                    Matrix::from_buffer([R::new(0)], 1, 1),
                ),
            ),
            CanonToCanon::new(
                Arc::clone(&z2),
                Arc::clone(&z4xz3_canon),
                Matrix::from_buffer([R::new(2), R::new(0)], 1, 2),
            )
        );
    }

    #[test]
    fn universal_morphism_in_medium() {
        type R = Fin<U12>;
        let z4xz3 = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([
            R::new(4),
            R::new(3),
        ])));
        let z3xz2 = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([
            R::new(3),
            R::new(2),
        ])));
        let z4xz3sqxz2_canon = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([
            R::new(4),
            R::new(3),
            R::new(3),
            R::new(2),
        ])));
        let z4xz3sqxz2_direct = DirectModule::sumproduct(&z3xz2, &z4xz3);
        let univ_in = z4xz3sqxz2_direct.universal_in(
            &CanonToCanon::new(
                Arc::clone(&z4xz3),
                Arc::clone(&z3xz2),
                Matrix::from_buffer([R::new(0), R::new(2), R::new(1), R::new(0)], 2, 2),
            ),
            &CanonToCanon::new(
                Arc::clone(&z4xz3),
                Arc::clone(&z4xz3),
                Matrix::from_buffer([R::new(2), R::new(0), R::new(0), R::new(1)], 2, 2),
            ),
        );
        let true_output_a = CanonToCanon::new(
            Arc::clone(&z4xz3),
            Arc::clone(&z4xz3sqxz2_canon),
            Matrix::from_buffer(
                [
                    R::new(2),
                    R::new(0),
                    R::new(0),
                    R::new(1),
                    R::new(0),
                    R::new(2),
                    R::new(1),
                    R::new(0),
                ],
                2,
                4,
            ),
        );
        let true_output_b = CanonToCanon::new(
            Arc::clone(&z4xz3),
            Arc::clone(&z4xz3sqxz2_canon),
            Matrix::from_buffer(
                [
                    R::new(2),
                    R::new(0),
                    R::new(0),
                    R::new(2),
                    R::new(0),
                    R::new(1),
                    R::new(1),
                    R::new(0),
                ],
                2,
                4,
            ),
        );
        // due to random id, one of those will be true
        assert!(univ_in == true_output_a || univ_in == true_output_b,);
    }

    #[test]
    fn universal_morphism_out_easy() {
        type R = Fin<U12>;
        let z2 = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(2)])));
        let z3 = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(3)])));
        let z4 = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(4)])));
        let z4xz3_canon = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([
            R::new(4),
            R::new(3),
        ])));
        let z4xz3_direct = DirectModule::sumproduct(&z4, &z3);
        assert_eq!(
            z4xz3_direct.universal_out(
                &CanonToCanon::new(
                    Arc::clone(&z4),
                    Arc::clone(&z2),
                    Matrix::from_buffer([R::new(1)], 1, 2),
                ),
                &CanonToCanon::new(
                    Arc::clone(&z3),
                    Arc::clone(&z2),
                    Matrix::from_buffer([R::new(0)], 1, 1),
                ),
            ),
            CanonToCanon::new(
                Arc::clone(&z4xz3_canon),
                Arc::clone(&z2),
                Matrix::from_buffer([R::new(1), R::new(0)], 2, 1),
            )
        );
    }

    #[test]
    fn universal_morphism_out_medium() {
        type R = Fin<U12>;
        let z4xz3 = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([
            R::new(4),
            R::new(3),
        ])));
        let z3xz2 = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([
            R::new(3),
            R::new(2),
        ])));
        let z4xz3sqxz2_canon = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([
            R::new(4),
            R::new(3),
            R::new(3),
            R::new(2),
        ])));
        let z4xz3sqxz2_direct = DirectModule::sumproduct(&z3xz2, &z4xz3);
        let univ_out = z4xz3sqxz2_direct.universal_out(
            &CanonToCanon::new(
                Arc::clone(&z3xz2),
                Arc::clone(&z4xz3),
                Matrix::from_buffer([R::new(0), R::new(2), R::new(2), R::new(0)], 2, 2),
            ),
            &CanonToCanon::new(
                Arc::clone(&z4xz3),
                Arc::clone(&z4xz3),
                Matrix::from_buffer([R::new(3), R::new(0), R::new(0), R::new(1)], 2, 2),
            ),
        );
        let true_output_a = CanonToCanon::new(
            Arc::clone(&z4xz3sqxz2_canon),
            Arc::clone(&z4xz3),
            Matrix::from_buffer(
                [
                    R::new(3),
                    R::new(0),
                    R::new(0),
                    R::new(2),
                    R::new(0),
                    R::new(2),
                    R::new(1),
                    R::new(0),
                ],
                4,
                2,
            ),
        );
        let true_output_b = CanonToCanon::new(
            Arc::clone(&z4xz3sqxz2_canon),
            Arc::clone(&z4xz3),
            Matrix::from_buffer(
                [
                    R::new(3),
                    R::new(0),
                    R::new(0),
                    R::new(2),
                    R::new(0),
                    R::new(1),
                    R::new(2),
                    R::new(0),
                ],
                4,
                2,
            ),
        );
        // due to random id, one of those will be true
        assert!(univ_out == true_output_a || univ_out == true_output_b,);
    }

    #[test]
    #[allow(non_snake_case)] // module names look this way
    fn sumproduct_of_Z2_and_Z4() {
        type R = Fin<U4>;
        let z2 = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(2)])));
        let z4 = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(4)])));
        let z4xz2_canon = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([
            R::new(2),
            R::new(4),
        ])));
        let z4xz2_direct = DirectModule::sumproduct(&z2, &z4);

        assert_eq!(z4xz2_direct.module(), z4xz2_canon);
        assert_eq!(
            z4xz2_direct.left_inclusion,
            CanonToCanon::new(
                Arc::clone(&z2),
                Arc::clone(&z4xz2_canon),
                Matrix::from_buffer([R::new(0), R::new(1)], 1, 2),
            )
        );
        assert_eq!(
            z4xz2_direct.right_inclusion,
            CanonToCanon::new(
                Arc::clone(&z4),
                Arc::clone(&z4xz2_canon),
                Matrix::from_buffer([R::new(1), R::new(0)], 1, 2),
            )
        );
        assert_eq!(
            z4xz2_direct.left_projection,
            CanonToCanon::new(
                Arc::clone(&z4xz2_canon),
                z2,
                Matrix::from_buffer([R::new(0), R::new(1)], 2, 1),
            )
        );
        assert_eq!(
            z4xz2_direct.right_projection,
            CanonToCanon::new(
                z4xz2_canon,
                z4,
                Matrix::from_buffer([R::new(1), R::new(0)], 2, 1),
            )
        );
    }

    #[test]
    fn sumproduct_and_duplicate() {
        use typenum::U3 as N;
        type R = Fin<N>;

        let zn_module: Arc<CanonModule<R>> = Arc::new(
            CoeffTree::<R, ()>::all_torsion_coeffs(1)
                .map(CanonModule::new)
                .next()
                .unwrap(),
        );

        let zn_zn = DirectModule::<R>::sumproduct(
            &Arc::clone(&zn_module),
            &Arc::new(zn_module.duplicate()),
        )
        .module();

        assert_eq!(zn_zn.to_string(), "Z3 x Z3");
    }

    #[test]
    #[should_panic]
    fn z7_issue() {
        use typenum::{Unsigned, U7 as N};
        let n = N::to_usize();
        type R = Fin<N>;

        let zn = CoeffTree::<R, ()>::all_torsion_coeffs(1)
            .map(|tc| CanonModule::<R>::new(tc))
            .find(|module| module.cardinality() == n)
            .expect("there is a zn module here");

        let direct =
            DirectModule::<R>::sumproduct(&Arc::new(zn.clone()), &Arc::new(zn.duplicate()));

        assert!(direct
            .submodules_goursat()
            .find(|canon_to_canon| canon_to_canon.to_string()
                == "Mtx(1x2)[Z7(1), Z7(1)] : Z7 -> Z7xZ7")
            .is_some());
        assert!(direct
            .submodules_goursat()
            .find(|canon_to_canon| canon_to_canon.to_string()
                == "Mtx(1x2)[Z7(4), Z7(4)] : Z7 -> Z7xZ7")
            .is_some());
    }
}
