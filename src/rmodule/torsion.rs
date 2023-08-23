use crate::{
    matrix::Matrix,
    rmodule::ring::{Ring, Zahl},
    util::iterator::product,
};
use derive_where::derive_where;
use nanoid::nanoid;
use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet, HashMap},
    ops::Rem,
    rc::Rc,
};

/* # factorisable trait */

pub trait Factorisable: Ring + Rem<Output = Self> + Ord + std::fmt::Debug {
    fn primes() -> Vec<Self>;

    fn prime_power_decomposition(&self) -> Vec<Self> {
        let mut decomposition = Vec::new();
        let zero = Self::zero();
        let one = Self::one();
        for p in Self::primes() {
            // find highest power of p that divides self
            let mut n = one;
            let mut seen_powers = BTreeSet::new();
            while !seen_powers.contains(&(n * p)) && *self % (n * p) == zero {
                seen_powers.insert(n * p);
                n = n * p;
            }
            // append this higest power to the decomposition
            if n != one {
                decomposition.push(n);
            }
        }
        decomposition
    }
}

/* # coeff wrapper */

#[derive(Clone, Debug, PartialEq, Eq)]
#[derive_where(Hash)]
#[derive_where(skip_inner(Hash))]
pub struct Coeff<T>
where
    T: Clone + Eq,
{
    coeff: T,
    index: Rc<str>,
}

impl<T> Coeff<T>
where
    T: Clone + Eq,
{
    pub fn new(coeff: T, index: Rc<str>) -> Self {
        Self { coeff, index }
    }
}

impl<T> PartialOrd for Coeff<T>
where
    T: Clone + Eq + PartialOrd,
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
    T: Clone + Eq + Ord,
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
but also that every element is either a prime or a power of a prime
*/
#[derive(Debug, Clone)]
#[derive_where(PartialEq, Eq, Hash; V)]
pub struct CoeffTree<T, V>
where
    T: Clone + Eq,
{
    buffer: BTreeMap<Coeff<T>, V>,
}

/* ## basic interface */

impl<T, V> CoeffTree<T, V>
where
    T: Clone + Eq + Ord,
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
        self.buffer.keys().map(|key| key.coeff.clone())
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
    T: Clone + Eq + Ord,
{
    pub fn map<W, F>(self, action: F) -> CoeffTree<T, W>
    where
        F: Fn(V, T) -> W,
    {
        CoeffTree {
            buffer: self
                .buffer
                .into_iter()
                .map(|(key, value)| (key.clone(), action(value, key.coeff.clone())))
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
                .map(|(key, value)| (key.clone(), action(value, key.coeff.clone())))
                .collect(),
        }
    }

    pub fn map_mut<F>(&mut self, action: F)
    where
        F: Fn(&mut V, T),
    {
        for (key, value) in self.buffer.iter_mut() {
            action(value, key.coeff.clone());
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
                        (
                            self_key.clone(),
                            action(self_value, other_value, self_key.coeff.clone()),
                        )
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
                        (
                            self_key.clone(),
                            action(self_value, other_value, self_key.coeff.clone()),
                        )
                    })
                })
                .collect::<BTreeMap<_, _>>(),
        }
    }
}

impl<T, V> CoeffTree<T, V>
where
    T: Clone + Eq + Ord,
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
                .map(|((coeff, _unit), value)| (coeff.clone(), value))
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

/* ## coeff tree set */

impl<T> CoeffTree<T, ()>
where
    T: Factorisable + Ord,
{
    // this is expensive and should not be done often
    fn insert(&mut self, key: T) {
        self.buffer
            .extend(key.prime_power_decomposition().into_iter().map(|p| {
                (
                    Coeff {
                        coeff: p,
                        index: Rc::from(nanoid!()),
                    },
                    (),
                )
            }));
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
        for (key, value) in other.buffer.into_iter() {
            self.buffer.insert(key, value);
        }
    }

    fn all_torsion_coeffs_fixed_dimension(dimension: Zahl) -> impl Iterator<Item = Self> {
        product(
            T::ideals()
                .filter(|ideal| !ideal.is_one() && ideal.prime_power_decomposition().len() == 1),
            dimension,
        )
        .map(|coeffs| Self::from_iter(coeffs))
    }

    pub fn all_torsion_coeffs(maximal_dimension: Zahl) -> impl Iterator<Item = Self> {
        (1..maximal_dimension + 1)
            .flat_map(|dimension| Self::all_torsion_coeffs_fixed_dimension(dimension))
    }

    pub fn all_torsion_coeffs_hashed(maximal_dimension: Zahl) -> HashMap<Zahl, Vec<Self>> {
        (1..maximal_dimension + 1)
            .map(|dimension| {
                (
                    dimension,
                    Self::all_torsion_coeffs_fixed_dimension(dimension).collect(),
                )
            })
            .collect()
    }
}

impl<T> FromIterator<T> for CoeffTree<T, ()>
where
    T: Ord + Factorisable,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut bt = Self {
            buffer: BTreeMap::new(),
        };
        for i in iter {
            bt.insert(i);
        }
        // if bt.buffer.is_empty() {
        //     bt.insert(T::one());
        // }
        bt
    }
}

/* # test */

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        rmodule::ring::{Set, Zahl},
        util::number::divisors,
    };
    use typenum::U256;

    impl Set for i8 {
        type Card = U256;

        fn new(x: Zahl) -> Self {
            x as i8
        }
        fn get(&self) -> Zahl {
            *self as Zahl
        }
    }
    impl Ring for i8 {
        fn zero() -> Self {
            0
        }
        fn one() -> Self {
            1
        }
        fn is_zero(&self) -> bool {
            *self == 0
        }
        fn is_one(&self) -> bool {
            *self == 1
        }
        fn divide_by(&self, other: &Self) -> Option<Self> {
            match self % other {
                0 => Some(self / other),
                _ => None,
            }
        }
        fn ideals() -> impl Iterator<Item = Self> + Clone {
            [1, 2, 4, 8, 16, 32, 64].into_iter()
        }
        fn subideals(&self) -> impl Iterator<Item = Self> {
            divisors(*self as Zahl).into_iter().map(|x| x as i8)
        }
    }

    impl Factorisable for i8 {
        fn primes() -> Vec<Self> {
            // enough for the tests
            vec![2, 3, 5]
        }
    }

    #[test]
    fn building() {
        let mut ct = CoeffTree::<i8, ()>::default();
        ct.insert(6);
        ct.insert(6);
        ct.insert(2);
        ct.insert(2);

        let mut keys = ct.buffer.keys();
        assert_eq!(keys.next().unwrap().coeff, 3);
        assert_eq!(keys.next().unwrap().coeff, 3);
        assert_eq!(keys.next().unwrap().coeff, 2);
        assert_eq!(keys.next().unwrap().coeff, 2);
        assert_eq!(keys.next().unwrap().coeff, 2);
        assert_eq!(keys.next().unwrap().coeff, 2);
        assert_eq!(keys.next(), None.as_ref());
    }

    #[test]
    fn building_from_iterator() {
        let ct = CoeffTree::<i8, ()>::from_iter(vec![6, 4]);

        let mut keys = ct.buffer.keys();
        assert_eq!(keys.next().unwrap().coeff, 4);
        assert_eq!(keys.next().unwrap().coeff, 3);
        assert_eq!(keys.next().unwrap().coeff, 2);
        assert_eq!(keys.next(), None.as_ref());
    }

    #[test]
    fn splitting() {
        let mut ct = CoeffTree::<i8, ()>::default();
        ct.insert(32);
        ct.insert(16);
        ct.insert(8);
        ct.insert(4);
        ct.insert(2);

        let (l, r) = ct.split();

        let mut keys = l.buffer.keys();
        assert_eq!(keys.next().unwrap().coeff, 32);
        assert_eq!(keys.next().unwrap().coeff, 8);
        assert_eq!(keys.next().unwrap().coeff, 2);
        assert_eq!(keys.next(), None.as_ref());

        let mut keys = r.buffer.keys();
        assert_eq!(keys.next().unwrap().coeff, 16);
        assert_eq!(keys.next().unwrap().coeff, 4);
        assert_eq!(keys.next(), None.as_ref());
    }

    #[test]
    fn joining() {
        let mut l = CoeffTree::<i8, ()>::default();
        l.insert(4);
        l.insert(2);

        let mut r = CoeffTree::<i8, ()>::default();
        r.insert(6);
        r.insert(3);

        l.join(r);

        let mut keys = l.buffer.keys();
        assert_eq!(keys.next().unwrap().coeff, 4);
        assert_eq!(keys.next().unwrap().coeff, 3);
        assert_eq!(keys.next().unwrap().coeff, 3);
        assert_eq!(keys.next().unwrap().coeff, 2);
        assert_eq!(keys.next().unwrap().coeff, 2);
        assert_eq!(keys.next(), None.as_ref());
    }

    #[test]
    fn addition() {
        let ct = CoeffTree::<i8, ()>::from_iter(vec![5, 3]);
        let mut left: CoeffTree<i8, i8> = ct.map_ref(|_, _| 0);
        for (key, value) in ct.keys().zip([3, 2]) {
            left.replace(key, value);
        }

        let mut right: CoeffTree<i8, i8> = ct.map_ref(|_, _| 0);
        for (key, value) in ct.keys().zip([4, 1]) {
            right.replace(key, value);
        }

        let sum = left.combine(right, |l, r, c| (l + r) % c);
        let mut result: CoeffTree<i8, i8> = ct.map_ref(|_, _| 0);
        for (key, value) in ct.keys().zip([2, 0]) {
            result.replace(key, value);
        }

        assert_eq!(sum.buffer, result.buffer);
    }

    #[test]
    fn multiplication() {
        let ct = CoeffTree::<i8, ()>::from_iter(vec![5, 3]);
        let mut left: CoeffTree<i8, i8> = ct.map_ref(|_, _| 0);
        for (key, value) in ct.keys().zip([3, 1]) {
            left.replace(key, value);
        }

        let product = left.map(|l, c| (l * 2) % c);
        let mut result: CoeffTree<i8, i8> = ct.map_ref(|_, _| 0);
        for (key, value) in ct.keys().zip([1, 2]) {
            result.replace(key, value);
        }

        assert_eq!(product.buffer, result.buffer);
    }
}
