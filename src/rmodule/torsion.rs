// use crate::rmodule::ring::Ring;
use gcd::Gcd;
use std::{
    cmp::Ordering,
    collections::BTreeMap,
    ops::{Add, Mul},
};

/* # coprime trait */

pub trait Coprime: Copy + Sized + Eq + Gcd {
    fn one() -> Self;

    fn is_coprime(self, other: Self) -> bool {
        self.gcd(other) == Self::one()
    }
}

/* # coeff wrapper */

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Coeff<T>
where
    T: Clone + Copy + PartialEq + Eq,
{
    coeff: T,
    index: u8,
}

impl<T> PartialOrd for Coeff<T>
where
    T: Copy + Eq + PartialOrd,
{
    // lexicographic order, reversed on both fields
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (
            self.coeff.partial_cmp(&other.coeff),
            self.index.partial_cmp(&other.index),
        ) {
            (None, _) => None,
            (Some(Ordering::Equal), ord) => Some(ord?.reverse()),
            (Some(ord), _) => Some(ord.reverse()),
        }
    }
}

impl<T> Ord for Coeff<T>
where
    T: Copy + Eq + Ord,
{
    // lexicographic order, reversed on both fields
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.coeff.cmp(&other.coeff), self.index.cmp(&other.index)) {
            (Ordering::Equal, ord) => ord.reverse(),
            (ord, _) => ord.reverse(),
        }
    }
}

/* # coefficient tree */

/**
this is structurally guaranteed to be not only sorted (descending),
but also that no pair of elements are coprime
*/
pub struct CoeffTree<T, V>
where
    T: Copy + Eq,
{
    buffer: BTreeMap<Coeff<T>, V>,
}

/* ## basic interface */

impl<T, V> CoeffTree<T, V>
where
    T: Copy + Eq,
{
    fn new() -> Self {
        Self {
            buffer: BTreeMap::new(),
        }
    }

    // fn get() {}
    // fn get_mut() {}
    // fn iter() {}
    // fn iter_mut() {}
}

/* ## coeff tree set */

impl<T> CoeffTree<T, ()>
where
    T: Copy + Eq + Ord + Mul<T, Output = T> + Coprime,
{
    // this is expensive and should not be done often
    fn insert(&mut self, key: T) {
        let mut indexed_key = Coeff {
            coeff: key,
            index: 0,
        };

        match self.buffer.contains_key(&indexed_key) {
            true => {
                // if the tree already contains this key,
                // the key is not coprime with any other key
                indexed_key.index += 1;
                while self.buffer.contains_key(&indexed_key) {
                    indexed_key.index += 1;
                }
                self.buffer.insert(indexed_key, ());
            }
            false => {
                // the key is new
                // search for a key that is relatively prime
                match self
                    .buffer
                    .extract_if(|k, _v| k.coeff.is_coprime(key))
                    .next()
                {
                    Some(k) => self.insert(k.0.coeff * key),
                    None => {
                        self.buffer.insert(indexed_key, ());
                    }
                };
            }
        }
    }

    pub fn split(self) -> (Self, Self) {
        let mut buffer = self.buffer.into_keys().enumerate().collect::<Vec<_>>();
        let buffer_left = buffer
            .extract_if(|(index, _key)| *index % 2 == 0)
            .map(|(_index, key)| (key, ()))
            .collect::<BTreeMap<_, _>>();
        let buffer_right = buffer
            .into_iter()
            .map(|(_index, key)| (key, ()))
            .collect::<BTreeMap<_, _>>();
        (
            Self {
                buffer: buffer_left,
            },
            Self {
                buffer: buffer_right,
            },
        )
    }

    pub fn combine(&mut self, other: Self) {
        for key in other.buffer.into_keys() {
            self.insert(key.coeff);
        }
    }
}

impl<T> FromIterator<T> for CoeffTree<T, ()>
where
    T: Copy + Eq + Ord + Mul<T, Output = T> + Coprime,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut bt = Self {
            buffer: BTreeMap::new(),
        };
        for i in iter {
            bt.insert(i);
        }
        bt
    }
}

/* ## coeff tree map */

impl<T, V> Add for CoeffTree<T, V>
where
    T: Copy + Eq + Ord,
    V: Add<V, Output = V>,
{
    type Output = Self;

    /**
      # WARNING

      only performs operation on keys present in both trees,
      so if the trees have different structures,
      this will invalidate the structural integrity of the trees
      and — therefore — lead to undefined behaviour
    */
    fn add(self, other: Self) -> Self::Output {
        let mut other_buffer = other.buffer;
        Self {
            buffer: self
                .buffer
                .into_iter()
                .filter_map(|(self_key, self_value)| {
                    other_buffer
                        .remove(&self_key)
                        .map(|other_value| (self_key, self_value + other_value))
                })
                .collect::<BTreeMap<_, _>>(),
        }
    }
}

/* # test */

#[cfg(test)]
mod test {
    use super::*;

    impl Coprime for u8 {
        fn one() -> Self {
            1
        }
    }

    #[test]
    fn building() {
        let mut ct = CoeffTree::<u8, ()>::new();
        ct.insert(2);
        ct.insert(3);
        ct.insert(3);
        ct.insert(2);
        ct.insert(2);
        ct.insert(2);

        let mut keys = ct.buffer.keys();
        assert_eq!(keys.next(), Some(Coeff { coeff: 6, index: 1 }).as_ref());
        assert_eq!(keys.next(), Some(Coeff { coeff: 6, index: 0 }).as_ref());
        assert_eq!(keys.next(), Some(Coeff { coeff: 2, index: 1 }).as_ref());
        assert_eq!(keys.next(), Some(Coeff { coeff: 2, index: 0 }).as_ref());
        assert_eq!(keys.next(), None.as_ref());
    }

    #[test]
    fn building_from_iterator() {
        let ct = CoeffTree::<u8, ()>::from_iter(vec![2, 3, 2]);

        let mut keys = ct.buffer.keys();
        assert_eq!(keys.next(), Some(Coeff { coeff: 6, index: 0 }).as_ref());
        assert_eq!(keys.next(), Some(Coeff { coeff: 2, index: 0 }).as_ref());
        assert_eq!(keys.next(), None.as_ref());
    }

    #[test]
    fn splitting() {
        let mut ct = CoeffTree::<u8, ()>::new();
        ct.insert(32);
        ct.insert(16);
        ct.insert(8);
        ct.insert(4);
        ct.insert(2);

        let (l, r) = ct.split();

        let mut keys = l.buffer.keys();
        assert_eq!(
            keys.next(),
            Some(Coeff {
                coeff: 32,
                index: 0
            })
            .as_ref()
        );
        assert_eq!(keys.next(), Some(Coeff { coeff: 8, index: 0 }).as_ref());
        assert_eq!(keys.next(), Some(Coeff { coeff: 2, index: 0 }).as_ref());
        assert_eq!(keys.next(), None.as_ref());

        let mut keys = r.buffer.keys();
        assert_eq!(
            keys.next(),
            Some(Coeff {
                coeff: 16,
                index: 0
            })
            .as_ref()
        );
        assert_eq!(keys.next(), Some(Coeff { coeff: 4, index: 0 }).as_ref());
        assert_eq!(keys.next(), None.as_ref());
    }

    #[test]
    fn combining() {
        let mut l = CoeffTree::<u8, ()>::new();
        l.insert(4);
        l.insert(2);

        let mut r = CoeffTree::<u8, ()>::new();
        r.insert(3);
        r.insert(2);

        l.combine(r);

        let mut keys = l.buffer.keys();
        assert_eq!(keys.next(), Some(Coeff { coeff: 6, index: 0 }).as_ref());
        assert_eq!(keys.next(), Some(Coeff { coeff: 4, index: 0 }).as_ref());
        assert_eq!(keys.next(), Some(Coeff { coeff: 2, index: 0 }).as_ref());
        assert_eq!(keys.next(), None.as_ref());
    }
}
