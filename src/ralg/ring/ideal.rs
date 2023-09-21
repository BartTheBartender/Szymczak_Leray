use crate::ralg::ring::{Bezout, Ring};

pub trait Ideal: Clone + PartialEq + Eq {
    type Parent: Ring;

    fn principal(r: Self::Parent) -> Self;
    // all these (including principal) could probably be a macro
    // fn from_pair(r: Self::Parent, s: Self::Parent) -> Self;
    // fn from_triple(r,s,t) -> Self;

    fn generators(self) -> impl Iterator<Item = Self::Parent>;

    /// is the given element in this ideal?
    fn contains(&self, r: Self::Parent) -> bool;

    fn is_trivial(&self) -> bool;
    fn is_full(&self) -> bool;
}

#[allow(clippy::expect_used, reason = "structural guarantees")]
// TODO check that this Bezout is enforced
pub trait Principal: Ideal<Parent: Bezout> {
    fn generator(self) -> Self::Parent {
        self.generators()
            .reduce(|acc, next| Self::Parent::gcd(acc, next).0)
            .expect("every ideal will have at least one generator")
    }
}
