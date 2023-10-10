use crate::{
    category::morphism::{Abelian, Enumerable, Morphism, PreAbelian},
    ralg::{
        cgroup::{ideal::CIdeal, Radix, C},
        matrix::Matrix,
        module::{canon::object::Object as CanonModule, map::CanonToCanon},
        ring::{
            ideal::{Ideal, Principal as PrincipalIdeal},
            AdditivePartialMonoid, Factorial as FactorialRing, Ring,
        },
    },
};
use itertools::iproduct;
use rayon::prelude::*;
use std::sync::Arc;
use typenum::{IsGreater, U1};

#[derive(Clone)]
pub struct Object<R: Ring, I: Ideal<Parent = R> + Ord> {
    pub left_inclusion: CanonToCanon<R, I>,
    pub right_inclusion: CanonToCanon<R, I>,
    pub left_projection: CanonToCanon<R, I>,
    pub right_projection: CanonToCanon<R, I>,
}

/* # debug and display */

/* # conversions */

impl<R: FactorialRing, I: PrincipalIdeal<Parent = R> + Ord> From<CanonModule<R, I>>
    for Object<R, I>
{
    fn from(canon: CanonModule<R, I>) -> Self {
        let (left, right) = canon.split();
        Self::sumproduct(&Arc::new(left), &Arc::new(right))
    }
}

/* # funcionality */

impl<R: Ring + Copy, I: Ideal<Parent = R> + Ord> Object<R, I> {
    pub fn left(&self) -> Arc<CanonModule<R, I>> {
        // should be the same as left_projection.target()
        Arc::clone(&self.left_inclusion.source())
    }

    pub fn right(&self) -> Arc<CanonModule<R, I>> {
        // should be the same as right_projection.target()
        Arc::clone(&self.right_inclusion.source())
    }

    pub fn module(&self) -> Arc<CanonModule<R, I>> {
        // should be the same as right_inclusion.target()
        // should be the same as left_projection.source()
        // should be the same as right_projection.source()
        Arc::clone(&self.left_inclusion.target())
    }

    /**
    given two functions with the same source
    and whose target is self,
    construct the universal morphism from self to their source
    */
    #[allow(clippy::expect_used, reason = "structural properties")]
    fn universal_in(
        &self,
        left_par: &CanonToCanon<R, I>,
        right_par: &CanonToCanon<R, I>,
    ) -> CanonToCanon<R, I> {
        let mut rows = Vec::new();
        let mut rows_left = left_par.rows();
        let mut rows_right = right_par.rows();
        let mut coeffs_left = Arc::unwrap_or_clone(self.left());
        let mut coeffs_right = Arc::unwrap_or_clone(self.right());
        for mark in self.module().iter() {
            if coeffs_left.remove(mark) {
                rows.push(
                    rows_left
                        .next()
                        .expect("the number of keys should match")
                        .copied()
                        .collect(),
                );
            } else if coeffs_right.remove(mark) {
                rows.push(
                    rows_right
                        .next()
                        .expect("the number of keys should match")
                        .copied()
                        .collect(),
                );
            } else {
                // either of the above will be true
            }
        }
        CanonToCanon::new(
            &left_par.source(),
            &self.module(),
            Matrix::from_rows_custom(
                rows,
                left_par.source().dimension(),
                self.module().dimension(),
            ),
        )
    }

    /**
    given two functions with the same target
    and whose source is self,
    construct the universal morphism from self to their target
    */
    #[allow(clippy::expect_used, reason = "structural properties")]
    fn universal_out(
        &self,
        left_par: &CanonToCanon<R, I>,
        right_par: &CanonToCanon<R, I>,
    ) -> CanonToCanon<R, I> {
        let mut cols = Vec::new();
        let mut cols_left = left_par.cols();
        let mut cols_right = right_par.cols();
        let mut coeffs_left = Arc::unwrap_or_clone(self.left());
        let mut coeffs_right = Arc::unwrap_or_clone(self.right());
        for mark in self.module().iter() {
            if coeffs_left.remove(mark) {
                cols.push(
                    cols_left
                        .next()
                        .expect("the number of keys should match")
                        .copied()
                        .collect(),
                );
            } else if coeffs_right.remove(mark) {
                cols.push(
                    cols_right
                        .next()
                        .expect("the number of keys should match")
                        .copied()
                        .collect(),
                );
            } else {
                // either of the above will be true
            }
        }
        CanonToCanon::new(
            &self.module(),
            &left_par.target(),
            Matrix::from_cols_custom(
                cols,
                self.module().dimension(),
                left_par.target().dimension(),
            ),
        )
    }
}

impl<R: FactorialRing, I: PrincipalIdeal<Parent = R> + Ord> Object<R, I> {
    pub fn sumproduct(left: &Arc<CanonModule<R, I>>, right: &Arc<CanonModule<R, I>>) -> Self {
        let module = Arc::new(CanonModule::join((**left).clone(), (**right).clone()));
        let left_dim: usize = left.dimension();
        let right_dim: usize = right.dimension();
        let module_dim: usize = module.dimension();
        Self {
            left_inclusion: CanonToCanon::new(
                left,
                &module,
                Matrix::from_cols_custom(
                    left.iter()
                        .map(|mark| module.versor(mark).into_values().collect()),
                    left_dim,
                    module_dim,
                ),
            ),
            right_inclusion: CanonToCanon::new(
                right,
                &module,
                Matrix::from_cols_custom(
                    right
                        .iter()
                        .map(|mark| module.versor(mark).into_values().collect()),
                    right_dim,
                    module_dim,
                ),
            ),
            left_projection: CanonToCanon::new(
                &module,
                left,
                Matrix::from_cols_custom(
                    module.iter().map(|mark| match left.contains(mark) {
                        true => left.versor(mark).into_values().collect(),
                        false => left
                            .element_from_iterator((0..left_dim).map(|_| R::zero()))
                            .into_values()
                            .collect(),
                    }),
                    module_dim,
                    left_dim,
                ),
            ),
            right_projection: CanonToCanon::new(
                &module,
                right,
                Matrix::from_cols_custom(
                    module.iter().map(|mark| match right.contains(mark) {
                        true => right.versor(mark).into_values().collect(),
                        false => right
                            .element_from_iterator((0..right_dim).map(|_| R::zero()))
                            .into_values()
                            .collect(),
                    }),
                    module_dim,
                    right_dim,
                ),
            ),
        }
    }
}

impl<Period: Radix + IsGreater<U1> + Send + Sync> Object<C<Period>, CIdeal<Period>> {
    #[allow(
        clippy::expect_used,
        reason = "the compositions were tailor made by our skilled team of artisan mathematicians"
    )]
    // i wish this did not require so many collects,
    // but i am too dumb to do this right.
    // for now, let it be
    pub fn submodules_goursat(self) -> Vec<CanonToCanon<C<Period>, CIdeal<Period>>> {
        iproduct!(
            Arc::unwrap_or_clone(self.left()).submodules(),
            Arc::unwrap_or_clone(self.right()).submodules()
        )
        .par_bridge()
        .flat_map(|(left_sub, right_sub)| {
            let smol = Self::sumproduct(&left_sub.source(), &right_sub.source());
            Arc::unwrap_or_clone(right_sub.source())
                .submodules()
                .into_par_iter()
                .map(|sub| sub.cokernel())
                .flat_map(|right_quot| {
                    CanonToCanon::hom(smol.left(), right_quot.target())
                        .par_bridge()
                        .filter(|map| map.cokernel().is_zero())
                        .map(|phi| {
                            smol.left_projection
                                .try_compose(&phi)
                                .expect("phi after smol.left_projection")
                                .try_equaliser(
                                    smol.right_projection
                                        .try_compose(&right_quot)
                                        .expect("right_quot after smol.right_projection"),
                                )
                                .expect("equaliser")
                                .try_compose(
                                    &smol.universal_out(
                                        &left_sub
                                            .try_compose(&self.left_inclusion)
                                            .expect("self.left_inclusion after left_sub"),
                                        &right_sub
                                            .try_compose(&self.right_inclusion)
                                            .expect("self.right_inclusion after right_sub"),
                                    ),
                                )
                                .expect("universal_out after equaliser")
                        })
                        .collect::<Vec<_>>() // necessary to force the closure to release borrowed variables
                })
                .collect::<Vec<_>>() // necessary to force the closure to release borrowed variables
        })
        .collect::<Vec<_>>() // necessary to force the closure to release borrowed variables
    }

    #[allow(
        clippy::expect_used,
        reason = "the compositions were tailor made by our skilled team of artisan mathematicians"
    )]
    // i wish this did not require so many collects,
    // but i am too dumb to do this right.
    // for now, let it be
    pub fn quotients_goursat(&self) -> Vec<CanonToCanon<C<Period>, CIdeal<Period>>> {
        iproduct!(
            Arc::unwrap_or_clone(self.left()).quotients(),
            Arc::unwrap_or_clone(self.right()).quotients()
        )
        .par_bridge()
        .flat_map(|(left_quot, right_quot)| {
            let smol = Self::sumproduct(&left_quot.target(), &right_quot.target());
            Arc::unwrap_or_clone(left_quot.target())
                .quotients()
                .into_par_iter()
                .map(|quot| quot.kernel())
                .flat_map(|left_sub| {
                    CanonToCanon::hom(smol.right(), left_sub.source())
                        .par_bridge()
                        .filter(|map| map.kernel().is_zero())
                        .map(|phi| {
                            smol.universal_in(
                                &self
                                    .left_projection
                                    .try_compose(&left_quot)
                                    .expect("left_quot after self.left_projection"),
                                &self
                                    .right_projection
                                    .try_compose(&right_quot)
                                    .expect("right_quot after self.right_projection"),
                            )
                            .try_compose(
                                &phi.try_compose(&smol.right_inclusion)
                                    .expect("smol.right_inclusion after phi")
                                    .try_coequaliser(
                                        left_sub
                                            .try_compose(&smol.left_inclusion)
                                            .expect("smol.left_inclusion after left_sub"),
                                    )
                                    .expect("coequaliser"),
                            )
                            .expect("coequaliser after universal_in")
                        })
                        .collect::<Vec<_>>() // necessary to force the closure to release borrowed variables
                })
                .collect::<Vec<_>>() // necessary to force the closure to release borrowed variables
        })
        .collect::<Vec<_>>() // necessary to force the closure to release borrowed variables
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ralg::cgroup::{ideal::CIdeal, C};
    use typenum::{U12, U3, U4};

    #[test]
    fn universal_morphism_in_easy() {
        type R = C<U12>;
        type I = CIdeal<U12>;
        let z2 = Arc::new(CanonModule::<R, I>::from_iter([2]));
        let z3 = Arc::new(CanonModule::<R, I>::from_iter([3]));
        let z4 = Arc::new(CanonModule::<R, I>::from_iter([4]));
        let z34_canon = Arc::new(CanonModule::<R, I>::from_iter([3, 4]));
        let z34_direct = Object::sumproduct(&z3, &z4);
        assert_eq!(
            z34_direct.universal_in(
                &CanonToCanon::new(&z2, &z3, Matrix::from_buffer([0].map(R::from), 1, 1)),
                &CanonToCanon::new(&z2, &z4, Matrix::from_buffer([2].map(R::from), 1, 1)),
            ),
            CanonToCanon::new(
                &z2,
                &z34_canon,
                Matrix::from_buffer([0, 2].map(R::from), 1, 2),
            )
        );
    }

    #[test]
    fn universal_morphism_in_medium() {
        type R = C<U12>;
        type I = CIdeal<U12>;
        let z34 = Arc::new(CanonModule::<R, I>::from_iter([4, 3]));
        let z23 = Arc::new(CanonModule::<R, I>::from_iter([3, 2]));
        let z2334_canon = Arc::new(CanonModule::<R, I>::from_iter([4, 3, 3, 2]));
        let z2334_direct = Object::sumproduct(&z23, &z34);
        let univ_in = z2334_direct.universal_in(
            &CanonToCanon::new(
                &z34,
                &z23,
                Matrix::from_buffer([0, 1, 2, 0].map(R::from), 2, 2),
            ),
            &CanonToCanon::new(
                &z34,
                &z34,
                Matrix::from_buffer([1, 0, 0, 2].map(R::from), 2, 2),
            ),
        );
        let true_output_a = CanonToCanon::new(
            &z34,
            &z2334_canon,
            Matrix::from_buffer([1, 0, 0, 1, 2, 0, 0, 2].map(R::from), 2, 4),
        );
        let true_output_b = CanonToCanon::new(
            &z34,
            &z2334_canon,
            Matrix::from_buffer([0, 1, 1, 0, 2, 0, 0, 2].map(R::from), 2, 4),
        );
        // due to random id, one of those will be true
        assert!(univ_in == true_output_a || univ_in == true_output_b,);
    }

    #[test]
    fn universal_morphism_out_easy() {
        type R = C<U12>;
        type I = CIdeal<U12>;
        let z2 = Arc::new(CanonModule::<R, I>::from_iter([2]));
        let z3 = Arc::new(CanonModule::<R, I>::from_iter([3]));
        let z4 = Arc::new(CanonModule::<R, I>::from_iter([4]));
        let z34_canon = Arc::new(CanonModule::<R, I>::from_iter([4, 3]));
        let z34_direct = Object::sumproduct(&z3, &z4);
        assert_eq!(
            z34_direct.universal_out(
                &CanonToCanon::new(&z3, &z2, Matrix::from_buffer([0].map(R::from), 1, 1)),
                &CanonToCanon::new(&z4, &z2, Matrix::from_buffer([1].map(R::from), 1, 1)),
            ),
            CanonToCanon::new(
                &z34_canon,
                &z2,
                Matrix::from_buffer([0, 1].map(R::from), 2, 1),
            )
        );
    }

    #[test]
    fn universal_morphism_out_medium() {
        type R = C<U12>;
        type I = CIdeal<U12>;
        let z34 = Arc::new(CanonModule::<R, I>::from_iter([4, 3]));
        let z23 = Arc::new(CanonModule::<R, I>::from_iter([3, 2]));
        let z2334_canon = Arc::new(CanonModule::<R, I>::from_iter([4, 3, 3, 2]));
        let z2334_direct = Object::sumproduct(&z23, &z34);
        let univ_out = z2334_direct.universal_out(
            &CanonToCanon::new(
                &z23,
                &z34,
                Matrix::from_buffer([0, 2, 2, 0].map(R::from), 2, 2),
            ),
            &CanonToCanon::new(
                &z34,
                &z34,
                Matrix::from_buffer([1, 0, 0, 3].map(R::from), 2, 2),
            ),
        );
        let true_output_a = CanonToCanon::new(
            &z2334_canon,
            &z34,
            Matrix::from_buffer([0, 1, 2, 0, 2, 0, 0, 3].map(R::from), 4, 2),
        );
        let true_output_b = CanonToCanon::new(
            &z2334_canon,
            &z34,
            Matrix::from_buffer([1, 0, 0, 2, 2, 0, 0, 3].map(R::from), 4, 2),
        );
        // due to random id, one of those will be true
        assert!(univ_out == true_output_a || univ_out == true_output_b);
    }

    #[test]
    #[allow(non_snake_case, reason = "module names look this way")]
    fn sumproduct_of_Z2_and_Z4() {
        type R = C<U4>;
        type I = CIdeal<U4>;
        let z2 = Arc::new(CanonModule::<R, I>::from_iter([2]));
        let z4 = Arc::new(CanonModule::<R, I>::from_iter([4]));
        let z24_canon = Arc::new(CanonModule::<R, I>::from_iter([4, 2]));
        let z24_direct = Object::sumproduct(&z4, &z2);

        assert_eq!(z24_canon, z24_direct.module());
        assert_eq!(
            z24_direct.right_inclusion,
            CanonToCanon::new(
                &z2,
                &z24_canon,
                Matrix::from_buffer([1, 0].map(R::from), 1, 2),
            )
        );
        assert_eq!(
            z24_direct.left_inclusion,
            CanonToCanon::new(
                &z4,
                &z24_canon,
                Matrix::from_buffer([0, 1].map(R::from), 1, 2),
            )
        );
        assert_eq!(
            z24_direct.right_projection,
            CanonToCanon::new(
                &z24_canon,
                &z2,
                Matrix::from_buffer([1, 0].map(R::from), 2, 1),
            )
        );
        assert_eq!(
            z24_direct.left_projection,
            CanonToCanon::new(
                &z24_canon,
                &z4,
                Matrix::from_buffer([0, 1].map(R::from), 2, 1),
            )
        );
    }

    #[test]
    #[allow(non_snake_case, reason = "module names look this way")]
    fn sumproduct_of_split() {
        type R = C<U4>;
        type I = CIdeal<U4>;
        let z2sq = Arc::new(CanonModule::<R, I>::from_iter([2, 2]));
        let z2to4 = Arc::new(CanonModule::<R, I>::from_iter([2, 2, 2, 2]));
        let (l, r) = (*z2to4).clone().split();
        let left = Arc::new(l);
        let right = Arc::new(r);
        let direct = Object::sumproduct(&left, &right);

        assert_eq!(z2to4, direct.module());
        assert_eq!(
            direct.left_inclusion,
            CanonToCanon::new(
                &z2sq,
                &z2to4,
                Matrix::from_buffer([1, 0, 0, 0, 0, 1, 0, 0].map(R::from), 2, 4),
            ),
            "left inclusion"
        );
        assert_eq!(
            direct.right_inclusion,
            CanonToCanon::new(
                &z2sq,
                &z2to4,
                Matrix::from_buffer([0, 0, 1, 0, 0, 0, 0, 1].map(R::from), 2, 4),
            ),
            "right inclusion"
        );
        assert_eq!(
            direct.left_projection,
            CanonToCanon::new(
                &z2to4,
                &z2sq,
                Matrix::from_buffer([1, 0, 0, 0, 0, 0, 1, 0].map(R::from), 4, 2),
            ),
            "left projection"
        );
        assert_eq!(
            direct.right_projection,
            CanonToCanon::new(
                &z2to4,
                &z2sq,
                Matrix::from_buffer([0, 1, 0, 0, 0, 0, 0, 1].map(R::from), 4, 2),
            ),
            "right projection"
        );
    }

    #[test]
    fn sumproduct_and_duplicate() {
        type R = C<U3>;
        type I = CIdeal<U3>;
        let z3 = Arc::new(CanonModule::<R, I>::from_iter([3]));

        let z33_canon = Arc::new(CanonModule::<R, I>::from_iter([3, 3]));
        let z33_direct = Object::sumproduct(&z3, &Arc::new(z3.duplicate()));

        assert_eq!(z33_canon, z33_direct.module());
    }
}
