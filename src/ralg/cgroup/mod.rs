use crate::ralg::{
    ring::{
        AdditiveGroup, AdditiveMonoid, AdditivePartialGroup, AdditivePartialMonoid,
        Bezout as BezoutRing, Demesne, Enumerable, Factorial as FactorialRing,
        MultiplicativeMonoid, MultiplicativePartialMonoid, Ring,
    },
    util::{extended_euclid, try_inverse},
};
use std::{fmt, hash::Hash, marker};
use typenum::{IsGreater, NonZero, Unsigned, U1};
pub trait Radix = Unsigned + NonZero + Copy + Eq + Hash;

pub mod ideal;

/* # cyclic groups */

/**
cyclic group of order `Period`
*/
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct C<Period: Radix> {
    raw: u16, // we might want to generalize this type to include more than one possible storage type
    _period: marker::PhantomData<Period>,
}

/* # helper functions */

impl<Period: Radix> C<Period> {
    /**
    will return a number between 0 and `Period`-1
    */
    const fn raw(self) -> u16 {
        self.raw
    }

    fn naive_multiples(self) -> impl Iterator<Item = Self> + Clone {
        let self_raw = u16::from(self);
        (0..Period::U16)
            .filter(move |r| r.rem_euclid(self_raw) == 0)
            .map(Self::from)
    }

    pub fn naive_divisors(self) -> impl Iterator<Item = Self> + Clone {
        let self_raw = u16::from(self);
        (1..=Period::U16)
            .filter(move |&r| self_raw.rem_euclid(r) == 0)
            .map(Self::from)
    }
}

/* ## debug and display */

impl<Period: Radix> fmt::Debug for C<Period> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "C{}[{}]", Period::U16, self.raw)
    }
}

impl<Period: Radix> fmt::Display for C<Period> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.raw)
    }
}

/* ## send and sync */

unsafe impl<Period: Radix + Send> Send for C<Period> {}
unsafe impl<Period: Radix + Sync> Sync for C<Period> {}

/* ## algebraic structure */

/* ### demesne */

impl<Period: Radix> From<u16> for C<Period> {
    fn from(raw: u16) -> Self {
        Self {
            raw: raw.rem_euclid(Period::U16), // Period is known to be NonZero
            _period: marker::PhantomData,
        }
    }
}

impl<Period: Radix> From<C<Period>> for u16 {
    /**
    will return a number between 1 and `Period`
    */
    fn from(group: C<Period>) -> Self {
        match group.raw {
            0 => Period::U16,
            raw => raw,
        }
    }
}

impl<Period: Radix> Demesne for C<Period> {}

impl<Period: Radix> Enumerable for C<Period> {
    fn terms() -> impl Iterator<Item = Self> + Clone {
        (1..=Period::U16).map(Self::from)
    }

    fn cardinality() -> usize {
        Period::USIZE
    }
}

/* ### additive structure */

impl<Period: Radix> AdditivePartialMonoid for C<Period> {
    fn try_add(self, other: Self) -> Option<Self> {
        Some(self.add(other))
    }

    fn own_zero(&self) -> Self {
        Self::zero()
    }

    fn is_zero(&self) -> bool {
        self.raw == 0
    }

    fn is_negable(&self) -> bool {
        true
    }

    fn try_neg(self) -> Option<Self> {
        Some(self.neg())
    }
}

impl<Period: Radix> AdditiveMonoid for C<Period> {
    fn zero() -> Self {
        Self::from(0)
    }

    #[allow(
        clippy::expect_used,
        reason = "this should panic if parameters are illchosen"
    )]
    fn add(self, other: Self) -> Self {
        Self::from(
            self.raw
                .checked_add(other.raw)
                .expect("u16 should be bigger than `Period`-1 squared"),
        )
    }

    #[allow(
        clippy::expect_used,
        reason = "this should panic if parameters are illchosen"
    )]
    fn add_assign(&mut self, other: Self) {
        self.raw = self
            .raw
            .checked_add(other.raw)
            .expect("u16 should be bigger than `Period`-1 squared")
            .rem_euclid(Period::U16);
    }
}

impl<Period: Radix> AdditivePartialGroup for C<Period> {
    fn neg(self) -> Self {
        Self::from(Period::U16.saturating_sub(self.raw))
    }

    fn neg_inplace(&mut self) {
        // no need to rem,
        // since difference will be smaller than `Period`
        self.raw = Period::U16.saturating_sub(self.raw);
    }
}

impl<Period: Radix> AdditiveGroup for C<Period> {}

/* ### multiplicative structure */

impl<Period: Radix + IsGreater<U1>> MultiplicativePartialMonoid for C<Period> {
    fn try_mul(self, other: Self) -> Option<Self> {
        Some(self.mul(other))
    }

    fn own_one(&self) -> Self {
        Self::one()
    }

    fn is_one(&self) -> bool {
        self.raw == 1
    }

    fn is_invable(&self) -> bool {
        let (gcd, _x, _y) = extended_euclid(self.raw.into(), Period::I32);
        gcd == 1_i32
    }

    #[allow(clippy::expect_used, reason = "structural properties")]
    fn try_inv(self) -> Option<Self> {
        try_inverse(self.raw.into(), Period::I32).map(|inv| {
            Self::from(u16::try_from(inv).expect("modulo `Period` guarantees this will match"))
        })
    }
}

impl<Period: Radix + IsGreater<U1>> MultiplicativeMonoid for C<Period> {
    fn one() -> Self {
        Self::from(1)
    }

    #[allow(
        clippy::expect_used,
        reason = "this should panic if parameters are illchosen"
    )]
    fn mul(self, other: Self) -> Self {
        Self::from(
            self.raw
                .checked_mul(other.raw)
                .expect("u16 should be bigger than `Period`-1 squared"),
        )
    }

    #[allow(
        clippy::expect_used,
        reason = "this should panic if parameters are illchosen"
    )]
    fn mul_assign(&mut self, other: Self) {
        self.raw = self
            .raw
            .checked_mul(other.raw)
            .expect("u16 should be bigger than `Period`-1 squared")
            .rem_euclid(Period::U16);
    }
}

/* ### rings */

impl<Period: Radix + IsGreater<U1>> Ring for C<Period> {
    fn try_divide(self, r: Self) -> impl Iterator<Item = Self> + Clone {
        Self::terms().filter(move |&x| r.mul(x) == self)
    }

    fn is_divisor(&self, r: Self) -> bool {
        r.try_divide(*self).next().is_some()
    }

    fn divisors(self) -> impl Iterator<Item = Self> + Clone {
        Self::terms().filter(move |r| r.is_divisor(self))
    }
}

/* #### factorial ring */

#[derive(Clone)]
pub struct Factors<Period: Radix> {
    marble: C<Period>,
}

const SMALL_PRIMES: &[u16] = &[
    2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97,
];

impl<Period: Radix + IsGreater<U1>> Iterator for Factors<Period> {
    type Item = C<Period>;

    #[allow(
        clippy::expect_used,
        reason = "const list needs to be extended on panic"
    )]
    fn next(&mut self) -> Option<Self::Item> {
        match self.marble.is_invable() {
            true => None,
            false => {
                let divisor: u16 = *SMALL_PRIMES
                    .iter()
                    .find(|&&r| u16::from(self.marble).rem_euclid(r) == 0)
                    .expect("small primes list is too short");
                self.marble = Self::Item::from(u16::from(self.marble).div_euclid(divisor));
                Some(Self::Item::from(divisor))
            }
        }
    }
}

impl<Period: Radix + IsGreater<U1>> FactorialRing for C<Period> {
    fn factors(self) -> impl Iterator<Item = Self> + Clone {
        let (gcd, _, _) = Self::gcd(self, Self::zero());
        Factors::<Period> { marble: gcd }
    }
}

/* #### bezout ring */

impl<Period: Radix + IsGreater<U1>> BezoutRing for C<Period> {
    #[allow(clippy::panic, reason = "structural guarantees are violated on panic")]
    fn gcd(r: Self, s: Self) -> (Self, Self, Self) {
        let r_raw = u16::from(r);
        let s_raw = u16::from(s);
        let (gcd, x, y) = extended_euclid(r_raw.into(), s_raw.into());
        match (
            u16::try_from(gcd),
            u16::try_from(x.rem_euclid(Period::I32)),
            u16::try_from(y.rem_euclid(Period::I32)),
        ) {
            (Ok(gcd_u16), Ok(x_u16), Ok(y_u16)) => {
                (Self::from(gcd_u16), Self::from(x_u16), Self::from(y_u16))
            }
            _ => panic!("bezout coefficients were not found"),
        }
    }
}

// - - -

/* # tests */

#[cfg(test)]
mod test {
    use super::*;
    use typenum::{U3, U36, U6};

    /* # collection */

    #[test]
    fn creation() {
        assert_eq!(C::<U3>::from(1), C::<U3>::from(1));
        assert_eq!(C::<U3>::from(1), C::<U3>::from(4));
    }

    #[test]
    fn listing_elements() {
        let mut els = C::<U6>::terms();
        assert_eq!(els.next(), Some(C::<U6>::from(1)));
        assert_eq!(els.next(), Some(C::<U6>::from(2)));
        assert_eq!(els.next(), Some(C::<U6>::from(3)));
        assert_eq!(els.next(), Some(C::<U6>::from(4)));
        assert_eq!(els.next(), Some(C::<U6>::from(5)));
        assert_eq!(els.next(), Some(C::<U6>::from(0)));
        assert_eq!(els.next(), None);
    }

    /* # additive structure */

    #[test]
    fn addition() {
        assert_eq!(C::<U3>::from(1).add(C::<U3>::zero()), C::<U3>::from(1));
        assert_eq!(C::<U3>::from(1).add(C::<U3>::from(1)), C::<U3>::from(2));
        assert_eq!(C::<U3>::from(1).add(C::<U3>::from(2)), C::<U3>::from(0));
        assert_eq!(C::<U3>::from(1).add(C::<U3>::from(4)), C::<U3>::from(2));
    }

    #[test]
    fn negation() {
        assert_eq!(C::<U3>::from(0).neg(), C::<U3>::from(0));
        assert_eq!(C::<U3>::from(1).neg(), C::<U3>::from(2));
        assert_eq!(C::<U3>::from(2).neg(), C::<U3>::from(1));
    }

    /* # multiplicative structure */

    #[test]
    fn multiplication() {
        assert_eq!(C::<U6>::from(2).mul(C::<U6>::one()), C::<U6>::from(2));
        assert_eq!(C::<U6>::from(2).mul(C::<U6>::from(2)), C::<U6>::from(4));
        assert_eq!(C::<U6>::from(2).mul(C::<U6>::from(3)), C::<U6>::from(0));
        assert_eq!(C::<U6>::from(2).mul(C::<U6>::from(4)), C::<U6>::from(2));
    }

    #[test]
    fn inversion() {
        assert_eq!(C::<U6>::from(0).try_inv(), None);
        assert_eq!(C::<U6>::from(1).try_inv(), Some(C::<U6>::from(1)));
        assert_eq!(C::<U6>::from(2).try_inv(), None);
        assert_eq!(C::<U6>::from(3).try_inv(), None);
        assert_eq!(C::<U6>::from(4).try_inv(), None);
        assert_eq!(C::<U6>::from(5).try_inv(), Some(C::<U6>::from(5)));
    }

    /* # ring structure */

    #[test]
    fn divisoring() {
        assert!(C::<U6>::from(0).is_divisor(C::<U6>::from(0)));
        assert!(C::<U6>::from(2).is_divisor(C::<U6>::from(0)));
        assert!(C::<U6>::from(2).is_divisor(C::<U6>::from(4)));
        assert!(C::<U6>::from(3).is_divisor(C::<U6>::from(0)));
        assert!(C::<U6>::from(4).is_divisor(C::<U6>::from(2)));
        assert!(C::<U6>::from(4).is_divisor(C::<U6>::from(4)));
    }

    #[test]
    fn finding_naive_divisors() {
        assert_eq!(
            C::<U6>::from(0).naive_divisors().collect::<Vec<_>>(),
            [1, 2, 3, 0].map(C::<U6>::from).to_vec()
        );
        assert_eq!(
            C::<U6>::from(2).naive_divisors().collect::<Vec<_>>(),
            [1, 2].map(C::<U6>::from).to_vec()
        );
        assert_eq!(
            C::<U6>::from(4).naive_divisors().collect::<Vec<_>>(),
            [1, 2, 4].map(C::<U6>::from).to_vec()
        );
    }

    #[test]
    fn finding_true_divisors() {
        assert_eq!(
            C::<U6>::from(0).divisors().collect::<Vec<_>>(),
            [1, 2, 3, 4, 5, 0].map(C::<U6>::from).to_vec()
        );
        assert_eq!(
            C::<U6>::from(2).divisors().collect::<Vec<_>>(),
            [1, 2, 4, 5].map(C::<U6>::from).to_vec()
        );
        assert_eq!(
            C::<U6>::from(4).divisors().collect::<Vec<_>>(),
            [1, 2, 4, 5].map(C::<U6>::from).to_vec()
        );
    }

    /* # factorial ring */

    #[test]
    fn finding_factors() {
        assert_eq!(
            C::<U6>::from(0).factors().collect::<Vec<_>>(),
            [2, 3].map(C::<U6>::from).to_vec(),
        );
        assert_eq!(C::<U6>::from(5).factors().collect::<Vec<_>>(), Vec::new(),);
        assert_eq!(
            C::<U36>::from(0).factors().collect::<Vec<_>>(),
            [2, 2, 3, 3].map(C::<U36>::from).to_vec(),
        );
        assert_eq!(
            C::<U36>::from(20).factors().collect::<Vec<_>>(),
            [2, 2].map(C::<U36>::from).to_vec(),
        );
        assert_eq!(C::<U36>::from(35).factors().collect::<Vec<_>>(), Vec::new(),);
    }

    #[test]
    fn finding_power_factors() {
        assert_eq!(
            C::<U6>::from(0).power_factors().collect::<Vec<_>>(),
            [2, 3].map(C::<U6>::from).to_vec(),
        );
        assert_eq!(
            C::<U36>::from(0).power_factors().collect::<Vec<_>>(),
            [4, 9].map(C::<U36>::from).to_vec(),
        );
        assert_eq!(
            C::<U36>::from(20).power_factors().collect::<Vec<_>>(),
            [4].map(C::<U36>::from).to_vec(),
        );
    }

    /* # bezout ring */

    #[test]
    fn finding_greatest_common_divisor() {
        assert_eq!(
            C::<U6>::gcd(C::from(2), C::from(3)),
            (C::from(1), C::from(5), C::from(1))
        );
        assert_eq!(
            C::<U6>::gcd(C::from(2), C::from(4)),
            (C::from(2), C::from(1), C::from(0))
        );
        assert_eq!(
            C::<U36>::gcd(C::from(18), C::from(12)),
            (C::from(6), C::from(1), C::from(35))
        );
    }
}
