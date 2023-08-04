use crate::{
    error::Error,
    rmodule::ring::{Radix, Ring},
};

pub mod canon;
pub mod direct;
pub mod map;
pub mod ring;
pub mod torsion;

pub trait Module<RCard: Radix, R: Ring<RCard>> {
    type Element;

    fn zero(&self) -> Self::Element;
    fn is_element(&self, v: &Self::Element) -> bool;

    fn add_unchecked(&self, v: &Self::Element, u: &Self::Element) -> Self::Element;
    // fn increment_unchecked(&self, v: &mut Self::Element, u: &Self::Element);
    fn mul_by_scalar_unchecked(&self, x: R, v: &Self::Element) -> Self::Element;

    fn add(&self, v: &Self::Element, u: &Self::Element) -> Result<Self::Element, Error> {
        match self.is_element(v) && self.is_element(u) {
            true => Ok(self.add_unchecked(v, u)),
            false => Err(Error::InvalidElement),
        }
    }

    // fn increment(&self, v: &mut Self::Element, u: &Self::Element) -> Result<(), Error> {
    //     match self.is_element(v) && self.is_element(u) {
    //         true => {
    //             self.increment_unchecked(v, u);
    //             Ok(())
    //         }
    //         false => Err(Error::InvalidElement),
    //     }
    // }

    fn mul_by_scalar(&self, x: R, v: &Self::Element) -> Result<Self::Element, Error> {
        match self.is_element(v) {
            true => Ok(self.mul_by_scalar_unchecked(x, v)),
            false => Err(Error::InvalidElement),
        }
    }
}
