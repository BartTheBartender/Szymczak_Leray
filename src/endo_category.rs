pub trait Morphism<P, Q> {
    // tu wszystkie te rzeczy, które były wcześniej, jak source czy kernel
}

pub trait ComposeLeft<S, M, T, Lhs: Morphism<M, T>> {
    type Output: Morphism<S, T>;

    fn compose_left(self, left: Lhs) -> Self::Output;
}
