use crate::{rmodule::torsion::Coprime, util::number::divisors, Int};
use gcd::Gcd;
use std::{
    array::from_fn,
    cmp::Ordering,
    ops::{Add, Mul, Neg, Rem},
};
use typenum::{IsEqual, Mod, NonZero, Unsigned, U0};

pub type Zahl = Int;
pub trait Radix = Unsigned + NonZero + Copy + Eq + Send + Sync;
pub trait SuperRing<RC: Unsigned + NonZero + Eq + Send + Sync> =
    Ring<RC> + Ord + Rem<Output = Self> + Coprime;

/* # structure */

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fin<Card: Radix> {
    zahl: Zahl,
    _radix: std::marker::PhantomData<Card>,
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

pub trait Set<Card: Radix>: PartialEq + Eq + Clone + Copy + Send + Sync {
    fn get(&self) -> Zahl;
    fn new(z: Zahl) -> Self;

    fn elements() -> [Self; Card::USIZE] {
        from_fn::<Self, { Card::USIZE }, _>(|j| {
            Self::new(j.try_into().expect("we're gonna need a bigger int"))
        })
    }
}

impl<Card: Radix> Set<Card> for Fin<Card> {
    fn get(&self) -> Zahl {
        self.zahl
    }

    fn new(z: Zahl) -> Self {
        // this will never panic, since Radix is known to be NonZero at compile time
        Self {
            zahl: z % Card::U8,
            _radix: std::marker::PhantomData,
        }
    }
}

/* ## ring */

// i could try to enforce non zero CARD for rings
// but this would require some advanced types that would have to be dragged around
// technically they would be evaluated at comiple time
// but they would take up memory
pub trait Ring<Card: Radix>:
    Neg<Output = Self> + Add<Self, Output = Self> + Mul<Self, Output = Self> + Set<Card>
{
    fn zero() -> Self;
    fn one() -> Self;
    fn is_zero(&self) -> bool;
    fn is_one(&self) -> bool;
    fn ideals() -> impl Iterator<Item = Self>;
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
        Self::Output::new(Card::U8 - self.zahl)
    }
}

impl<Card: Radix> Ring<Card> for Fin<Card> {
    fn zero() -> Self {
        Self::new(0)
    }

    fn one() -> Self {
        Self::new(1)
    }

    fn is_zero(&self) -> bool {
        self.zahl == 0
    }

    fn is_one(&self) -> bool {
        self.zahl == 1
    }

    fn ideals() -> impl Iterator<Item = Self> {
        divisors(Card::U8).into_iter().map(Self::new)
    }

    fn subideals(&self) -> impl Iterator<Item = Self> {
        divisors(self.zahl).into_iter().map(Self::new)
    }
}

impl<Small: Radix, Large: Radix> Rem<Fin<Small>> for Fin<Large>
where
    Small: Radix,
    Large: Radix + Rem<Small>,
    Mod<Large, Small>: IsEqual<U0>,
{
    type Output = Fin<Small>;

    /**
    this is unusual, since we are modding by the const
    NOT by the actual given value.
    i could write a separate trait just for that,
    but again, i am too lazy, and this interface works well
    */
    fn rem(self, _other: Fin<Small>) -> Self::Output {
        Self::Output::new(self.zahl)
    }
}

impl<Card: Radix> PartialOrd for Fin<Card> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.zahl.partial_cmp(&other.zahl)
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

impl<Card: Radix> Coprime for Fin<Card> {
    fn one() -> Self {
        <Self as Ring<Card>>::one()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use typenum::{U2, U3, U4, U6, U8};

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
        assert_eq!(Fin::<U6>::new(5) % Fin::<U3>::new(0), Fin::<U3>::new(2));
        assert_eq!(Fin::<U8>::new(7) % Fin::<U4>::new(0), Fin::<U4>::new(3));
        assert_eq!(Fin::<U8>::new(7) % Fin::<U2>::new(0), Fin::<U2>::new(1));
    }
}
