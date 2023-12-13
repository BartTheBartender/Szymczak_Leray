use crate::{
    category::{
        object::{
            Concrete as ConcreteObject, Duplicable as DuplicableObject,
            Enumerable as EnumerableObject, Object as CatObject,
            PartiallyEnumerable as PartiallyEnumerableObject,
        },
        PrettyName,
    },
    ralg::{
        cgroup::{ideal::CIdeal, Radix, C},
        matrix::Matrix,
        module::{
            canon::{element::Element, mark::Mark, MarkTree},
            direct::Object as DirectModule,
            map::CanonToCanon,
            quotient::Object as QuotientObject,
            ModuleObject,
        },
        ring::{
            ideal::{Ideal, Principal as PrincipalIdeal},
            Factorial as FactorialRing, MultiplicativeMonoid, MultiplicativePartialMonoid, Ring,
        },
    },
};
use itertools::Itertools;
use std::{collections::BTreeSet, fmt, sync::Arc};
use typenum::{IsGreater, Unsigned, U1};

/* # torsion coefficients object */

/**
this is structurally guaranteed to be not only sorted (descending),
but also that every element is either a prime or a power of a prime.
this is used in the representation of a module
as a product of quotients of the underlying ring.
*/
#[allow(type_alias_bounds, reason = "waiting on feature `lazy_type_alias`")]
pub type Object<R: Ring, I: Ideal<Parent = R> + Ord> = MarkTree<QuotientObject<R, I>>;

/* ## conversion */

impl<R: FactorialRing, I: PrincipalIdeal<Parent = R> + Ord> FromIterator<QuotientObject<R, I>>
    for Object<R, I>
{
    fn from_iter<J: IntoIterator<Item = QuotientObject<R, I>>>(iter: J) -> Self {
        let mut bt = Self::default();
        for j in iter {
            bt.insert(j);
        }
        bt
    }
}

impl<R: FactorialRing, I: PrincipalIdeal<Parent = R> + Ord> FromIterator<I> for Object<R, I> {
    fn from_iter<J: IntoIterator<Item = I>>(iter: J) -> Self {
        iter.into_iter().map(QuotientObject::from).collect()
    }
}

impl<R: FactorialRing + From<u16>, I: PrincipalIdeal<Parent = R> + Ord> FromIterator<u16>
    for Object<R, I>
{
    fn from_iter<J: IntoIterator<Item = u16>>(iter: J) -> Self {
        iter.into_iter().map(QuotientObject::from).collect()
    }
}

/* ## debug and display */

impl<R: Ring + fmt::Debug, I: Ideal<Parent = R> + Ord + fmt::Debug> fmt::Debug for Object<R, I>
where
    QuotientObject<R, I>: fmt::Debug,
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

/*
impl<R: Ring + fmt::Display, I: Ideal<Parent = R> + Ord + fmt::Display> fmt::Display
    for Object<R, I>
where
    QuotientObject<R, I>: fmt::Display,
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
                        .map(|mark| format!("Z{}", mark.thing))
                        .collect::<Vec<_>>()
                        .join("x"),
                )
            }
        }
    }
}
*/

impl<Period: Radix + IsGreater<U1>> fmt::Display for Object<C<Period>, CIdeal<Period>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.is_empty() {
            true => write!(f, "0"),
            false => {
                write!(
                    f,
                    "{}",
                    self.buffer
                        .iter()
                        .map(|mark| format!("Z{}", u16::from(mark.thing.ideal.generator())))
                        .collect::<Vec<_>>()
                        .join("x"),
                )
            }
        }
    }
}

impl<Period: Radix + IsGreater<U1> + Unsigned> PrettyName for Object<C<Period>, CIdeal<Period>> {
    const PRETTY_NAME: &'static str = "Zn-Module";
}

/* ## functionality */

/* ### builders */

impl<R: FactorialRing, I: PrincipalIdeal<Parent = R> + Ord> Object<R, I> {
    // this is expensive and should not be done often
    fn insert(&mut self, quotient: QuotientObject<R, I>) {
        if !quotient.is_trivial() {
            self.buffer.extend(
                quotient
                    .ideal
                    .generator()
                    .power_factors()
                    .map(|p| Mark::from(QuotientObject::from(I::principal(p)))),
            );
        }
    }

    pub fn extend(&mut self, other: Self) {
        for mark in other.buffer {
            self.buffer.insert(mark);
        }
    }

    pub fn join(mut left: Self, right: Self) -> Self {
        left.extend(right);
        left
    }
}

impl<R: Ring + Copy, I: Ideal<Parent = R> + Ord> Object<R, I> {
    pub fn split(self) -> (Self, Self) {
        let mut buffer = self.buffer.into_iter().enumerate().collect::<Vec<_>>();
        let buffer_left = buffer
            .extract_if(|&mut (ref index, ref _mark)| *index % 2 == 0)
            .map(|(_index, mark)| mark)
            .collect::<BTreeSet<_>>();
        let buffer_right = buffer
            .into_iter()
            .map(|(_index, mark)| mark)
            .collect::<BTreeSet<_>>();
        (
            Self {
                buffer: buffer_left,
            },
            Self {
                buffer: buffer_right,
            },
        )
    }

    /*
    transforms the set to a map,
    # warning
    will behave badly if an unmatching number of elements is provided
    */
    pub fn element_from_iterator<J: Iterator<Item = R>>(&self, iter: J) -> Element<R, I> {
        Element {
            buffer: self
                .buffer
                .iter()
                .zip(iter)
                .map(|(object, element)| {
                    object
                        .clone()
                        .map(|quotient| quotient.attach_element(element))
                })
                .collect(),
        }
    }

    /*
    transforms the set to a map,
    # warning
    will behave badly if an unmatching number of elements is provided
    */
    pub fn element_from_matrix(&self, matrix: Matrix<R>) -> Element<R, I> {
        self.element_from_iterator(matrix.into_iter())
    }
}

/* ### populators */

impl<R: Ring + Into<u16>, I: PrincipalIdeal<Parent = R> + Ord> Object<R, I> {
    pub fn torsion_coeffs_as_u16(&self) -> impl Iterator<Item = u16> + '_ {
        self.iter()
            .map(|mark| mark.thing.ideal.clone().generator().into())
    }
}

impl<R: FactorialRing, I: PrincipalIdeal<Parent = R> + Ord> PartiallyEnumerableObject
    for Object<R, I>
{
    fn all_fixed_dimension(dimension: usize) -> impl Iterator<Item = Self> + Clone {
        match dimension {
            0 => vec![Self::trivial()].into_iter(),
            d => {
                // this elements function should actually produce all quotient objects
                let i = QuotientObject::<R, I>::all().filter(|quotient| {
                    !quotient.is_trivial()
                        && quotient.ideal.clone().generator().power_factors().count() == 1
                });
                (0..d)
                    .map(|_| i.clone())
                    .multi_cartesian_product()
                    .map(Self::from_iter)
                    // the collect is necessary to force iterator type.
                    // we could box, but that requires lifetime bounds
                    // and is difficult to enforce clone
                    .collect::<Vec<_>>()
                    .into_iter()
            }
        }
    }
}

/* ## module structure */

impl<R: Ring, I: Ideal<Parent = R> + Ord> CatObject for Object<R, I> {}

impl<R: Ring + Copy, I: Ideal<Parent = R> + Ord> ConcreteObject for Object<R, I> {
    type Element = Element<R, I>;

    fn is_element(&self, el: &Self::Element) -> bool {
        itertools::equal(
            self.buffer.iter().map(|mark| &mark.thing.ideal),
            el.buffer.iter().map(|mark| &mark.thing.ideal),
        )
    }

    fn elements(&self) -> impl Iterator<Item = Self::Element> + Clone + '_ {
        match self.dimension() {
            0 => vec![self.zero()].into_iter(),
            _ => self
                .buffer
                .iter()
                .map(|mark| mark.thing.elements())
                .multi_cartesian_product()
                .map(move |vec| {
                    self.clone()
                        .element_from_iterator(vec.into_iter().map(|qelement| qelement.element))
                })
                // collect necessary to force return type to be the same in both branches
                .collect::<Vec<_>>()
                .into_iter(),
        }
    }

    default fn cardinality(&self) -> usize {
        self.elements().count()
    }
}

impl<R: Ring + Copy + Into<u16>, I: PrincipalIdeal<Parent = R> + Ord> ConcreteObject
    for Object<R, I>
{
    fn cardinality(&self) -> usize {
        self.torsion_coeffs_as_u16()
            .fold(1_u16, u16::saturating_mul)
            .into()
    }
}

impl<R: Ring + Copy, I: Ideal<Parent = R> + Ord> ModuleObject<R> for Object<R, I> {
    fn is_trivial(&self) -> bool {
        self.is_empty()
    }

    fn trivial() -> Self {
        Self::default()
    }

    fn zero(&self) -> Self::Element {
        self.element_from_iterator(self.buffer.iter().map(|_| R::zero()))
    }
}

impl<R: Ring + Copy, I: Ideal<Parent = R> + Ord> Object<R, I> {
    pub fn dimension(&self) -> usize {
        self.len()
    }

    pub fn versor(&self, mark: &Mark<QuotientObject<R, I>>) -> <Self as ConcreteObject>::Element {
        self.element_from_iterator(self.buffer.iter().map(|m| match m == mark {
            true => R::one(),
            false => R::zero(),
        }))
    }
}

impl<R: Ring + Copy, I: Ideal<Parent = R> + Ord> DuplicableObject for Object<R, I> {
    /**
    returns a tree isomorphic to self,
    but with *different* coefficient uuids
    */
    fn duplicate(&self) -> Self {
        Self {
            buffer: self.buffer.iter().map(Mark::duplicate).collect(),
        }
    }
}

/* ### sub and quot structures */

impl<Period: Radix + IsGreater<U1> + Send + Sync> Object<C<Period>, CIdeal<Period>> {
    #[allow(clippy::panic, reason = "structural properties")]
    pub fn submodules(self) -> Vec<CanonToCanon<C<Period>, CIdeal<Period>>> {
        match self.dimension() {
            0 => {
                let arc = Arc::new(self);
                vec![CanonToCanon::new(&arc, &arc, Matrix::from_buffer([], 0, 0))]
            }
            1 => submodules_of_cyclic_module(self),
            _n => DirectModule::from(self).submodules_goursat(),
        }
    }

    #[allow(clippy::panic, reason = "structural properties")]
    pub fn quotients(self) -> Vec<CanonToCanon<C<Period>, CIdeal<Period>>> {
        match self.dimension() {
            0 => {
                let arc = Arc::new(self);
                vec![CanonToCanon::new(&arc, &arc, Matrix::from_buffer([], 0, 0))]
            }
            1 => quotients_of_cyclic_module(self),
            _n => DirectModule::from(self).quotients_goursat(),
        }
    }
}

#[allow(clippy::expect_used, reason = "structural properties")]
pub fn submodules_of_cyclic_module<Period: Radix + IsGreater<U1>>(
    module: Object<C<Period>, CIdeal<Period>>,
) -> Vec<CanonToCanon<C<Period>, CIdeal<Period>>> {
    let target = Arc::new(module);
    let coeff = target
        .iter()
        .next()
        .expect("we assumed the module is cyclic, so it should exactly one coefficient");
    let generator = coeff.thing.ideal.generator();
    generator
        .naive_divisors()
        .map(|divisor| {
            let source = Arc::new(Object::from_iter([CIdeal::principal(divisor)]));
            CanonToCanon::new(
                &source,
                &target,
                match divisor.is_one() {
                    true => Matrix::from_buffer([], 0, 1),
                    false => Matrix::from_buffer(
                        [generator
                            .try_divide(divisor)
                            .next()
                            .expect("divisor will divide")],
                        1,
                        1,
                    ),
                },
            )
        })
        // necessary to force ownership
        .collect()
}

#[allow(clippy::expect_used, reason = "structural properties")]
pub fn quotients_of_cyclic_module<Period: Radix + IsGreater<U1>>(
    module: Object<C<Period>, CIdeal<Period>>,
) -> Vec<CanonToCanon<C<Period>, CIdeal<Period>>> {
    let source = Arc::new(module);
    let coeff = source
        .iter()
        .next()
        .expect("we assumed the module is cyclic, so it should exactly one coefficient");
    let generator = coeff.thing.ideal.generator();
    generator
        .naive_divisors()
        .map(|divisor| {
            let target = Arc::new(Object::from_iter([CIdeal::principal(divisor)]));
            CanonToCanon::new(
                &source,
                &target,
                match divisor.is_one() {
                    true => Matrix::from_buffer([], 1, 0),
                    false => Matrix::from_buffer([C::one()], 1, 1),
                },
            )
        })
        .collect()
}

// - - -

/* # test */

#[cfg(test)]
mod test {
    #![allow(non_snake_case, reason = "module names look this way")]
    use super::*;
    use typenum::{U2, U3, U30, U36, U4, U6, U64, U7, U8};

    /* ## building */

    #[test]
    fn inserting_trivial_modules() {
        type R = C<U6>;
        type I = CIdeal<U6>;
        let mut ct = Object::<R, I>::default();

        ct.insert(QuotientObject::from(1));
        let mut marks = ct.buffer.iter();
        assert_eq!(marks.next(), None, "cannot insert trivial module");
    }

    #[test]
    fn inserting_modules() {
        type R = C<U36>;
        type I = CIdeal<U36>;
        let mut ct = Object::<R, I>::default();

        ct.insert(QuotientObject::from(6));
        ct.insert(QuotientObject::from(6));
        ct.insert(QuotientObject::from(2));
        ct.insert(QuotientObject::from(2));

        let mut marks = ct
            .buffer
            .iter()
            .map(|x| u16::from(x.thing.ideal.generator()));
        assert_eq!(marks.next(), Some(2));
        assert_eq!(marks.next(), Some(2));
        assert_eq!(marks.next(), Some(2));
        assert_eq!(marks.next(), Some(2));
        assert_eq!(marks.next(), Some(3));
        assert_eq!(marks.next(), Some(3));
        assert_eq!(marks.next(), None);
    }

    #[test]
    fn building_from_iterator() {
        type R = C<U36>;
        type I = CIdeal<U36>;
        let ct = Object::<R, I>::from_iter([6, 4].map(|j| CIdeal::principal(C::from(j))));

        let mut marks = ct
            .buffer
            .iter()
            .map(|x| u16::from(x.thing.ideal.generator()));
        assert_eq!(marks.next(), Some(2));
        assert_eq!(marks.next(), Some(3));
        assert_eq!(marks.next(), Some(4));
        assert_eq!(marks.next(), None);
    }

    #[test]
    fn splitting() {
        type R = C<U64>;
        type I = CIdeal<U64>;
        let ct =
            Object::<R, I>::from_iter([32, 16, 8, 4, 2].map(|j| CIdeal::principal(C::from(j))));

        let (l, r) = ct.split();

        let mut marks_left = l
            .buffer
            .iter()
            .map(|x| u16::from(x.thing.ideal.generator()));
        assert_eq!(marks_left.next(), Some(2));
        assert_eq!(marks_left.next(), Some(8));
        assert_eq!(marks_left.next(), Some(32));
        assert_eq!(marks_left.next(), None);

        let mut marks_right = r
            .buffer
            .iter()
            .map(|x| u16::from(x.thing.ideal.generator()));
        assert_eq!(marks_right.next(), Some(4));
        assert_eq!(marks_right.next(), Some(16));
        assert_eq!(marks_right.next(), None);
    }

    #[test]
    fn joining() {
        type R = C<U36>;
        type I = CIdeal<U36>;
        let l = Object::<R, I>::from_iter([4, 2].map(|j| CIdeal::principal(C::from(j))));
        let r = Object::<R, I>::from_iter([6, 3].map(|j| CIdeal::principal(C::from(j))));

        let ct = Object::join(l, r);

        let mut marks = ct
            .buffer
            .iter()
            .map(|x| u16::from(x.thing.ideal.generator()));
        assert_eq!(marks.next(), Some(2));
        assert_eq!(marks.next(), Some(2));
        assert_eq!(marks.next(), Some(3));
        assert_eq!(marks.next(), Some(3));
        assert_eq!(marks.next(), Some(4));
        assert_eq!(marks.next(), None);
    }

    /* ## module structure */

    #[test]
    #[allow(clippy::shadow_unrelated, reason = "useful in test")]
    fn checking_elements() {
        type R = C<U6>;
        type I = CIdeal<U6>;
        let z1 = Object::<R, I>::from_iter([1]);
        let z6 = Object::<R, I>::from_iter([6]);

        assert!(z1.is_element(&z1.element_from_iterator([0].map(R::from).into_iter())));
        assert!(z6.is_element(&z6.element_from_iterator([0, 0].map(R::from).into_iter())));
        assert!(!z1.is_element(&z6.element_from_iterator([0, 0].map(R::from).into_iter())));
        assert!(!z6.is_element(&z1.element_from_iterator([0].map(R::from).into_iter())));
    }

    #[test]
    #[allow(clippy::shadow_unrelated, reason = "useful in test")]
    fn enumerating_elements() {
        type R = C<U6>;
        type I = CIdeal<U6>;
        let a = Object::<R, I>::from_iter([1].map(|j| CIdeal::principal(C::from(j))));
        assert!(a.is_trivial());
        itertools::assert_equal(
            a.elements().map(|element| {
                element
                    .buffer
                    .into_iter()
                    .map(|quotient| quotient.thing.element)
                    .collect::<Vec<_>>()
            }),
            [vec![]],
        );

        let a = Object::<R, I>::from_iter([3].map(|j| CIdeal::principal(C::from(j))));
        itertools::assert_equal(
            a.elements().map(|element| {
                element
                    .buffer
                    .into_iter()
                    .map(|quotient| quotient.thing.element)
                    .collect::<Vec<_>>()
            }),
            [1, 2, 3].map(|x| vec![C::from(x)]),
        );

        let a = Object::<R, I>::from_iter([2, 2].map(|j| CIdeal::principal(C::from(j))));
        itertools::assert_equal(
            a.elements().map(|element| {
                element
                    .buffer
                    .into_iter()
                    .map(|quotient| quotient.thing.element)
                    .collect::<Vec<_>>()
            }),
            [(1, 1), (1, 2), (2, 1), (2, 2)].map(|(x, y)| vec![C::from(x), C::from(y)]),
        );
    }

    /* ## sub and quot spaces */

    #[test]
    fn submodules_of_Z8() {
        type R = C<U8>;
        type I = CIdeal<U8>;
        let z8 = Object::<R, I>::from_iter([0]);
        let mut submodules = z8.submodules().into_iter();
        assert_eq!(
            submodules.next(),
            Some(CanonToCanon::new(
                &Arc::new(Object::from_iter([1])),
                &Arc::new(Object::from_iter([0])),
                Matrix::from_buffer([], 0, 1),
            ))
        );
        assert_eq!(
            submodules.next(),
            Some(CanonToCanon::new(
                &Arc::new(Object::from_iter([2])),
                &Arc::new(Object::from_iter([0])),
                Matrix::from_buffer([4].map(R::from), 1, 1),
            ))
        );
        assert_eq!(
            submodules.next(),
            Some(CanonToCanon::new(
                &Arc::new(Object::from_iter([4])),
                &Arc::new(Object::from_iter([0])),
                Matrix::from_buffer([2].map(R::from), 1, 1),
            ))
        );
        assert_eq!(
            submodules.next(),
            Some(CanonToCanon::new(
                &Arc::new(Object::from_iter([0])),
                &Arc::new(Object::from_iter([0])),
                Matrix::from_buffer([1].map(R::from), 1, 1),
            ))
        );
        assert_eq!(submodules.next(), None);
    }

    #[test]
    fn quotients_of_Z8() {
        type R = C<U8>;
        type I = CIdeal<U8>;
        let z8 = Object::<R, I>::from_iter([0]);
        let mut quotients = z8.quotients().into_iter();
        assert_eq!(
            quotients.next(),
            Some(CanonToCanon::new(
                &Arc::new(Object::from_iter([0])),
                &Arc::new(Object::from_iter([1])),
                Matrix::from_buffer([], 1, 0),
            ))
        );
        assert_eq!(
            quotients.next(),
            Some(CanonToCanon::new(
                &Arc::new(Object::from_iter([0])),
                &Arc::new(Object::from_iter([2])),
                Matrix::from_buffer([1].map(R::from), 1, 1),
            ))
        );
        assert_eq!(
            quotients.next(),
            Some(CanonToCanon::new(
                &Arc::new(Object::from_iter([0])),
                &Arc::new(Object::from_iter([4])),
                Matrix::from_buffer([1].map(R::from), 1, 1),
            ))
        );
        assert_eq!(
            quotients.next(),
            Some(CanonToCanon::new(
                &Arc::new(Object::from_iter([0])),
                &Arc::new(Object::from_iter([0])),
                Matrix::from_buffer([1].map(R::from), 1, 1),
            ))
        );
        assert_eq!(quotients.next(), None);
    }

    #[test]
    fn submodules_of_Z2xZ4() {
        type R = C<U4>;
        type I = CIdeal<U4>;
        let z24 = Arc::new(Object::<R, I>::from_iter([4, 2]));
        let submodules = (*z24).clone().submodules();

        let trivial = CanonToCanon::new(
            &Arc::new(Object::from_iter([1])),
            &z24,
            Matrix::from_buffer([], 0, 2),
        );
        assert!(submodules.contains(&trivial), "trivial submodule");

        let right_z2 = CanonToCanon::new(
            &Arc::new(Object::from_iter([2])),
            &z24,
            Matrix::from_buffer([0, 2].map(R::from), 1, 2),
        );
        assert!(submodules.contains(&right_z2), "right Z2");

        let left_z2 = CanonToCanon::new(
            &Arc::new(Object::from_iter([2])),
            &z24,
            Matrix::from_buffer([1, 0].map(R::from), 1, 2),
        );
        assert!(submodules.contains(&left_z2), "left Z2");

        let diagonal_z2 = CanonToCanon::new(
            &Arc::new(Object::from_iter([2])),
            &z24,
            Matrix::from_buffer([1, 2].map(R::from), 1, 2),
        );
        assert!(submodules.contains(&diagonal_z2), "diagonal Z2");

        let z2sq_a = CanonToCanon::new(
            &Arc::new(Object::from_iter([2, 2])),
            &z24,
            Matrix::from_buffer([1, 0, 0, 2].map(R::from), 2, 2),
        );
        let z2sq_b = CanonToCanon::new(
            &Arc::new(Object::from_iter([2, 2])),
            &z24,
            Matrix::from_buffer([0, 1, 2, 0].map(R::from), 2, 2),
        );
        assert!(
            submodules.contains(&z2sq_a) || submodules.contains(&z2sq_b),
            "Z2 squared"
        );

        let straight_z4 = CanonToCanon::new(
            &Arc::new(Object::from_iter([4])),
            &z24,
            Matrix::from_buffer([0, 1].map(R::from), 1, 2),
        );
        assert!(submodules.contains(&straight_z4), "straight Z4");

        let diagonal_z4 = CanonToCanon::new(
            &Arc::new(Object::from_iter([4])),
            &z24,
            Matrix::from_buffer([1, 1].map(R::from), 1, 2),
        );
        assert!(submodules.contains(&diagonal_z4), "diagonal Z4");

        let full = CanonToCanon::new(
            &Arc::new(Object::from_iter([4, 2])),
            &z24,
            Matrix::from_buffer([1, 0, 0, 1].map(R::from), 2, 2),
        );
        assert!(submodules.contains(&full), "full submodule");
    }

    #[test]
    fn submodules_of_Z3xZ3() {
        type R = C<U3>;
        type I = CIdeal<U3>;
        let z33 = Arc::new(Object::<R, I>::from_iter([3, 3]));
        let submodules = (*z33).clone().submodules();

        let trivial = CanonToCanon::new(
            &Arc::new(Object::from_iter([1])),
            &z33,
            Matrix::from_buffer([], 0, 2),
        );
        assert!(submodules.contains(&trivial), "trivial submodule");

        let left = CanonToCanon::new(
            &Arc::new(Object::from_iter([3])),
            &z33,
            Matrix::from_buffer([0, 1].map(R::from), 1, 2),
        );
        assert!(submodules.contains(&left), "left Z3");

        let right = CanonToCanon::new(
            &Arc::new(Object::from_iter([3])),
            &z33,
            Matrix::from_buffer([1, 0].map(R::from), 1, 2),
        );
        assert!(submodules.contains(&right), "right Z3");

        let middle = CanonToCanon::new(
            &Arc::new(Object::from_iter([3])),
            &z33,
            Matrix::from_buffer([1, 1].map(R::from), 1, 2),
        );
        assert!(submodules.contains(&middle), "middle Z3");

        let skew_a = CanonToCanon::new(
            &Arc::new(Object::from_iter([3])),
            &z33,
            Matrix::from_buffer([2, 1].map(R::from), 1, 2),
        );
        let skew_b = CanonToCanon::new(
            &Arc::new(Object::from_iter([3])),
            &z33,
            Matrix::from_buffer([1, 2].map(R::from), 1, 2),
        );
        assert!(
            submodules.contains(&skew_a) || submodules.contains(&skew_b),
            "skew Z3"
        );

        let full_a = CanonToCanon::new(
            &Arc::new(Object::from_iter([3, 3])),
            &z33,
            Matrix::from_buffer([1, 0, 0, 1].map(R::from), 2, 2),
        );
        let full_b = CanonToCanon::new(
            &Arc::new(Object::from_iter([3, 3])),
            &z33,
            Matrix::from_buffer([0, 1, 1, 0].map(R::from), 2, 2),
        );
        assert!(
            submodules.contains(&full_a) || submodules.contains(&full_b),
            "full submodule"
        );
    }

    #[test]
    fn submodules_of_Z7xZ7() {
        type R = C<U7>;
        type I = CIdeal<U7>;

        let z7 = Arc::new(Object::<R, I>::from_iter([7]));
        let z77 = Arc::new(Object::<R, I>::from_iter([7, 7]));

        let submodules = (*z77).clone().submodules();

        for j in 2..7 {
            assert!(!submodules.contains(&CanonToCanon::new(
                &z7,
                &z77,
                Matrix::from_buffer([j, j].map(R::from), 1, 2),
            )));
        }
    }

    #[test]
    fn submodules_of_Z2xZ2() {
        type R = C<U2>;
        type I = CIdeal<U2>;

        let z2to2 = Object::<R, I>::from_iter([2, 2]);
        let submodules: Vec<CanonToCanon<R, I>> = z2to2.submodules();

        assert_eq!(
            submodules.len(),
            5,
            "there should be 1 + 3 + 1 = 5 subgroups"
        );

        for module in submodules.iter().combinations(2) {
            assert_ne!(module.get(0), module.get(1));
        }
    }

    #[test]
    fn submodules_of_Z2xZ2xZ2() {
        type R = C<U2>;
        type I = CIdeal<U2>;

        let z2to3 = Object::<R, I>::from_iter([2, 2, 2]);
        let submodules: Vec<CanonToCanon<R, I>> = z2to3.submodules();

        assert_eq!(
            submodules.len(),
            16,
            "there should be 1 + 7 + 7 + 1 = 16 subgroups"
        );

        for module in submodules.iter().combinations(2) {
            assert_ne!(module.get(0), module.get(1));
        }
    }

    #[test]
    fn submodules_of_Z2xZ3xZ5() {
        type R = C<U30>;
        type I = CIdeal<U30>;

        let z = Object::<R, I>::from_iter([2, 3, 5]);
        let submodules: Vec<CanonToCanon<R, I>> = z.submodules();

        assert_eq!(
            submodules.len(),
            8,
            "there should be 1 + 3 + 3 + 1 = 8 subgroups"
        );

        for module in submodules.iter().combinations(2) {
            assert_ne!(module.get(0), module.get(1));
        }
    }

    #[test]
    fn submodules_of_Z2xZ2xZ2xZ2() {
        type R = C<U2>;
        type I = CIdeal<U2>;

        let z2to4 = Object::<R, I>::from_iter([2, 2, 2, 2]);
        let submodules: Vec<CanonToCanon<R, I>> = z2to4.submodules();

        assert_eq!(
            submodules.len(),
            67,
            "there should be 1 + 15 + 35 + 15 + 1 = 67 subgroups"
        );

        for module in submodules.iter().combinations(2) {
            assert_ne!(module.get(0), module.get(1));
        }
    }

    #[test]
    fn submodules_of_Z2xZ2xZ2xZ2xZ2() {
        type R = C<U2>;
        type I = CIdeal<U2>;

        let z2to5 = Object::<R, I>::from_iter([2, 2, 2, 2, 2]);
        let submodules: Vec<CanonToCanon<R, I>> = z2to5.submodules();

        assert_eq!(
            submodules.len(),
            374,
            "there should be 1 + 31 + 155 + + 155 + 31 + 1 = 374 subgroups"
        );

        for module in submodules.iter().combinations(2) {
            assert_ne!(module.get(0), module.get(1));
        }
    }
}
