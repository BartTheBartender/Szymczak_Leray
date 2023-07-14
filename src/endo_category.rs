pub trait Morphism<P, Q> {
    fn compose<S, M, T, L: Morphism<S, M>, R: Morphism<M, T>, O: Morphism<S, T>>(
        left: L,
        right: R,
    ) -> O;
}
