//lorem ipsum
use std::ops::Mul;

pub trait Object {}//

pub trait Morphism<source: Object, target: Object>: Mul {
    fn source(&self) -> Object;
    fn target(&self) -> Object;
    fn kernel(&self) -> ??? //
    fn cokernel(&self) -> ???
    fn is_zero(&self) -> bool;
}

pub trait EndoMorphism: Morphism {
    
    fn roof(&self) -> ???
}

