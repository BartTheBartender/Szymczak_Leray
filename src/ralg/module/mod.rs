use crate::{
    category::object::Concrete,
    ralg::ring::{AdditivePartialGroup, Ring},
};

pub mod canon;
pub mod direct;
pub mod map;
mod quotient;

/*
i would love this to be a group,
but having each module be a different type
is fucking hard to achieve within rust's type system.
even if we assume that the ring is bezout
and that the structural theorem holds,
we would have to somehow encode every ideal as a separate type
— which by itself is doable, i think —
and the module would have to depend on a list of ideals,
of possibly unknown at compile time lenght.
*/
pub trait Module<R: Ring>: AdditivePartialGroup {
    fn mul(self, r: R) -> Self;
    fn mul_assign(&mut self, r: R);
}

pub trait ModuleObject<R: Ring>: Concrete {
    fn is_trivial(&self) -> bool;
    fn trivial() -> Self;
    fn zero(&self) -> Self::Element;
}
