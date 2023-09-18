use std::{collections::HashMap, ops};

pub trait Object: Sized + PartialEq + Eq {}

pub trait PartiallyEnumerable: Object {
    fn all_fixed_dimension(dim: usize) -> impl Iterator<Item = Self> + Clone;

    fn all_by_dimension<J: ops::RangeBounds<usize> + IntoIterator<Item = usize>>(
        range: J,
    ) -> impl Iterator<Item = Self> {
        range.into_iter().flat_map(|d| Self::all_fixed_dimension(d))
    }

    fn all_by_dimension_hashed<J: ops::RangeBounds<usize> + IntoIterator<Item = usize>>(
        range: J,
    ) -> HashMap<usize, Vec<Self>> {
        range
            .into_iter()
            .map(|d| (d, Self::all_fixed_dimension(d).collect()))
            .collect()
    }
}

pub trait Enumerable: Object {
    fn all() -> impl Iterator<Item = Self> + Clone;
}

pub trait Concrete: Object {
    type Element: Sized + PartialEq + Eq;

    fn elements(&self) -> impl Iterator<Item = Self::Element> + Clone + '_;
    fn is_element(&self, element: &Self::Element) -> bool;

    fn cardinality(&self) -> usize {
        self.elements().count()
    }
}

pub trait Duplicable: Object {
    fn duplicate(&self) -> Self;
}
