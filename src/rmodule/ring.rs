use crate::Int;
use std::{
    array::from_fn,
    ops::{Add, Mul},
};

pub type Zahl = Int;

/* # structure */

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Fin<const RADIX: Zahl> {
    zahl: Zahl,
}

/* # traits */

/* ## set */

pub trait Set: PartialEq + Eq + Clone + Copy + Send + Sync {
    const CARD: Zahl;

    fn get(&self) -> Zahl;
    fn new(z: Zahl) -> Option<Self>;

    fn elements() -> [Self; Self::CARD as usize] {
        from_fn::<Self, { Self::CARD as usize }, _>(|j| {
            Self::new(j.try_into().expect("we're gonna need a bigger int"))
                .expect("values will not be out of range")
        })
    }
}

impl<const RADIX: Zahl> Set for Fin<RADIX> {
    const CARD: Zahl = RADIX;

    fn get(&self) -> Zahl {
        self.zahl
    }

    fn new(z: Zahl) -> Option<Self> {
        match RADIX == 0 {
            true => None,
            false => Some(Self { zahl: z % RADIX }),
        }
    }
}

/* ## ring */

// i could try to enforce non zero CARD for rings
// but this would require some advanced types that would have to be dragged around
// technically they would be evaluated at comiple time
// but they would take up memory
pub trait Ring: Add<Self, Output = Self> + Mul<Self, Output = Self> + Set {
    fn zero() -> Self;
    fn one() -> Self;
}

impl<const RADIX: Zahl> Add<Self> for Fin<RADIX> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(self.zahl + other.zahl).expect("output is always Some(z)")
    }
}

impl<const RADIX: Zahl> Mul<Self> for Fin<RADIX> {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self::new(self.zahl * other.zahl).expect("output is always Some(z)")
    }
}

impl<const RADIX: Zahl> Ring for Fin<RADIX> {
    fn zero() -> Self {
        Self::new(0).expect("output is always Some(z)")
    }

    fn one() -> Self {
        Self::new(1).expect("output is always Some(z)")
    }
}
