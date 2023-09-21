use crate::{
    category::object::{
        Concrete as ConcreteObject, Enumerable as EnumerableObject, Object as CatObject,
    },
    ralg::{
        cgroup::{Radix, C},
        module::{Module, ModuleObject},
        ring::{
            ideal::{Ideal, Principal as PrincipalIdeal},
            AdditivePartialGroup, AdditivePartialMonoid, Bezout as BezoutRing, Demesne, Ring,
        },
    },
};
use dedup::noncon::{DedupNonConAdapter, DedupNonConByAdapter};
use std::{cmp, fmt};
use typenum::{IsGreater, U1};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Quotient<R: Ring, I: Ideal<Parent = R>, E> {
    /// generator of the ideal that the ring is divided by
    pub ideal: I,
    pub element: E,
}

/* ## helper functions */

impl<R: Ring, I: Ideal<Parent = R>, E> Quotient<R, I, E> {
    pub const fn new(ideal: I, element: E) -> Self {
        Self { ideal, element }
    }
}

/* ## order */

impl<R: Ring, I: Ideal<Parent = R> + PartialOrd, E: PartialEq> PartialOrd for Quotient<R, I, E> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.ideal.partial_cmp(&other.ideal)
    }
}

impl<R: Ring, I: Ideal<Parent = R> + Ord, E: Eq> Ord for Quotient<R, I, E> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.ideal.cmp(&other.ideal)
    }
}

/* # objects */

#[allow(type_alias_bounds, reason = "waiting on feature `lazy_type_alias`")]
pub type Object<R: Ring, I: Ideal<Parent = R>> = Quotient<R, I, ()>;

/* ## helper functions */

impl<R: Ring, I: Ideal<Parent = R>> Object<R, I> {
    pub fn attach_element(&self, r: R) -> Element<R, I> {
        Element {
            ideal: self.ideal.clone(),
            element: r,
        }
    }
}

/* ## conversions */

impl<R: Ring, I: Ideal<Parent = R>> From<I> for Object<R, I> {
    fn from(ideal: I) -> Self {
        Self { ideal, element: () }
    }
}

impl<R: Ring + From<u16>, I: Ideal<Parent = R>> From<u16> for Object<R, I> {
    fn from(n: u16) -> Self {
        Self::from(I::principal(R::from(n)))
    }
}

/* ## debug and display */

impl<R: Ring, I: Ideal<Parent = R> + fmt::Debug> fmt::Debug for Object<R, I> {
    default fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.ideal.is_full() {
            write!(f, "0")
        } else if self.ideal.is_trivial() {
            write!(f, "R",)
        } else {
            write!(f, "R/{:?}", self.ideal)
        }
    }
}

impl<Period: Radix + IsGreater<U1>, I: Ideal<Parent = C<Period>> + fmt::Debug> fmt::Debug
    for Object<C<Period>, I>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.ideal.is_full() {
            write!(f, "0")
        } else if self.ideal.is_trivial() {
            write!(f, "C{}", Period::U16)
        } else {
            write!(f, "C{}/{:?}", Period::U16, self.ideal)
        }
    }
}

impl<R: Ring, I: Ideal<Parent = R> + fmt::Display> fmt::Display for Object<R, I> {
    default fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "R/{}", self.ideal)
    }
}

impl<Period: Radix + IsGreater<U1>, I: Ideal<Parent = C<Period>> + fmt::Display> fmt::Display
    for Object<C<Period>, I>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "C{}/{}", Period::U16, self.ideal)
    }
}

/* ### module object */

impl<R: Ring, I: Ideal<Parent = R>> CatObject for Object<R, I> {}

impl<R: Ring + Clone, I: Ideal<Parent = R>> EnumerableObject for Object<R, I> {
    /**
    this generates all quotients of the ring,
    although rather slowly.

    moreover, it only divides by principal ideals so far,
    but that can be fixed by iterating over all tuples of elements of a ring,
    instead of only oce by elements.
    */
    fn all() -> impl Iterator<Item = Self> + Clone {
        R::terms()
            .map(|r| Self::from(I::principal(r)))
            .dedup_non_con()
    }
}

impl<R: Ring + Copy, I: Ideal<Parent = R>> ConcreteObject for Object<R, I> {
    type Element = Element<R, I>;

    fn elements(&self) -> impl Iterator<Item = Self::Element> + Clone + '_ {
        R::terms()
            .dedup_non_con_by(|r, s| self.ideal.contains(r.sub(*s)))
            .map(|r| Self::Element::new(self.ideal.clone(), r))
    }

    fn is_element(&self, el: &Self::Element) -> bool {
        self.ideal == el.ideal
    }
}

impl<R: Ring + Copy, I: Ideal<Parent = R>> ModuleObject<R> for Object<R, I> {
    fn is_trivial(&self) -> bool {
        self.ideal.is_full()
    }

    fn trivial() -> Self {
        Self::from(I::principal(R::one()))
    }

    fn zero(&self) -> Self::Element {
        self.attach_element(R::zero())
    }
}

impl<R: BezoutRing + Copy, I: PrincipalIdeal<Parent = R>> Object<R, I> {
    #[allow(clippy::expect_used, reason = "structural guarantees")]
    pub fn mul(self, r: R) -> Self {
        let generator = self.ideal.generator();
        Self::from(I::principal(
            generator
                .try_divide(R::gcd(generator, r).0)
                .next()
                .expect("gcd should divide"),
        ))
    }

    pub fn try_div(self, other: Self) -> Option<Self> {
        let self_generator = self.ideal.generator();
        let other_generator = other.ideal.generator();
        other_generator
            .is_divisor(self_generator)
            .then_some(Self::from(I::principal(
                self_generator.try_divide(other_generator).next()?,
            )))
    }
}

/* # elements */

#[allow(type_alias_bounds, reason = "waiting on feature `lazy_type_alias`")]
pub type Element<R: Ring, I: Ideal<Parent = R>> = Quotient<R, I, R>;

/* ## helper functions */

impl<R: Ring + Copy, I: Ideal<Parent = R>> Element<R, I> {
    /// compares two elements for equivalence over ideal
    /// returns `None` if elements are incomparable
    pub fn is_equal(&self, other: &Self) -> Option<bool> {
        (self.ideal == other.ideal).then_some(self.ideal.contains(self.element.sub(other.element)))
    }

    pub fn object(&self) -> Object<R, I> {
        Object::from(self.ideal.clone())
    }
}

/* ## debug and display */

impl<Period: Radix + IsGreater<U1>, I: Ideal<Parent = C<Period>> + fmt::Debug> fmt::Debug
    for Element<C<Period>, I>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "C{}/{:?}[{:?}]", Period::U16, self.ideal, self.element)
    }
}

impl<Period: Radix + IsGreater<U1>, I: Ideal<Parent = C<Period>> + fmt::Display> fmt::Display
    for Element<C<Period>, I>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "C{}/{}[{}]", Period::U16, self.ideal, self.element)
    }
}

/* ## algebraic structure */

/* ### demesne */

impl<R: Ring, I: Ideal<Parent = R>> Demesne for Element<R, I> {}

/* ### additive structure */

impl<R: Ring + Copy, I: Ideal<Parent = R>> AdditivePartialMonoid for Element<R, I> {
    fn try_add(self, other: Self) -> Option<Self> {
        (self.ideal == other.ideal)
            .then_some(Self::new(self.ideal, self.element.add(other.element)))
    }

    fn own_zero(&self) -> Self {
        Self::new(self.ideal.clone(), R::zero())
    }

    fn is_zero(&self) -> bool {
        self.ideal.contains(self.element)
    }

    fn is_negable(&self) -> bool {
        true
    }

    fn try_neg(self) -> Option<Self> {
        Some(self.neg())
    }
}

impl<R: Ring + Copy, I: Ideal<Parent = R>> AdditivePartialGroup for Element<R, I> {
    fn neg(self) -> Self {
        Self::new(self.ideal, self.element.neg())
    }

    fn neg_inplace(&mut self) {
        self.element.neg_inplace();
    }
}

/* ### module structure */

impl<R: Ring + Copy, I: Ideal<Parent = R>> Module<R> for Element<R, I> {
    fn mul(self, r: R) -> Self {
        Self::new(self.ideal, self.element.mul(r))
    }

    fn mul_assign(&mut self, r: R) {
        self.element.mul_assign(r);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ralg::cgroup::{ideal::CIdeal, C};
    use typenum::{U36, U6, U7};

    /* # objects */

    #[test]
    fn finding_all_quotient_objects() {
        type N = U6;
        let mut quotients = Object::<C<N>, CIdeal<N>>::all();
        assert_eq!(quotients.next(), Some(Object::from(1)));
        assert_eq!(quotients.next(), Some(Object::from(2)));
        assert_eq!(quotients.next(), Some(Object::from(3)));
        assert_eq!(quotients.next(), Some(Object::from(0)));
        assert_eq!(quotients.next(), None,);
    }

    #[test]
    fn multiplying_object_by_ring_element() {
        type N = U36;
        let z1 = Object::<C<N>, CIdeal<N>>::from(1);
        let z2 = Object::<C<N>, CIdeal<N>>::from(2);
        let z3 = Object::<C<N>, CIdeal<N>>::from(3);
        let z4 = Object::<C<N>, CIdeal<N>>::from(4);
        let z6 = Object::<C<N>, CIdeal<N>>::from(6);

        assert_eq!(z3.mul(C::from(0)), z1);
        assert_eq!(z3.mul(C::from(1)), z3);
        assert_eq!(z3.mul(C::from(2)), z3);

        assert_eq!(z4.mul(C::from(0)), z1);
        assert_eq!(z4.mul(C::from(1)), z4);
        assert_eq!(z4.mul(C::from(2)), z2);
        assert_eq!(z4.mul(C::from(3)), z4);

        assert_eq!(z6.mul(C::from(0)), z1);
        assert_eq!(z6.mul(C::from(1)), z6);
        assert_eq!(z6.mul(C::from(2)), z3);
        assert_eq!(z6.mul(C::from(3)), z2);
        assert_eq!(z6.mul(C::from(4)), z3);
        assert_eq!(z6.mul(C::from(5)), z6);
    }

    #[test]
    #[allow(clippy::shadow_unrelated, reason = "useful in test")]
    fn dividing_objects() {
        type N = U36;
        type M = U7;

        let z1 = Object::<C<N>, CIdeal<N>>::from(1);
        let z2 = Object::<C<N>, CIdeal<N>>::from(2);
        let z3 = Object::<C<N>, CIdeal<N>>::from(3);
        let z4 = Object::<C<N>, CIdeal<N>>::from(4);
        let z6 = Object::<C<N>, CIdeal<N>>::from(6);

        assert_eq!(z4.try_div(z1), Some(z4));
        assert_eq!(z4.try_div(z2), Some(z2));
        assert_eq!(z4.try_div(z3), None);
        assert_eq!(z4.try_div(z4), Some(z1));

        assert_eq!(z6.try_div(z1), Some(z6));
        assert_eq!(z6.try_div(z2), Some(z3));
        assert_eq!(z6.try_div(z3), Some(z2));
        assert_eq!(z6.try_div(z4), None);
        assert_eq!(z6.try_div(z6), Some(z1));

        let z1 = Object::<C<M>, CIdeal<M>>::from(1);
        let z7 = Object::<C<M>, CIdeal<M>>::from(0);
        assert_eq!(z7.try_div(z1), Some(z7));
        assert_eq!(z7.try_div(z7), Some(z1));
    }

    /* # elements */

    #[test]
    #[allow(clippy::shadow_unrelated, reason = "useful in test")]
    fn finding_all_elements_of_quotient() {
        type N = U6;

        let ideal = CIdeal::<N>::principal(C::from(0));
        let quotient = Object::from(ideal);
        let mut elements = quotient.elements();
        assert_eq!(elements.next(), Some(Element::new(ideal, C::from(1))));
        assert_eq!(elements.next(), Some(Element::new(ideal, C::from(2))));
        assert_eq!(elements.next(), Some(Element::new(ideal, C::from(3))));
        assert_eq!(elements.next(), Some(Element::new(ideal, C::from(4))));
        assert_eq!(elements.next(), Some(Element::new(ideal, C::from(5))));
        assert_eq!(elements.next(), Some(Element::new(ideal, C::from(0))));
        assert_eq!(elements.next(), None);

        let ideal = CIdeal::<N>::principal(C::from(1));
        let quotient = Object::from(ideal);
        let mut elements = quotient.elements();
        assert_eq!(elements.next(), Some(Element::new(ideal, C::from(1))));
        assert_eq!(elements.next(), None);

        let ideal = CIdeal::<N>::principal(C::from(2));
        let quotient = Object::from(ideal);
        let mut elements = quotient.elements();
        assert_eq!(elements.next(), Some(Element::new(ideal, C::from(1))));
        assert_eq!(elements.next(), Some(Element::new(ideal, C::from(2))));
        assert_eq!(elements.next(), None);

        let ideal = CIdeal::<N>::principal(C::from(3));
        let quotient = Object::from(ideal);
        let mut elements = quotient.elements();
        assert_eq!(elements.next(), Some(Element::new(ideal, C::from(1))));
        assert_eq!(elements.next(), Some(Element::new(ideal, C::from(2))));
        assert_eq!(elements.next(), Some(Element::new(ideal, C::from(3))));
        assert_eq!(elements.next(), None);
    }

    #[test]
    #[allow(clippy::shadow_unrelated, reason = "useful in test")]
    fn equality_of_elements_in_quotient() {
        type N = U6;

        let ideal = CIdeal::<N>::principal(C::from(3));
        assert_eq!(
            Element::new(ideal, C::from(1)).is_equal(&Element::new(ideal, C::from(4))),
            Some(true)
        );
        assert_eq!(
            Element::new(ideal, C::from(2)).is_equal(&Element::new(ideal, C::from(5))),
            Some(true)
        );
        assert_eq!(
            Element::new(ideal, C::from(1)).is_equal(&Element::new(ideal, C::from(5))),
            Some(false)
        );
        assert!(Element::new(ideal, C::from(1))
            .is_equal(&Element::new(
                CIdeal::<N>::principal(C::from(2)),
                C::from(1)
            ))
            .is_none());
    }
}
