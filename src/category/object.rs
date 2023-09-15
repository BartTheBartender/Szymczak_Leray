pub trait Object: Sized + PartialEq + Eq {}

pub trait Enumerable: Object {
    fn all() -> impl Iterator<Item = Self> + Clone;
}

pub trait Concrete: Object {
    type Element: Sized + PartialEq + Eq;

    fn elements(&self) -> impl Iterator<Item = Self::Element> + Clone + '_;
    fn is_element(&self, element: &Self::Element) -> bool;
}
