use crate::ralg::{
    module::{canon::MarkTree, quotient::Element as QuotientElement, Module},
    ring::{ideal::Ideal, AdditivePartialGroup, AdditivePartialMonoid, Demesne, Ring},
};
use std::fmt;

/* # torsion coefficients object */

#[allow(type_alias_bounds, reason = "waiting on feature `lazy_type_alias`")]
pub type Element<R: Ring, I: Ideal<Parent = R> + Ord> = MarkTree<QuotientElement<R, I>>;

impl<R: Ring + Copy, I: Ideal<Parent = R> + Ord> Element<R, I> {
    /* # structure */

    pub fn has_same_coeffs(&self, other: &Self) -> bool {
        itertools::equal(
            self.buffer.iter().map(|mark| &mark.thing.ideal),
            other.buffer.iter().map(|mark| &mark.thing.ideal),
        )
    }

    pub fn is_equal(&self, other: &Self) -> Option<bool> {
        self.has_same_coeffs(other).then_some(
            self.buffer
                .iter()
                .zip(other.buffer.iter())
                .map(|(self_mark, other_mark)| self_mark.thing.is_equal(&other_mark.thing))
                .try_fold(true, |acc, next| next.map(|n| acc && n))?,
        )
    }
}

impl<R: Ring, I: Ideal<Parent = R> + Ord> Element<R, I> {
    /* # iterators */

    pub fn into_values(self) -> impl Iterator<Item = R> {
        self.buffer.into_iter().map(|mark| mark.thing.element)
        // for some reason BTreeSet::IntoIter does not implement clone
        // .collect::<Vec<_>>()
        // .into_iter()
    }
}

/* ## debug and display */

impl<R: Ring + fmt::Debug, I: Ideal<Parent = R> + Ord + fmt::Debug> fmt::Debug for Element<R, I>
where
    QuotientElement<R, I>: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.is_empty() {
            true => write!(f, "0"),
            false => {
                write!(
                    f,
                    "{}",
                    self.buffer
                        .iter()
                        .map(|mark| format!("{:?}", mark.thing))
                        .collect::<Vec<_>>()
                        .join(" x "),
                )
            }
        }
    }
}

impl<R: Ring + fmt::Display, I: Ideal<Parent = R> + Ord + fmt::Display> fmt::Display
    for Element<R, I>
where
    QuotientElement<R, I>: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.is_empty() {
            true => write!(f, "0"),
            false => {
                write!(
                    f,
                    "{}",
                    self.buffer
                        .iter()
                        .map(|mark| format!("Z{}", mark.thing.ideal))
                        .collect::<Vec<_>>()
                        .join("x"),
                )
            }
        }
    }
}

/* ## module strucutre */

impl<R: Ring, I: Ideal<Parent = R> + Ord> Demesne for Element<R, I> {}

impl<R: Ring + Copy, I: Ideal<Parent = R> + Ord> AdditivePartialMonoid for Element<R, I> {
    fn try_add(self, other: Self) -> Option<Self> {
        self.has_same_coeffs(&other).then_some(Self {
            buffer: self
                .buffer
                .into_iter()
                .zip(other.buffer.into_iter().map(|mark| mark.thing))
                .map(|(self_mark, other_thing)| {
                    self_mark
                        .map(|self_thing| self_thing.try_add(other_thing))
                        .transmute()
                })
                .try_collect()?,
        })
    }

    fn own_zero(&self) -> Self {
        Self {
            buffer: self
                .buffer
                .iter()
                .map(|mark| mark.clone().map(|element| element.own_zero()))
                .collect(),
        }
    }

    fn is_zero(&self) -> bool {
        self.buffer.iter().all(|mark| mark.thing.is_zero())
    }

    fn is_negable(&self) -> bool {
        true
    }

    fn try_neg(self) -> Option<Self> {
        Some(self.neg())
    }
}

impl<R: Ring + Copy, I: Ideal<Parent = R> + Ord> AdditivePartialGroup for Element<R, I> {
    fn neg(self) -> Self {
        Self {
            buffer: self
                .buffer
                .into_iter()
                .map(|mark| mark.map(AdditivePartialGroup::neg))
                .collect(),
        }
    }

    fn neg_inplace(&mut self) {
        // BTreeSet is not mutable
        self.buffer = self.clone().neg().buffer;
    }
}

impl<R: Ring + Copy, I: Ideal<Parent = R> + Ord> Module<R> for Element<R, I> {
    fn mul(self, r: R) -> Self {
        Self {
            buffer: self
                .buffer
                .into_iter()
                .map(|mark| mark.map(|thing| thing.mul(r)))
                .collect(),
        }
    }

    fn mul_assign(&mut self, r: R) {
        // BTreeSet is not mutable
        self.buffer = self.clone().mul(r).buffer;
    }
}

// - - -

#[cfg(test)]
mod test {
    use super::*;
    use crate::ralg::{
        cgroup::{ideal::CIdeal, C},
        module::canon::object::Object,
    };
    use typenum::{U3, U6};

    #[test]
    fn equality() {
        type N = U6;
        let z223 =
            Object::<C<N>, CIdeal<N>>::from_iter([0, 3].map(|j| CIdeal::principal(C::from(j))));
        let z2233 =
            Object::<C<N>, CIdeal<N>>::from_iter([0, 0].map(|j| CIdeal::principal(C::from(j))));

        let a = z223.element_from_iterator([1, 1, 1].into_iter().map(C::from));
        let b = z223.element_from_iterator([1, 1, 2].into_iter().map(C::from));
        let c = z223.element_from_iterator([1, 1, 4].into_iter().map(C::from));
        let d = z2233.element_from_iterator([1, 1, 1, 0].into_iter().map(C::from));
        assert_eq!(a.is_equal(&b), Some(false));
        assert_eq!(a.is_equal(&c), Some(true));
        assert_eq!(a.is_equal(&d), None);
    }

    #[test]
    #[allow(clippy::shadow_unrelated, reason = "useful in test")]
    fn addition() {
        type N = U6;
        let z223 =
            Object::<C<N>, CIdeal<N>>::from_iter([0, 3].map(|j| CIdeal::principal(C::from(j))));

        let a = z223.element_from_iterator([1, 1, 1].into_iter().map(C::from));
        let b = z223.element_from_iterator([1, 0, 2].into_iter().map(C::from));
        let c = z223.element_from_iterator([0, 1, 0].into_iter().map(C::from));
        assert_eq!(
            a.try_add(b.clone()).and_then(|e| e.is_equal(&c)),
            Some(true)
        );

        let a = z223.element_from_iterator([3, 7, 4].into_iter().map(C::from));
        assert_eq!(
            a.try_add(b.clone()).and_then(|e| e.is_equal(&c)),
            Some(true)
        );

        let z6sq =
            Object::<C<N>, CIdeal<N>>::from_iter([0, 0].map(|j| CIdeal::principal(C::from(j))));
        let a = z6sq.element_from_iterator([4, 1, 2, 2].into_iter().map(C::from));
        assert_eq!(a.try_add(b), None);
    }

    #[test]
    #[allow(clippy::shadow_unrelated, reason = "useful in test")]
    fn multiplication() {
        type N = U3;
        let z3sq =
            Object::<C<N>, CIdeal<N>>::from_iter([0, 0].map(|j| CIdeal::principal(C::from(j))));

        let a = z3sq.element_from_iterator([2, 1].into_iter().map(C::from));
        let b = z3sq.element_from_iterator([1, 2].into_iter().map(C::from));
        assert_eq!(a.mul(C::from(2)).is_equal(&b), Some(true));

        let a = z3sq.element_from_iterator([5, 1].into_iter().map(C::from));
        let b = z3sq.element_from_iterator([4, 2].into_iter().map(C::from));
        assert_eq!(a.mul(C::from(2)).is_equal(&b), Some(true));
    }
}
