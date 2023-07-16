pub trait Morphism: PartialEq + Eq + Sized {
    fn compose(&self, other: &Self) -> Self; //i realized that i dont understand the difference between compose left and right, we should also have "*=" to save time and memory but i cannot find not verbose name for this operation

    fn generate_orbit(&self) -> Vec<Self> {
        todo!() //use composing ofc
    }

    //other stuff with kernels/cokernels...
}
