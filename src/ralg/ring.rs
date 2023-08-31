/* # collections */

/**
an element of an algebraic structure,
which can bo constructed from a natural number.
we are excluding empty collections.
*/
// possibly we might want to exchange u16 for own type used solely to index stuff
pub trait Demesne: PartialEq + Eq + From<u16> + Into<u16> {
    fn elements() -> impl Iterator<Item = Self> + Clone;
}

/* # additive structure */

pub trait AdditiveMonoid: Demesne {
    fn zero() -> Self;
    fn add(self, other: Self) -> Self;
    fn add_assign(&mut self, other: Self);

    fn is_zero(&self) -> bool;
    fn is_negable(&self) -> bool;
    fn try_neg(self) -> Option<Self>;

    fn negable_elements() -> impl Iterator<Item = Self> + Clone {
        Self::elements().filter_map(Self::try_neg)
    }

    fn try_sub(self, other: Self) -> Option<Self> {
        other.try_neg().map(|other_neg| self.add(other_neg))
    }
}

pub trait AdditiveGroup: AdditiveMonoid {
    fn neg(self) -> Self;
    fn neg_inplace(&mut self);

    fn sub(self, other: Self) -> Self {
        self.add(other.neg())
    }

    fn sub_assign(&mut self, other: Self) {
        self.add_assign(other.neg());
    }
}

/* # multiplicative structure */

pub trait MultiplicativeMonoid: Demesne {
    fn one() -> Self;
    fn mul(self, other: Self) -> Self;
    fn mul_assign(&mut self, other: Self);

    fn is_one(&self) -> bool;
    fn is_invable(&self) -> bool;
    fn try_inv(self) -> Option<Self>;

    fn invable_elements() -> impl Iterator<Item = Self> + Clone {
        Self::elements().filter_map(Self::try_inv)
    }

    fn try_div(self, other: Self) -> Option<Self> {
        other.try_inv().map(|other_inv| self.mul(other_inv))
    }
}

/* # rings */

pub trait Ring: AdditiveGroup + MultiplicativeMonoid {
    /**
    attempts to find an element x,
    such that self * x = r
    */
    fn try_divisor(self, r: Self) -> impl Iterator<Item = Self> + Clone;
    fn is_divisor(&self, r: Self) -> bool;
    fn divisors(self) -> impl Iterator<Item = Self> + Clone;

    fn principal_ideal(self) -> impl Iterator<Item = Self> + Clone;
    fn ideal(r: Self, s: Self) -> impl Iterator<Item = Self> + Clone;
}

// i would love to have this impl here,
// but the feature `return_position_impl_trait_in_trait`
// does not allow specialisation here.
// i prefer returning impl Iterators to having this implementation,
// that does not get used anywhere anyway.
/*
use itertools::Itertools;

impl<R> Ring for R
where
    R: Copy + AdditiveGroup + MultiplicativeMonoid,
{
    default fn try_divisor(self, r: Self) -> Option<Self> {
        Self::elements().find(|&x| self.mul(x) == r)
    }

    default fn is_divisor(&self, r: Self) -> bool {
        self.try_divisor(r).is_some()
    }

    default fn divisors(self) -> impl Iterator<Item = Self> + Clone {
        Self::elements().filter(move |r| r.is_divisor(self))
    }

    default fn principal_ideal(self) -> impl Iterator<Item = Self> + Clone {
        Self::elements()
            .map(|x| self.mul(x))
            .sorted_by_key(|&r| <Self as Into<u16>>::into(r))
            .dedup()
    }

    default fn ideal(r: Self, s: Self) -> impl Iterator<Item = Self> + Clone {
        Self::elements()
            .cartesian_product(Self::elements())
            .map(|(x, y)| r.mul(x).add(s.mul(y)))
            .sorted_by_key(|&x| <Self as Into<u16>>::into(x))
            .dedup()
    }
}
*/

#[allow(
    clippy::module_name_repetitions,
    reason = "the module name will change"
)]
pub trait FactorialRing: Ring + Copy {
    fn factors(self) -> impl Iterator<Item = Self> + Clone;

    fn power_factors(self) -> impl Iterator<Item = Self> + Clone {
        let f: Vec<_> = self.factors().collect();
        f.group_by(|x, y| x == y)
            .map(|group| {
                group
                    .iter()
                    .copied()
                    .reduce(Self::mul)
                    .unwrap_or_else(Self::one)
            })
            .collect::<Vec<_>>()
            .into_iter()
    }
}

#[allow(
    clippy::module_name_repetitions,
    reason = "the module name will change"
)]
pub trait BezoutRing: Ring {
    /**
    returns (g,x,y) such that g = rx + sy
    */
    fn gcd(r: Self, s: Self) -> (Self, Self, Self);
}
