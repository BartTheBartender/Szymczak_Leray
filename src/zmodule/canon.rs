use crate::{
    category::morphism::{Compose, Morphism},
    util::{
        iterator::{product, Dedup},
        number::{are_coprime, divisors, versor},
    },
    zmodule::{
        coset::{Coset, CosetZModule},
        map::{CanonToCanon, CanonToCoset},
        ZModule,
    },
    Int, TorsionCoeff,
};

use itertools::*;
use std::{
    collections::HashMap,
    fmt::{self, Display},
    sync::Arc,
};

/* # torsion coefficients */

pub type Zahl = Int;

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

/* # canonical z module */

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CanonZModule {
    torsion_coeff: TorsionCoeff,
}

impl CanonZModule {
    pub fn new_unchecked(torsion_coeff: TorsionCoeff) -> Self {
        Self { torsion_coeff }
    }

    pub fn new(torsion_coeff: TorsionCoeff) -> Self {
        Self::new_unchecked(canonise_torsion_coeff(torsion_coeff))
    }

    pub fn dimension(&self) -> usize {
        self.torsion_coeff.len()
    }

    pub fn torsion_coeff(&self) -> TorsionCoeff {
        self.torsion_coeff.clone()
    }

    // to było twoje `new`
    pub fn product(left: CanonZModule, right: CanonZModule) -> Self {
        Self {
            torsion_coeff: [left.torsion_coeff, right.torsion_coeff].concat(),
        }
    }

    pub fn generators(&self) -> Vec<<Self as ZModule>::Element> {
        let dim = self.dimension();
        (0..dim).map(|index| versor(index, dim)).collect()
    }

    pub fn all_elements(&self) -> Vec<<Self as ZModule>::Element> {
        self.torsion_coeff
            .iter()
            .map(|coeff| 0..*coeff)
            .multi_cartesian_product()
            .collect()
    }

    pub fn submodules(self) -> Vec<CanonToCanon> {
        match self.dimension() {
            0 => panic!("coś poszło nie tak"),
            1 => submodules_of_cyclic_module(self),
            n => {
                // split in half
                // goursat
                todo!()
            }
        }
    }

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

    fn cosets(&self, subgroup: CanonToCanon) -> CanonToCoset {
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

    fn quotient(&self, subgroup: CanonToCanon) -> CanonToCanon {
        let cosets = self.cosets(subgroup);
        cosets.compose_unchecked(&cosets.target().canonise())
        // compose_canon_coset_canon(cosets.target().canonise(), cosets)
    }

    pub fn cardinality(&self) -> usize {
        self.torsion_coeff.iter().product::<Int>() as usize
    }
}

impl ZModule for CanonZModule {
    type Element = Vec<Zahl>;

    fn is_element(&self, v: &Self::Element) -> bool {
        self.dimension() == v.len()
    }

    fn zero(&self) -> Self::Element {
        vec![0; self.dimension()]
    }

    fn add_unchecked(&self, v: &Self::Element, u: &Self::Element) -> Self::Element {
        self.torsion_coeff
            .iter()
            // zipping instead of indexing does not perform a bounds check at every indexing attempt
            // so is both faster and safer ;)
            .zip(v.iter().zip(u.iter()))
            .map(|(coeff, (ve, ue))| (ve + ue) % coeff)
            .collect()
    }

    fn increment_unchecked(&self, v: &mut Self::Element, u: &Self::Element) {
        for ((ve, ue), coeff) in v.iter_mut().zip(u.iter()).zip(self.torsion_coeff.iter()) {
            *ve += ue;
            *ve %= coeff;
        }
    }

    fn mul_by_scalar_unchecked(&self, x: Zahl, v: &Self::Element) -> Self::Element {
        self.torsion_coeff
            .iter()
            .zip(v.iter())
            .map(|(coeff, ve)| (ve * x) % coeff)
            .collect()
    }
}

pub fn submodules_of_cyclic_module(module: CanonZModule) -> Vec<CanonToCanon> {
    let target = Arc::new(module);
    divisors(
        target
            .as_ref()
            .dimension()
            .try_into()
            .expect("we're gonna need a bigger int"),
    )
    .into_iter()
    .map(|divisor| {
        let source = Arc::new(CanonZModule::new_unchecked(vec![divisor]));
        CanonToCanon::new_unchecked(source, target.clone(), vec![vec![divisor]])
    })
    .collect()
}

impl Display for CanonZModule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.torsion_coeff
                .iter()
                .map(|x| {
                    let y = x.to_string();
                    y
                })
                .collect::<String>()
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::error::Error;

    #[test]
    fn canonising_torsion_coefficients() {
        assert_eq!(canonise_torsion_coeff(vec![2, 2]), vec![2, 2]);
        assert_eq!(canonise_torsion_coeff(vec![2, 3, 3]), vec![3, 6]);
        assert_eq!(canonise_torsion_coeff(vec![2, 4, 3, 2]), vec![2, 4, 6]);
    }

    #[test]
    fn addition() {
        let z3sq = CanonZModule::new(vec![3, 3]);
        assert_eq!(z3sq.add(&vec![1, 1], &vec![2, 1]), Ok(vec![0, 2]));
        assert_eq!(z3sq.add(&vec![4, 1], &vec![2, 1]), Ok(vec![0, 2]));
        assert_eq!(
            z3sq.add(&vec![4, 1, 2], &vec![2, 1]),
            Err(Error::InvalidElement)
        );

        let mut x = vec![1, 2];
        let r = z3sq.increment(&mut x, &vec![1, 1]);
        assert!(r.is_ok());
        assert_eq!(x, vec![2, 0]);

        let r = z3sq.increment(&mut x, &vec![1]);
        assert!(r.is_err());
        assert_eq!(x, vec![2, 0]);
    }

    #[test]
    fn multiplication() {
        let z3sq = CanonZModule::new(vec![3, 3]);
        assert_eq!(z3sq.mul_by_scalar(2, &vec![2, 1]), Ok(vec![1, 2]));
        assert_eq!(z3sq.mul_by_scalar(5, &vec![2, 4]), Ok(vec![1, 2]));
        assert_eq!(
            z3sq.mul_by_scalar(2, &vec![2, 1, 3]),
            Err(Error::InvalidElement)
        );
    }
}
