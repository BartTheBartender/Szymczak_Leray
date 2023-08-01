#![allow(unused_imports)] // DELETE LATER
use crate::{
    category::morphism::{Compose, Morphism},
    matrix::Matrix,
    rmodule::{
        map::CanonToCanon,
        ring::{Radix, Ring, SuperRing},
        torsion::{Coeff, CoeffTree},
        Module,
    },
    util::{
        iterator::{product, Dedup},
        number::{are_coprime, divisors, versor},
    },
    Int, TorsionCoeff,
};

use itertools::*;
use std::{collections::HashMap, ops::Rem, sync::Arc};

/* # torsion coefficients */

/*
pub fn all_torsion_coeffs(base: Zahl, max_dimension: Zahl) -> HashMap<Zahl, Vec<TorsionCoeff>> {
    (1..max_dimension + 1)
        .map(|dimension| (dimension, all_torsion_coeffs_fixed_dim(base, dimension)))
        .collect()
}

fn all_torsion_coeffs_fixed_dim(base: Zahl, dimension: Zahl) -> Vec<TorsionCoeff> {
    product(divisors(base), dimension)
}

pub fn canonise_torsion_coeff(torsion_coeff: TorsionCoeff) -> TorsionCoeff {
    // combine all relatively prime elements
    // może jest lepszy sposób by to zrobić
    // zastanowię się nad tym później
    let mut torsion_coeff = torsion_coeff;
    let mut new_torsion_coeff = Vec::<TorsionCoeff>::new();
    'outer: while let Some(x) = torsion_coeff.pop() {
        for class in new_torsion_coeff.iter_mut() {
            if class.iter().all(|&y| are_coprime(x, y)) {
                class.push(x);
                continue 'outer;
            }
        }
        new_torsion_coeff.push(vec![x]);
    }

    // sort the resulting vec
    new_torsion_coeff
        .into_iter()
        .flat_map(|class| class.into_iter().reduce(|acc, next| acc * next))
        .sorted()
        .collect()
}
*/

/* # canonical module */

// #[derive(Clone, PartialEq, Eq, Hash)]
#[derive(PartialEq, Eq)]
pub struct CanonModule<RC: Radix, R: Ring<RC>> {
    // technically, this R in the Tree should be an ideal of the ring
    torsion_coeff: CoeffTree<R, ()>,
    _rc: std::marker::PhantomData<RC>,
}

impl<RC: Radix, R: SuperRing<RC>> CanonModule<RC, R> {
    pub fn new(torsion_coeff: CoeffTree<R, ()>) -> Self {
        Self {
            torsion_coeff,
            _rc: std::marker::PhantomData,
        }
    }

    pub fn dimension(&self) -> usize {
        self.torsion_coeff.len()
    }

    // pub fn cardinality(&self) -> usize {
    //     match self.torsion_coeff.coeffs().reduce(|acc, next| acc * next) {
    //         None => 0,
    //         Some(product) => product.get() as usize,
    //     }
    // }

    pub fn torsion_coeffs(&self) -> impl Iterator<Item = R> + '_ {
        self.torsion_coeff.coeffs()
    }

    pub fn coeff_tree(&self) -> &CoeffTree<R, ()> {
        &self.torsion_coeff
    }

    /* # module stuff */

    pub fn versor(&self, key: &Coeff<R>) -> <Self as Module<RC, R>>::Element {
        let mut v = self.torsion_coeff.map_ref(|_, _| R::zero());
        v.replace(key, <R as Ring<RC>>::one());
        v
    }

    pub fn generators(&self) -> Vec<<Self as Module<RC, R>>::Element> {
        self.torsion_coeff
            .keys()
            .map(|key| self.versor(key))
            .collect()
    }

    pub fn element_from_matrix(&self, matrix: Matrix<R>) -> <Self as Module<RC, R>>::Element {
        CoeffTree::<R, R>::from_matrix(matrix, &self.torsion_coeff)
    }

    /*
    pub fn all_elements(&self) -> Vec<<Self as ZModule>::Element> {
        self.torsion_coeff
            .iter()
            .map(|coeff| 0..*coeff)
            .multi_cartesian_product()
            .collect()
    }
    */

    pub fn submodules(self) -> Vec<CanonToCanon<RC, R>> {
        match self.dimension() {
            0 => panic!("coś poszło nie tak"),
            1 => submodules_of_cyclic_module(self),
            _n => todo!(), //BiProductZModule::from(self).submodules_goursat(),
        }
    }

    /* the following fns should be somewhere else
    fn coset(
        &self,
        element: &<Self as ZModule>::Element,
        image_of_subgroup: &Coset<Self>,
    ) -> Coset<Self> {
        Coset::new(
            image_of_subgroup
                .set
                .iter()
                .map(|el| self.add_unchecked(el, element))
                .collect(),
        )
    }

    fn cosets(&self, subgroup: &CanonToCanon) -> CanonToCoset {
        let imgroup: Coset<Self> = Coset::new(subgroup.image());
        let mut cos = Vec::new();
        let mut hom = HashMap::new();
        for element in self.all_elements() {
            let im = self.coset(&element, &imgroup);
            cos.push(im.clone());
            hom.insert(element, im);
        }
        cos.clear_duplicates();
        let source = Arc::new(self.clone());
        CanonToCoset::new(
            source.clone(),
            Arc::new(CosetZModule::new(cos, source)),
            hom,
        )
    }

    // this can be replaced by a cokernel
    fn quotient(&self, subgroup: &CanonToCanon) -> CanonToCanon {
        let cosets = self.cosets(subgroup);
        cosets.compose_unchecked(&cosets.target().canonise())
    }
    */
}

impl<RC: Radix, R: Ring<RC> + Ord + Rem<Output = R>> Module<RC, R> for CanonModule<RC, R> {
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

pub fn submodules_of_cyclic_module<RC: Radix, R: SuperRing<RC>>(
    module: CanonModule<RC, R>,
) -> Vec<CanonToCanon<RC, R>> {
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
            CanonToCanon::new_unchecked(
                source,
                target.clone(),
                Matrix::<R>::from_buffer([subideal], 1, 1),
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
    use typenum::U3;

    #[test]
    fn addition() {
        type R = Fin<U3>;
        let z3sq = CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(0), R::new(0)]));
        let a = Matrix::from_buffer([R::new(1), R::new(1)], 1, 2);
        let b = Matrix::from_buffer([R::new(2), R::new(1)], 1, 2);
        let c = Matrix::from_buffer([R::new(0), R::new(2)], 1, 2);
        assert_eq!(
            z3sq.add(
                &z3sq.element_from_matrix(a),
                &z3sq.element_from_matrix(b.clone())
            ),
            Ok(z3sq.element_from_matrix(c.clone()))
        );
        let a = Matrix::from_buffer([R::new(4), R::new(1)], 1, 2);
        assert_eq!(
            z3sq.add(
                &z3sq.element_from_matrix(a),
                &z3sq.element_from_matrix(b.clone())
            ),
            Ok(z3sq.element_from_matrix(c))
        );
        let z3cb = CanonModule::new(CoeffTree::<R, ()>::from_iter([
            R::new(0),
            R::new(0),
            R::new(0),
        ]));
        let a = Matrix::from_buffer([R::new(4), R::new(1), R::new(2)], 1, 3);
        assert_eq!(
            z3sq.add(&z3cb.element_from_matrix(a), &z3sq.element_from_matrix(b)),
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
