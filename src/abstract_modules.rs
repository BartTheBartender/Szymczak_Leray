pub trait Module: Display + Iterator {
    fn split(self) -> (Self, Self);

    fn generate_submodules_from_cyclic(self) -> Vec<Self>;

    fn generate_submodules_goursat(A: &Self, B: &Self) -> Vec<Self>;

    fn generate_submodules(self) -> Vec<Self> {
        let (A, B) = Self::split(A, B);

        if B.len() == 0 {
            A.generate_submodules_from_cyclic()
        } else {
            generate_submodules_goursat(A, B);
        }
    }
}
