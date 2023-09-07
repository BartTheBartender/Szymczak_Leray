use crate::ralg::ring::{AdditivePartialMonoid, Demesne, Ring};
use itertools::Itertools;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Quotient<R: Ring, I: Ideal<R>, E> {
    /// generator of the ideal that the ring is divided by
    generator: I,
    index: E,
}

pub type Object<R: Ring, I: Ideal<R>> = Quotient<R, I, ()>;
pub type Element<R: Ring, I: ideal<R>> = Quotient<R, I, R>;

/* # helper functions */

impl<R: Ring> Quotient<R, ()> {
    pub fn new(r: R) -> Self {
        Self {
            generator: r.min_generator_of_ideal(),
            index: (),
        }
    }
}

/* ## algebraic structure */

/* ### demesne */

impl<R: Ring + Copy> Demesne for Quotient<R, ()> {
    /**
    this generates all quotients of the ring,
    although rather slowly
    */
    fn elements() -> impl Iterator<Item = Self> + Clone {
        R::elements()
            .map(Self::new)
            .sorted_by_key(|&r| <Self as Into<u16>>::into(r))
            .dedup()
    }
}

impl<R: Ring + Copy> Demesne for Quotient<R, R> {
    fn elements() -> impl Iterator<Item = Self> + Clone {
        // how do we represent an element of a quotent class?
        // on one hand, the numbers from 0 to the torsion coeff seem reasonable,
        // but how would that work with higher dimensional rings?
        // well if we have higher rings, ideals generated by one element are no longer the only ideals
        // so the quotient should really be a collection of elements
        // but that makes stuff soooo complicated
        Quotient::<R, ()>::elements().flat_map(|module| todo!())
    }
}

/*
impl<R: Ring> AdditivePartialMonoid for Quotient<R, R> {}
*/
