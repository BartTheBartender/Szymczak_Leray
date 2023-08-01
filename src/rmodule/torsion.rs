// use crate::rmodule::ring::Ring;
use crate::matrix::Matrix;
use derive_where::derive_where;
use gcd::Gcd;
use std::{cmp::Ordering, collections::BTreeMap, ops::Mul};

/* # coprime trait */

pub trait Coprime: Copy + Sized + Eq + Gcd {
    fn one() -> Self;

    fn is_coprime(self, other: Self) -> bool {
        self.gcd(other) == Self::one()
    }
}

/* # coeff wrapper */

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Coeff<T>
where
    T: Clone + Copy + PartialEq + Eq,
{
    coeff: T,
    index: u8,
}

impl<T> Coeff<T>
where
    T: Copy + Eq,
{
    pub fn new(coeff: T, index: u8) -> Self {
        Self { coeff, index }
    }
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
#[derive(Debug, Clone)]
#[derive_where(PartialEq, Eq; V)]
pub struct CoeffTree<T, V>
where
    T: Copy + Eq,
{
    buffer: BTreeMap<Coeff<T>, V>,
}

/* ## basic interface */

impl<T, V> CoeffTree<T, V>
where
    T: Copy + Eq + Ord,
{
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn get(&self, key: &Coeff<T>) -> Option<&V> {
        self.buffer.get(key)
    }

    pub fn get_mut(&mut self, key: &Coeff<T>) -> Option<&mut V> {
        self.buffer.get_mut(key)
    }

    pub fn replace(&mut self, key: &Coeff<T>, value: V) -> Option<()> {
        let v = self.buffer.get_mut(key)?;
        *v = value;
        Some(())
    }

    pub fn contains_key(&self, key: &Coeff<T>) -> bool {
        self.buffer.contains_key(key)
    }

    pub fn has_same_keys<W>(&self, other: &CoeffTree<T, W>) -> bool {
        self.buffer
            .keys()
            .zip(other.buffer.keys())
            .all(|(s, o)| s == o)
    }

    /* # iters */

    pub fn iter(&self) -> impl Iterator<Item = (&Coeff<T>, &V)> {
        self.buffer.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&Coeff<T>, &mut V)> {
        self.buffer.iter_mut()
    }

    pub fn into_iter(self) -> impl Iterator<Item = (Coeff<T>, V)> {
        self.buffer.into_iter()
    }

    pub fn coeffs(&self) -> impl Iterator<Item = T> + '_ {
        self.buffer.keys().map(|key| key.coeff)
    }

    pub fn keys(&self) -> impl Iterator<Item = &Coeff<T>> {
        self.buffer.keys()
    }

    pub fn into_keys(self) -> impl Iterator<Item = Coeff<T>> {
        self.buffer.into_keys()
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.buffer.values()
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.buffer.values_mut()
    }

    pub fn into_values(self) -> impl Iterator<Item = V> {
        self.buffer.into_values()
    }
}

/* ## operations */

impl<T, V> CoeffTree<T, V>
where
    T: Copy + Eq + Ord,
{
    pub fn map<W, F>(self, action: F) -> CoeffTree<T, W>
    where
        F: Fn(V, T) -> W,
    {
        CoeffTree {
            buffer: self
                .buffer
                .into_iter()
                .map(|(key, value)| (key, action(value, key.coeff)))
                .collect(),
        }
    }

    pub fn map_ref<W, F>(&self, action: F) -> CoeffTree<T, W>
    where
        F: Fn(&V, T) -> W,
    {
        CoeffTree {
            buffer: self
                .buffer
                .iter()
                .map(|(key, value)| (*key, action(value, key.coeff)))
                .collect(),
        }
    }

    pub fn map_mut<F>(&mut self, action: F)
    where
        F: Fn(&mut V, T),
    {
        for (key, value) in self.buffer.iter_mut() {
            action(value, key.coeff);
        }
    }

    /**
      # WARNING

      only performs action on keys present in both trees,
      so if the trees have different structures,
      this will invalidate the structural integrity of the trees
      and — therefore — lead to undefined behaviour
    */
    pub fn combine<U, W, F>(self, other: CoeffTree<T, U>, action: F) -> CoeffTree<T, W>
    where
        F: Fn(V, U, T) -> W,
    {
        let mut other_buffer = other.buffer;
        CoeffTree {
            buffer: self
                .buffer
                .into_iter()
                .filter_map(|(self_key, self_value)| {
                    other_buffer.remove(&self_key).map(|other_value| {
                        (self_key, action(self_value, other_value, self_key.coeff))
                    })
                })
                .collect::<BTreeMap<_, _>>(),
        }
    }

    /**
      # WARNING

      only performs action on keys present in both trees,
      so if the trees have different structures,
      this will invalidate the structural integrity of the trees
      and — therefore — lead to undefined behaviour
    */
    pub fn combine_ref<U, W, F>(&self, other: &CoeffTree<T, U>, action: F) -> CoeffTree<T, W>
    where
        F: Fn(&V, &U, T) -> W,
    {
        CoeffTree {
            buffer: self
                .buffer
                .iter()
                .filter_map(|(self_key, self_value)| {
                    other.buffer.get(self_key).map(|other_value| {
                        (*self_key, action(self_value, other_value, self_key.coeff))
                    })
                })
                .collect::<BTreeMap<_, _>>(),
        }
    }
}

impl<T, V> CoeffTree<T, V>
where
    T: Copy + Eq + Ord,
    V: Copy,
{
    pub fn as_matrix(&self) -> Matrix<V> {
        Matrix::from_cols([self.values().copied().collect()], 1)
    }

    pub fn from_matrix(matrix: Matrix<V>, coeffs: &CoeffTree<T, ()>) -> Self {
        Self {
            buffer: coeffs
                .iter()
                .zip(matrix.into_iter())
                .map(|((coeff, _unit), value)| (*coeff, value))
                .collect(),
        }
    }
}

impl<T, V> Default for CoeffTree<T, V>
where
    T: Copy + Eq,
{
    fn default() -> Self {
        Self {
            buffer: BTreeMap::new(),
        }
    }
}

/*
impl<T, V> CoeffTree<T, V>
where
    T: Copy + Eq + Ord,
    V: Default,
{
    fn build_default(coefftree: CoeffTree<T, ()>) -> Self {
        coefftree.map(|_| V::default())
    }
}
*/

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

    pub fn join(&mut self, other: Self) {
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
        let mut ct = CoeffTree::<u8, ()>::default();
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
        let mut ct = CoeffTree::<u8, ()>::default();
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
    fn joining() {
        let mut l = CoeffTree::<u8, ()>::default();
        l.insert(4);
        l.insert(2);

        let mut r = CoeffTree::<u8, ()>::default();
        r.insert(3);
        r.insert(2);

        l.join(r);

        let mut keys = l.buffer.keys();
        assert_eq!(keys.next(), Some(Coeff { coeff: 6, index: 0 }).as_ref());
        assert_eq!(keys.next(), Some(Coeff { coeff: 4, index: 0 }).as_ref());
        assert_eq!(keys.next(), Some(Coeff { coeff: 2, index: 0 }).as_ref());
        assert_eq!(keys.next(), None.as_ref());
    }

    #[test]
    fn addition() {
        let ct = CoeffTree::<u8, ()>::from_iter(vec![5, 3]);
        let mut left: CoeffTree<u8, u8> = ct.map_ref(|_, _| 0);
        left.replace(&Coeff::new(5, 0), 3);
        left.replace(&Coeff::new(3, 0), 2);

        let mut right: CoeffTree<u8, u8> = ct.map_ref(|_, _| 0);
        right.replace(&Coeff::new(5, 0), 4);
        right.replace(&Coeff::new(3, 0), 1);

        let sum = left.combine(right, |l, r, c| (l + r) % c);
        let mut result: CoeffTree<u8, u8> = ct.map_ref(|_, _| 0);
        result.replace(&Coeff::new(5, 0), 2);
        result.replace(&Coeff::new(3, 0), 0);

        assert_eq!(sum.buffer, result.buffer);
    }

    #[test]
    fn multiplication() {
        let ct = CoeffTree::<u8, ()>::from_iter(vec![5, 3]);
        let mut left: CoeffTree<u8, u8> = ct.map_ref(|_, _| 0);
        left.replace(&Coeff::new(5, 0), 3);
        left.replace(&Coeff::new(3, 0), 1);

        let product = left.map(|l, c| (l * 2) % c);
        let mut result: CoeffTree<u8, u8> = ct.map_ref(|_, _| 0);
        result.replace(&Coeff::new(5, 0), 1);
        result.replace(&Coeff::new(3, 0), 2);

        assert_eq!(product.buffer, result.buffer);
    }
}
