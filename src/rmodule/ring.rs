use crate::{rmodule::torsion::Factorisable, util::number::divisors, Int};
use gcd::Gcd;
use std::{
    array::from_fn,
    cmp::Ordering,
    fmt,
    hash::Hash,
    ops::{Add, Mul, Neg, Rem},
};
use typenum::{NonZero, Unsigned};

pub type Zahl = Int;
pub trait Radix = Unsigned + NonZero + Copy + Eq + Send + Sync;
pub trait SuperRing = Ring + Ord + Rem<Output = Self> + Factorisable + Gcd + Into<usize> + Hash;

/* # structure */

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Fin<Card: Radix> {
    zahl: Zahl,
    _radix: std::marker::PhantomData<Card>,
}

impl<Card: Radix> fmt::Debug for Fin<Card> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Z{}({})",
            Card::U16,
            if self.zahl == Card::U16 { 0 } else { self.zahl }
        )
    }
}

/*
TODO product rings
this could be done in at least two ways:

firstly, create a pair
```rust
struct Pair<L:Ring, R:Ring> {}
```
and imply `Ring` for that pair.
this is easy and will work within current constraints.
however, it has a non derivable associator and explodes in complexity for higher dimensions

secondly, impl `Ring` for `CoeffTree`.
this is more difficult, will require advanced trickery to satisfy trait constraits,
but is immediately associative and scales well.
*/

/* # traits */

/* ## set */

pub trait Set: PartialEq + Eq + Clone + Copy + Send + Sync {
    type Card: Radix;

    fn get(&self) -> Zahl;
    fn new(z: Zahl) -> Self;

    fn elements() -> [Self; Self::Card::USIZE] {
        from_fn::<Self, { Self::Card::USIZE }, _>(|j| {
            Self::new(j.try_into().expect("we're gonna need a bigger int"))
        })
    }
}

impl<Card: Radix> Set for Fin<Card> {
    type Card = Card;

    fn get(&self) -> Zahl {
        self.zahl
    }

    fn new(z: Zahl) -> Self {
        // this will never panic, since Radix is known to be NonZero at compile time
        Self {
            zahl: match z % Card::U16 {
                0 => Card::U16,
                n => n,
            },
            _radix: std::marker::PhantomData,
        }
    }
}

/* ## ring */

pub trait Ring:
    Neg<Output = Self> + Add<Self, Output = Self> + Mul<Self, Output = Self> + Set
{
    fn zero() -> Self;
    fn one() -> Self;
    fn is_zero(&self) -> bool;
    fn is_one(&self) -> bool;
    fn divide_by(&self, other: &Self) -> Self;
    fn ideals() -> impl Iterator<Item = Self> + Clone;
    fn subideals(&self) -> impl Iterator<Item = Self>;
}

impl<Card: Radix> Add<Self> for Fin<Card> {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self::Output::new(self.zahl + other.zahl)
    }
}

impl<Card: Radix> Mul<Self> for Fin<Card> {
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        Self::Output::new(self.zahl * other.zahl)
    }
}

impl<Card: Radix> Neg for Fin<Card> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        // this will never overflow
        Self::Output::new(Card::U16 - self.zahl)
    }
}

impl<Card: Radix> Ring for Fin<Card> {
    fn zero() -> Self {
        Self::new(0)
    }

    fn one() -> Self {
        Self::new(1)
    }

    fn is_zero(&self) -> bool {
        self.zahl == Card::U16
    }

    fn is_one(&self) -> bool {
        self.zahl == 1
    }

    fn divide_by(&self, other: &Self) -> Self {
        match other.zahl {
            0 => Self::zero(),
            r => match self.zahl % r {
                0 => Self::new(self.zahl / r),
                _ => *self,
            },
        }
    }

    fn ideals() -> impl Iterator<Item = Self> + Clone {
        divisors(Card::U16).into_iter().map(Self::new)
    }

    fn subideals(&self) -> impl Iterator<Item = Self> {
        divisors(self.zahl).into_iter().map(Self::new)
    }
}

impl<Card: Radix> Rem for Fin<Card> {
    type Output = Fin<Card>;

    fn rem(self, other: Fin<Card>) -> Self::Output {
        Self::Output::new(self.zahl % other.zahl)
    }
}

impl<Card: Radix> PartialOrd for Fin<Card> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<Card: Radix> Ord for Fin<Card> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.zahl.cmp(&other.zahl)
    }
}

impl<Card: Radix> Gcd for Fin<Card> {
    fn gcd(self, other: Self) -> Self {
        Self::new(self.zahl.gcd(other.zahl))
    }

    fn gcd_binary(self, other: Self) -> Self {
        Self::new(self.zahl.gcd_binary(other.zahl))
    }

    fn gcd_euclid(self, other: Self) -> Self {
        Self::new(self.zahl.gcd_euclid(other.zahl))
    }
}

impl<Card: Radix> Into<usize> for Fin<Card> {
    fn into(self) -> usize {
        self.get() as usize
    }
}

/// this should be enough for our needs
const SMALL_PRIMES: &[Zahl] = &[2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31];

impl<Card: Radix + std::fmt::Debug> Factorisable for Fin<Card> {
    fn primes() -> Vec<Self> {
        SMALL_PRIMES
            .iter()
            .filter(|&&p| p <= Card::U16)
            .map(|p| Self::new(*p))
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use typenum::{U3, U6, U8};

    #[test]
    fn finset_negation() {
        assert_eq!(Fin::<U3>::new(1) + -Fin::<U3>::new(1), Fin::<U3>::new(0));
        assert_eq!(Fin::<U3>::new(2) + -Fin::<U3>::new(2), Fin::<U3>::new(0));
    }

    #[test]
    fn finset_addition() {
        assert_eq!(Fin::<U3>::new(1) + Fin::<U3>::new(1), Fin::<U3>::new(2));
        assert_eq!(Fin::<U3>::new(1) + Fin::<U3>::new(2), Fin::<U3>::new(0));
        assert_eq!(Fin::<U3>::new(1) + Fin::<U3>::new(5), Fin::<U3>::new(0));
    }

    #[test]
    fn finset_multiplication() {
        assert_eq!(Fin::<U3>::new(1) * Fin::<U3>::new(1), Fin::<U3>::new(1));
        assert_eq!(Fin::<U3>::new(1) * Fin::<U3>::new(2), Fin::<U3>::new(2));
        assert_eq!(Fin::<U3>::new(1) * Fin::<U3>::new(5), Fin::<U3>::new(2));
    }

    #[test]
    fn finset_remainder() {
        assert_eq!(Fin::<U6>::new(5) % Fin::<U6>::new(3), Fin::<U6>::new(2));
        assert_eq!(Fin::<U8>::new(7) % Fin::<U8>::new(4), Fin::<U8>::new(3));
        assert_eq!(Fin::<U8>::new(7) % Fin::<U8>::new(2), Fin::<U8>::new(1));
    }
}
