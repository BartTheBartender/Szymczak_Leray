use crate::{
    matrix::Matrix,
    rmodule::{
        direct::DirectModule,
        map::CanonToCanon,
        ring::{Ring, SuperRing},
        torsion::{Coeff, CoeffTree},
        Module,
    },
};
use itertools::Itertools;
use std::{fmt, ops::Rem, sync::Arc};

/* # canonical module */

#[derive(Clone, Hash)]
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

impl<R: SuperRing> PartialEq for CanonModule<R> {
    // we do not care if the coeff tree keys have different uuids
    fn eq(&self, other: &Self) -> bool {
        self.torsion_coeff.coeffs().eq(other.torsion_coeff.coeffs())
    }
}

impl<R: SuperRing> Eq for CanonModule<R> {}

impl<R: SuperRing> CanonModule<R> {
    pub fn new(torsion_coeff: CoeffTree<R, ()>) -> Self {
        Self { torsion_coeff }
    }

    pub fn dimension(&self) -> usize {
        self.torsion_coeff.len()
    }

    pub fn cardinality(&self) -> usize {
        self.torsion_coeff
            .coeffs()
            .fold(1, |acc, next| acc * next.get() as usize)
    }

    pub fn torsion_coeffs(&self) -> impl Iterator<Item = R> + '_ {
        self.torsion_coeff.coeffs()
    }

    pub fn coeff_tree(&self) -> &CoeffTree<R, ()> {
        &self.torsion_coeff
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
            .map(move |vec| self.element_from_matrix(Matrix::<R>::from_buffer(vec, 1, dim)))
    }

    pub fn submodules(self) -> Vec<CanonToCanon<R>> {
        match self.dimension() {
            0 => panic!("coś poszło nie tak"),
            1 => submodules_of_cyclic_module(self),
            _n => DirectModule::from(self).submodules_goursat(),
        }
    }

    pub fn quotients(self) -> Vec<CanonToCanon<R>> {
        match self.dimension() {
            0 => panic!("coś poszło nie tak"),
            1 => quotients_of_cyclic_module(self),
            _n => DirectModule::from(self).quotients_goursat(),
        }
    }
}

impl<R: Ring + Ord + Rem<Output = R>> Module<R> for CanonModule<R> {
    type Element = CoeffTree<R, R>;

    fn is_element(&self, v: &Self::Element) -> bool {
        self.torsion_coeff.has_same_keys(v)
    }

    fn zero(&self) -> Self::Element {
        self.torsion_coeff.clone().map(|_, _| R::zero())
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

pub fn submodules_of_cyclic_module<R: SuperRing>(module: CanonModule<R>) -> Vec<CanonToCanon<R>> {
    let target = Arc::new(module);
    let out = target
        .torsion_coeffs()
        .next()
        .expect("we assumed the module is cyclic, so has exactly one coefficient")
        .subideals()
        .map(|subideal| {
            let source = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter(vec![
                subideal,
            ])));
            CanonToCanon::new(
                source,
                target.clone(),
                Matrix::<R>::from_buffer([subideal], 1, 1),
            )
        })
        .collect();
    out
}

pub fn quotients_of_cyclic_module<R: SuperRing>(module: CanonModule<R>) -> Vec<CanonToCanon<R>> {
    let source = Arc::new(module);
    let out = source
        .torsion_coeffs()
        .next()
        .expect("we assumed the module is cyclic, so has exactly one coefficient")
        .subideals()
        .map(|subideal| {
            let target = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter(vec![
                subideal,
            ])));
            CanonToCanon::new(
                source.clone(),
                target,
                Matrix::<R>::from_buffer([<R as Ring>::one()], 1, 1),
            )
        })
        .collect();
    out
}

/* # tests */

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        error::Error,
        rmodule::ring::{Fin, Set},
    };
    use typenum::{U3, U6};

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
}
