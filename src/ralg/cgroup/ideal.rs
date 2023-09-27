use crate::ralg::{
    cgroup::{Radix, C},
    ring::{
        ideal::{Ideal, Principal},
        AdditiveMonoid, AdditivePartialMonoid, Bezout as BezoutRing, MultiplicativePartialMonoid,
    },
};
use std::{cmp, fmt};
use typenum::{IsGreater, U1};

/* # ideals */

/**
ideal of a cyclic group of order `Period`
*/
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct CIdeal<Period: Radix> {
    generator: C<Period>,
}

/* ## debug and display */

impl<Period: Radix> fmt::Debug for CIdeal<Period> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:?})", self.generator)
    }
}

impl<Period: Radix> fmt::Display for CIdeal<Period> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({})", self.generator)
    }
}

/* ## order */
// i do not like this part

impl<Period: Radix + IsGreater<U1>> PartialOrd for CIdeal<Period> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<Period: Radix + IsGreater<U1>> Ord for CIdeal<Period> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        u16::from(self.generator).cmp(&u16::from(other.generator))
    }
}

/* ## ideal structure */

impl<Period: Radix + IsGreater<U1>> Ideal for CIdeal<Period> {
    type Parent = C<Period>;

    fn principal(r: Self::Parent) -> Self {
        let (gcd, _, _) = Self::Parent::gcd(r, Self::Parent::zero());
        Self { generator: gcd }
    }

    fn generators(self) -> impl Iterator<Item = Self::Parent> {
        [self.generator].into_iter()
    }

    fn contains(&self, r: Self::Parent) -> bool {
        u16::from(r).rem_euclid(u16::from(self.generator)) == 0
    }

    fn is_trivial(&self) -> bool {
        self.generator.is_zero()
    }

    fn is_full(&self) -> bool {
        self.generator.is_one()
    }
}

impl<Period: Radix + IsGreater<U1>> Principal for CIdeal<Period> {
    fn generator(self) -> Self::Parent {
        self.generator
    }
}

// - - -

/* # tests */

#[cfg(test)]
mod test {
    use super::*;
    use typenum::U6;

    /* # ideals */

    #[test]
    fn finding_ideals() {
        assert_eq!(
            CIdeal::<U6>::principal(C::<U6>::from(4)),
            CIdeal::<U6> {
                generator: C::<U6>::from(2)
            },
            "divisible by two"
        );
        assert_eq!(
            CIdeal::<U6>::principal(C::<U6>::from(5)),
            CIdeal::<U6> {
                generator: C::<U6>::from(1)
            },
            "any unit gives full ideal"
        );
    }

    #[test]
    fn elements_in_ideals() {
        let ideal_trivial = CIdeal::<U6>::principal(C::<U6>::from(0));
        assert!(ideal_trivial.contains(C::from(0)));
        assert!(!ideal_trivial.contains(C::from(1)));
        assert!(!ideal_trivial.contains(C::from(2)));
        assert!(!ideal_trivial.contains(C::from(3)));
        assert!(!ideal_trivial.contains(C::from(4)));
        assert!(!ideal_trivial.contains(C::from(5)));

        let ideal_full = CIdeal::<U6>::principal(C::<U6>::from(1));
        assert!(ideal_full.contains(C::from(0)));
        assert!(ideal_full.contains(C::from(1)));
        assert!(ideal_full.contains(C::from(2)));
        assert!(ideal_full.contains(C::from(3)));
        assert!(ideal_full.contains(C::from(4)));
        assert!(ideal_full.contains(C::from(5)));

        let ideal_twos = CIdeal::<U6>::principal(C::<U6>::from(2));
        assert!(ideal_twos.contains(C::from(0)));
        assert!(!ideal_twos.contains(C::from(1)));
        assert!(ideal_twos.contains(C::from(2)));
        assert!(!ideal_twos.contains(C::from(3)));
        assert!(ideal_twos.contains(C::from(4)));
        assert!(!ideal_twos.contains(C::from(5)));

        let ideal_threes = CIdeal::<U6>::principal(C::<U6>::from(3));
        assert!(ideal_threes.contains(C::from(0)));
        assert!(!ideal_threes.contains(C::from(1)));
        assert!(!ideal_threes.contains(C::from(2)));
        assert!(ideal_threes.contains(C::from(3)));
        assert!(!ideal_threes.contains(C::from(4)));
        assert!(!ideal_threes.contains(C::from(5)));
    }
}
