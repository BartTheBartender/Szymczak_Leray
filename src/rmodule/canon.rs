use crate::{
    category::{AllObjects, Duplicate},
    ralg::{
        ideal::PrincipalIdeal,
        ring::{BezoutRing, FactorialRing},
    },
    rmodule::{
        quotient::Object as QuotientObject,
        torsion::{Element as TorsionElement, Object as TorsionCoeff},
        ModuleObject,
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

    pub fn generators(&self) -> Vec<<Self as Module<R>>::Element> {
        self.torsion_coeff
            .keys()
            .map(|key| self.versor(key))
            .collect()
    }
}
