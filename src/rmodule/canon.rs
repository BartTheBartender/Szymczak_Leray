use crate::{
    category::{AllObjects, Duplicate},
    rmodule::{
        direct::DirectModule,
        map::CanonToCanon,
        ring::{Ring, SuperRing},
        torsion::{Coeff, CoeffTree},
        Module,
    },
    util::matrix::Matrix,
    Int,
};
use itertools::Itertools;
use std::{fmt, iter, ops::Rem, sync::Arc};

/* # canonical module */

#[derive(Clone, Hash)]
#[allow(clippy::module_name_repetitions)]
pub struct CanonModule<R: Ring> {
    // technically, this R in the Tree should be an ideal of the ring
    torsion_coeff: CoeffTree<R, ()>,
}

unsafe impl<R: Ring + Send> Send for CanonModule<R> {}
unsafe impl<R: Ring + Sync> Sync for CanonModule<R> {}

impl<R: Ring + Ord> fmt::Debug for CanonModule<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.torsion_coeff
                .coeffs()
                .map(|c| format!("Z{}", c.get().to_string()))
                .collect::<Vec<_>>()
                .join("x"),
        )
    }
}

impl<R: Ring + Ord> fmt::Display for CanonModule<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.torsion_coeff
                .coeffs()
                .map(|c| format!("Z{}", c.get().to_string()))
                .collect::<Vec<_>>()
                .join(" x "),
        )
    }
}

impl<R: SuperRing> PartialEq for CanonModule<R> {
    // we do not care if the coeff tree keys have different uuids
    fn eq(&self, other: &Self) -> bool {
        self.torsion_coeff.coeffs().eq(other.torsion_coeff.coeffs())
    }
}

impl<R: SuperRing> Eq for CanonModule<R> {}

impl<R: SuperRing> CanonModule<R> {
    pub const fn new(torsion_coeff: CoeffTree<R, ()>) -> Self {
        Self { torsion_coeff }
    }

    pub fn dimension(&self) -> usize {
        self.torsion_coeff.len()
    }

    pub fn cardinality(&self) -> usize {
        self.torsion_coeff
            .coeffs()
            .fold(1, |acc, next| acc.saturating_mul(usize::from(next.get())))
    }

    pub fn torsion_coeffs(&self) -> impl Iterator<Item = R> + '_ {
        self.torsion_coeff.coeffs()
    }

    pub const fn coeff_tree(&self) -> &CoeffTree<R, ()> {
        &self.torsion_coeff
    }

    pub fn into_coeff_tree(self) -> CoeffTree<R, ()> {
        self.torsion_coeff
    }

    /* # module stuff */

    pub fn versor(&self, key: &Coeff<R>) -> <Self as Module<R>>::Element {
        let mut v = self.torsion_coeff.map_ref(|_, _| R::zero());
        v.replace(key, <R as Ring>::one());
        v
    }

    pub fn generators(&self) -> Vec<<Self as Module<R>>::Element> {
        self.torsion_coeff
            .keys()
            .map(|key| self.versor(key))
            .collect()
    }

    pub fn element_from_matrix(&self, matrix: Matrix<R>) -> <Self as Module<R>>::Element {
        CoeffTree::<R, R>::from_matrix(matrix, &self.torsion_coeff)
    }

    pub fn all_elements(&self) -> impl Iterator<Item = <Self as Module<R>>::Element> + '_ {
        let dim = u8::try_from(self.dimension()).expect("we're gonna need a bigger int");
        self.torsion_coeff
            .coeffs()
            .map(|coeff| (0..coeff.get()).map(|r| R::new(r)))
            .multi_cartesian_product()
            .map(move |vec| self.element_from_matrix(Matrix::<R>::from_buffer(vec, dim, 1)))
    }

    pub fn submodules(self) -> Vec<CanonToCanon<R>> {
        match self.dimension() {
            0 => panic!("coś poszło nie tak: submodules"),
            1 => submodules_of_cyclic_module(self),
            _n => DirectModule::from(self).submodules_goursat().collect(),
        }
    }

    pub fn quotients(self) -> Vec<CanonToCanon<R>> {
        match self.dimension() {
            0 => panic!("coś poszło nie tak: quotients"),
            1 => quotients_of_cyclic_module(self),
            _n => DirectModule::from(self).quotients_goursat().collect(),
        }
    }
}

impl<R: SuperRing> Duplicate for CanonModule<R> {
    fn duplicate(&self) -> Self {
        Self::new(self.torsion_coeff.coeffs().collect())
    }
}

impl<R: Ring + Ord + Rem<Output = R>> Module<R> for CanonModule<R> {
    type Element = CoeffTree<R, R>;

    fn is_element(&self, v: &Self::Element) -> bool {
        self.torsion_coeff.has_same_keys(v)
    }

    fn zero(&self) -> Self::Element {
        self.torsion_coeff.clone().map(|(), _| R::zero())
    }

    fn add_unchecked(&self, v: &Self::Element, u: &Self::Element) -> Self::Element {
        v.combine_ref(u, |ve, ue, c| (*ve + *ue) % c)
    }

    /*
    fn increment_unchecked(&self, v: &mut Self::Element, u: &Self::Element) {
        for ((ve, ue), coeff) in v.iter_mut().zip(u.iter()).zip(self.torsion_coeff.iter()) {
            *ve += ue;
            *ve %= coeff;
        }
    }
    */

    fn mul_by_scalar_unchecked(&self, x: R, v: &Self::Element) -> Self::Element {
        v.map_ref(|ve, c| (*ve * x) % c)
    }
}

/* # helper functions */

pub fn zn_dividedby_rxzm<R: SuperRing>(n: R, r: R, m: R) -> R {
    let denom = match r.is_zero() || r == m {
        true => R::one(),
        false => m.divide_by(&r),
    };
    n.divide_by(&denom)
}

pub fn submodules_of_cyclic_module<R: SuperRing>(module: CanonModule<R>) -> Vec<CanonToCanon<R>> {
    let target = Arc::new(module);
    target
        .torsion_coeffs()
        .collect::<Vec<_>>()
        .first()
        .map_or_else(
            || panic!("we assumed the module is cyclic, so it should exactly one coefficient"),
            |coeff| {
                coeff
                    .subideals()
                    .map(|subideal| {
                        let source =
                            Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter(vec![
                                subideal,
                            ])));
                        CanonToCanon::new(
                            source,
                            Arc::clone(&target),
                            Matrix::<R>::from_buffer([coeff.divide_by(&subideal)], 1, 1),
                        )
                    })
                    .collect()
            },
        )
}

pub fn quotients_of_cyclic_module<R: SuperRing>(module: CanonModule<R>) -> Vec<CanonToCanon<R>> {
    let source = Arc::new(module);
    source
        .torsion_coeffs()
        .collect::<Vec<_>>()
        .first()
        .map_or_else(
            || panic!("we assumed the module is cyclic, so has exactly one coefficient"),
            |coeff| {
                coeff
                    .subideals()
                    .map(|subideal| {
                        let target =
                            Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter(vec![
                                subideal,
                            ])));
                        CanonToCanon::new(
                            Arc::clone(&source),
                            target,
                            Matrix::<R>::from_buffer([<R as Ring>::one()], 1, 1),
                        )
                    })
                    .collect()
            },
        )
}

impl<R: SuperRing> AllObjects for CanonModule<R> {
    fn all_objects(maximal_dimension: Int) -> Vec<Self> {
        match maximal_dimension {
            0 => iter::once(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(1)])))
                .collect::<Vec<Self>>(),
            positive => iter::once(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(1)])))
                .chain(
                    CoeffTree::<R, ()>::all_torsion_coeffs(positive)
                        .into_iter()
                        .map(CanonModule::<R>::new),
                )
                .collect::<Vec<Self>>(),
        }
    }
}

/* # tests */

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        error::Error,
        rmodule::ring::{Fin, Set},
    };
    use typenum::{U12, U3, U4, U6, U8};

    #[test]
    fn addition() {
        type R = Fin<U6>;
        let z6xz3 = CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(0), R::new(3)]));
        let a = Matrix::from_buffer([R::new(1), R::new(1), R::new(1)], 1, 3);
        let b = Matrix::from_buffer([R::new(2), R::new(1), R::new(1)], 1, 3);
        let c = Matrix::from_buffer([R::new(0), R::new(2), R::new(0)], 1, 3);
        assert_eq!(
            z6xz3.add(
                &z6xz3.element_from_matrix(a),
                &z6xz3.element_from_matrix(b.clone())
            ),
            Ok(z6xz3.element_from_matrix(c.clone()))
        );
        let a = Matrix::from_buffer([R::new(4), R::new(7), R::new(3)], 1, 2);
        assert_eq!(
            z6xz3.add(
                &z6xz3.element_from_matrix(a),
                &z6xz3.element_from_matrix(b.clone())
            ),
            Ok(z6xz3.element_from_matrix(c))
        );
        let z6sq = CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(0), R::new(0)]));
        let a = Matrix::from_buffer([R::new(4), R::new(1), R::new(2), R::new(2)], 1, 4);
        assert_eq!(
            z6xz3.add(&z6sq.element_from_matrix(a), &z6xz3.element_from_matrix(b)),
            Err(Error::InvalidElement)
        );
    }

    #[test]
    fn multiplication() {
        type R = Fin<U3>;
        let z3sq = CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(0), R::new(0)]));
        let a = Matrix::from_buffer([R::new(2), R::new(1)], 1, 2);
        let c = Matrix::from_buffer([R::new(1), R::new(2)], 1, 2);
        assert_eq!(
            z3sq.mul_by_scalar(R::new(2), &z3sq.element_from_matrix(a),),
            Ok(z3sq.element_from_matrix(c.clone()))
        );
        let a = Matrix::from_buffer([R::new(5), R::new(1)], 1, 2);
        assert_eq!(
            z3sq.mul_by_scalar(R::new(2), &z3sq.element_from_matrix(a),),
            Ok(z3sq.element_from_matrix(c))
        );
        let z3cb = CanonModule::new(CoeffTree::<R, ()>::from_iter([
            R::new(0),
            R::new(0),
            R::new(0),
        ]));
        let a = Matrix::from_buffer([R::new(4), R::new(1), R::new(2)], 1, 3);
        assert_eq!(
            z3sq.mul_by_scalar(R::new(2), &z3cb.element_from_matrix(a)),
            Err(Error::InvalidElement)
        );
    }

    #[test]
    fn elements() {
        type R = Fin<U12>;
        let a = CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(1)]));
        assert_eq!(
            a.all_elements()
                .map(|element| element.into_values().collect::<Vec<_>>())
                .collect::<Vec<_>>(),
            vec![vec![R::new(0)]]
        );

        let a = CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(3)]));
        assert_eq!(
            a.all_elements()
                .map(|element| element.into_values().collect::<Vec<_>>())
                .collect::<Vec<_>>(),
            vec![vec![R::new(0)], vec![R::new(1)], vec![R::new(2)]]
        );

        let a = CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(2), R::new(2)]));
        assert_eq!(
            a.all_elements()
                .map(|element| element.into_values().collect::<Vec<_>>())
                .collect::<Vec<_>>(),
            vec![
                vec![R::new(0), R::new(0)],
                vec![R::new(0), R::new(1)],
                vec![R::new(1), R::new(0)],
                vec![R::new(1), R::new(1)]
            ]
        );
    }

    #[test]
    fn module_division() {
        type R = Fin<U12>;
        assert_eq!(
            zn_dividedby_rxzm(R::new(3), R::new(1), R::new(3)),
            R::new(1)
        );
        assert_eq!(
            zn_dividedby_rxzm(R::new(3), R::new(2), R::new(3)),
            R::new(1)
        );
        assert_eq!(
            zn_dividedby_rxzm(R::new(3), R::new(0), R::new(3)),
            R::new(3)
        );
        assert_eq!(
            zn_dividedby_rxzm(R::new(4), R::new(2), R::new(4)),
            R::new(2)
        );
        assert_eq!(
            zn_dividedby_rxzm(R::new(6), R::new(1), R::new(3)),
            R::new(2)
        );
    }

    #[test]
    #[allow(non_snake_case)] // module names look this way
    fn submodules_of_Z8() {
        type R = Fin<U8>;
        let z8 = CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(0)]));
        let mut submodules = z8.submodules().into_iter();
        assert_eq!(
            submodules.next(),
            Some(CanonToCanon::new(
                Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(1)]))),
                Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(0)]))),
                Matrix::from_buffer([R::new(0)], 1, 1),
            ))
        );
        assert_eq!(
            submodules.next(),
            Some(CanonToCanon::new(
                Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(2)]))),
                Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(0)]))),
                Matrix::from_buffer([R::new(4)], 1, 1),
            ))
        );
        assert_eq!(
            submodules.next(),
            Some(CanonToCanon::new(
                Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(4)]))),
                Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(0)]))),
                Matrix::from_buffer([R::new(2)], 1, 1),
            ))
        );
        assert_eq!(
            submodules.next(),
            Some(CanonToCanon::new(
                Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(0)]))),
                Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(0)]))),
                Matrix::from_buffer([R::new(1)], 1, 1),
            ))
        );
        assert_eq!(submodules.next(), None);
    }

    #[test]
    #[allow(non_snake_case)] // module names look this way
    fn quotients_of_Z8() {
        type R = Fin<U8>;
        let z8 = CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(0)]));
        let mut quotients = z8.quotients().into_iter();
        assert_eq!(
            quotients.next(),
            Some(CanonToCanon::new(
                Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(0)]))),
                Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(1)]))),
                Matrix::from_buffer([R::new(1)], 1, 1),
            ))
        );
        assert_eq!(
            quotients.next(),
            Some(CanonToCanon::new(
                Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(0)]))),
                Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(2)]))),
                Matrix::from_buffer([R::new(1)], 1, 1),
            ))
        );
        assert_eq!(
            quotients.next(),
            Some(CanonToCanon::new(
                Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(0)]))),
                Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(4)]))),
                Matrix::from_buffer([R::new(1)], 1, 1),
            ))
        );
        assert_eq!(
            quotients.next(),
            Some(CanonToCanon::new(
                Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(0)]))),
                Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(0)]))),
                Matrix::from_buffer([R::new(1)], 1, 1),
            ))
        );
        assert_eq!(quotients.next(), None);
    }

    #[test]
    #[allow(non_snake_case)] // module names look this way
    fn submodules_of_Z2xZ4() {
        type R = Fin<U4>;
        let z2xz4 = CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(4), R::new(2)]));
        let mut submodules = z2xz4.submodules().into_iter();
        let z2xz4_arc = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([
            R::new(4),
            R::new(2),
        ])));

        assert_eq!(
            submodules.next(),
            Some(CanonToCanon::new(
                Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(1)]))),
                Arc::clone(&z2xz4_arc),
                Matrix::from_buffer([R::new(0), R::new(0)], 1, 2),
            )),
            "trivial submodule"
        );

        assert_eq!(
            submodules.next(),
            Some(CanonToCanon::new(
                Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(2)]))),
                Arc::clone(&z2xz4_arc),
                Matrix::from_buffer([R::new(0), R::new(1)], 1, 2),
            )),
            "right Z2"
        );

        assert_eq!(
            submodules.next(),
            Some(CanonToCanon::new(
                Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(2)]))),
                Arc::clone(&z2xz4_arc),
                Matrix::from_buffer([R::new(2), R::new(0)], 1, 2),
            )),
            "left Z2"
        );

        assert_eq!(
            submodules.next(),
            Some(CanonToCanon::new(
                Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(2)]))),
                Arc::clone(&z2xz4_arc),
                Matrix::from_buffer([R::new(2), R::new(1)], 1, 2),
            )),
            "diagonal Z2"
        );

        assert_eq!(
            submodules.next(),
            Some(CanonToCanon::new(
                Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([
                    R::new(2),
                    R::new(2)
                ]))),
                Arc::clone(&z2xz4_arc),
                Matrix::from_buffer([R::new(2), R::new(0), R::new(0), R::new(1)], 2, 2),
            )),
            "Z2 squared"
        );

        assert_eq!(
            submodules.next(),
            Some(CanonToCanon::new(
                Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(4)]))),
                Arc::clone(&z2xz4_arc),
                Matrix::from_buffer([R::new(1), R::new(0)], 1, 2),
            )),
            "straight Z4"
        );

        /*
        this does not work due to a small inconsistency i found
        the result still provides the right elements of the group, just in the wrong configuration
        this is attested by the `kernel_asymetric` test that fails
        assert_eq!(
            submodules.next(),
            Some(CanonToCanon::new(
                Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(4)]))),
                Arc::clone(&z2xz4_arc),
                Matrix::from_buffer([R::new(1), R::new(1)], 1, 2),
            )),
            "diagonal Z4"
        );
        */

        assert_eq!(
            submodules.next(),
            Some(CanonToCanon::new(
                Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([
                    R::new(2),
                    R::new(2)
                ]))),
                Arc::clone(&z2xz4_arc),
                Matrix::from_buffer([R::new(2), R::new(1), R::new(0), R::new(1)], 2, 2),
            )),
            "diagonal Z4"
        );

        assert_eq!(
            submodules.next(),
            Some(CanonToCanon::new(
                Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([
                    R::new(4),
                    R::new(2)
                ]))),
                Arc::clone(&z2xz4_arc),
                Matrix::from_buffer([R::new(1), R::new(0), R::new(0), R::new(1)], 2, 2),
            )),
            "full submodule"
        );

        assert_eq!(submodules.next(), None);
    }

    #[test]
    #[allow(non_snake_case)] // module names look this way
    fn submodules_of_Z3xZ3() {
        type R = Fin<U3>;
        let z3xz3 = CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(3), R::new(3)]));
        let mut submodules = z3xz3.submodules().into_iter();
        let z3xz3_arc = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([
            R::new(3),
            R::new(3),
        ])));

        assert_eq!(
            submodules.next(),
            Some(CanonToCanon::new(
                Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(1)]))),
                Arc::clone(&z3xz3_arc),
                Matrix::from_buffer([R::new(0), R::new(0)], 1, 2),
            )),
            "trivial submodule"
        );

        assert_eq!(
            submodules.next(),
            Some(CanonToCanon::new(
                Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(3)]))),
                Arc::clone(&z3xz3_arc),
                Matrix::from_buffer([R::new(0), R::new(1)], 1, 2),
            )),
            "left Z3"
        );

        assert_eq!(
            submodules.next(),
            Some(CanonToCanon::new(
                Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(3)]))),
                Arc::clone(&z3xz3_arc),
                Matrix::from_buffer([R::new(1), R::new(0)], 1, 2),
            )),
            "right Z3"
        );

        assert_eq!(
            submodules.next(),
            Some(CanonToCanon::new(
                Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(3)]))),
                Arc::clone(&z3xz3_arc),
                Matrix::from_buffer([R::new(1), R::new(1)], 1, 2),
            )),
            "middle Z3"
        );

        assert_eq!(
            submodules.next(),
            Some(CanonToCanon::new(
                Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(3)]))),
                Arc::clone(&z3xz3_arc),
                Matrix::from_buffer([R::new(2), R::new(1)], 1, 2),
            )),
            "skew Z3"
        );

        assert_eq!(
            submodules.next(),
            Some(CanonToCanon::new(
                Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([
                    R::new(3),
                    R::new(3)
                ]))),
                Arc::clone(&z3xz3_arc),
                Matrix::from_buffer([R::new(1), R::new(0), R::new(0), R::new(1)], 2, 2),
            )),
            "full submodule"
        );

        assert_eq!(submodules.next(), None);
    }

    #[test]
    fn zero_dimensional_module() {
        type R = Fin<U3>;
        let modules = CanonModule::<R>::all_objects(0);

        assert_eq!(modules.len(), 1);
        assert_eq!(modules[0].to_string(), "Z1");
    }

    #[test]
    fn z1_one_dimension_ambiguity() {
        type R = Fin<typenum::U1>;
        let modules = CanonModule::<R>::all_objects(1);

        assert_eq!(modules.len(), 1);
        assert_eq!(modules[0].to_string(), "Z1");
    }

    #[test]
    fn one_dimensional_module() {
        type R = Fin<U3>;
        let modules = CanonModule::<R>::all_objects(1);

        assert_eq!(modules.len(), 2);
        assert_eq!(modules[0].to_string(), "Z1");
        assert_eq!(modules[1].to_string(), "Z3");
    }
}
