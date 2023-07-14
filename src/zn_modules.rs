//imports from other parts of the crate
use crate::torsion_coeff::TORSION_COEFF;
use crate::{Int, BASE, MAX_DIMENSION};

//external imports
use std::fmt::{self, Display};
use std::ops::{Add, Index, IndexMut, Mul};

//aliases
type Container<T> = Vec<T>;

///////////////////////////////////////////////////////////////////////////////////////////////

//a generic element of Z/n module
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct ZnModElement<const DIMENSION: usize, const TC_INDEX: usize>([Int; DIMENSION]);

impl<const DIMENSION: usize, const TC_INDEX: usize> ZnModElement<DIMENSION, TC_INDEX> {
    fn new() -> Self {
        ZnModElement::<DIMENSION, TC_INDEX>([0; DIMENSION])
    }
}

impl<const DIMENSION: usize, const TC_INDEX: usize> Iterator for ZnModElement<DIMENSION, TC_INDEX> {
    type Item = Int;

    fn next(&mut self) -> Option<Self::Item> {
        for i in 0..DIMENSION {
            self.0[i] += 1;
            if self.0[i] == TORSION_COEFF[TC_INDEX][i] {
                self.0[i] = 0;
            } else {
                return Some(self.0[i]);
            }
        }
        None
    }
}

impl<const DIMENSION: usize, const TC_INDEX: usize> Display for ZnModElement<DIMENSION, TC_INDEX> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let elements: Vec<String> = self.0.iter().map(|value| value.to_string()).collect();
        write!(f, "{}", elements.join(""))
    }
}

impl<const DIMENSION: usize, const TC_INDEX: usize> Index<usize>
    for ZnModElement<DIMENSION, TC_INDEX>
{
    type Output = Int;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<const DIMENSION: usize, const TC_INDEX: usize> IndexMut<usize>
    for ZnModElement<DIMENSION, TC_INDEX>
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl<const DIMENSION: usize, const TC_INDEX: usize> Add for ZnModElement<DIMENSION, TC_INDEX> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let torsion_coeff = TORSION_COEFF[TC_INDEX];
        let mut output_array = [0; DIMENSION];

        for index in 0..DIMENSION {
            //consider using an iterator

            output_array[index] = (self.0[index] + other.0[index]) % torsion_coeff[index];
        }

        ZnModElement(output_array)
    }
}

impl<const DIMENSION: usize, const TC_INDEX: usize> Mul for ZnModElement<DIMENSION, TC_INDEX> {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        let torsion_coeff = TORSION_COEFF[TC_INDEX];
        let mut output_array = [0; DIMENSION];

        for index in 0..DIMENSION {
            //consider using an iterator

            output_array[index] = (self.0[index] * other.0[index]) % torsion_coeff[index];
        }

        ZnModElement(output_array)
    }
}

//a generic Z/n module
pub struct ZnModule<const DIMENSION: usize, const TC_INDEX: usize>(
    pub Container<ZnModElement<DIMENSION, TC_INDEX>>,
);

impl<const DIMENSION: usize, const TC_INDEX: usize> ZnModule<DIMENSION, TC_INDEX> {
    pub fn new() -> Self {
        if DIMENSION > MAX_DIMENSION {
            panic!("DIMENSION cannot exceed MAX_DIMENSION in ZnModule constructor");
        }

        let mut container = Container::new();
        let element = ZnModElement::new();

        Self::new_help(&mut container, element, 0);

        ZnModule(container)
    }

    fn new_help(
        container: &mut Container<ZnModElement<DIMENSION, TC_INDEX>>,
        element: ZnModElement<DIMENSION, TC_INDEX>,
        index: usize,
    ) {
        if index == DIMENSION {
            container.push(element); //here be careful, it depends on the current container type
        } else {
            for value in 0..BASE {
                let mut element_ = element.clone();
                element_[index] = value as Int;
                Self::new_help(container, element_, index + 1);
            }
        }
    }
}

impl<const DIMENSION: usize, const TC_INDEX: usize> Iterator for ZnModule<DIMENSION, TC_INDEX> {
    type Item = ZnModElement<DIMENSION, TC_INDEX>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.iter().next().cloned()
    }
}

impl<const DIMENSION: usize, const TC_INDEX: usize> Display for ZnModule<DIMENSION, TC_INDEX> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let elements: Vec<String> = self.0.iter().map(|value| value.to_string()).collect();
        write!(f, "{}", elements.join(" "))
    }
}
