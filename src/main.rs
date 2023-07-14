/*const BASE: usize = 5;
const MAX_DIMENSION: usize = 2;

type Int = u8;

mod endo_category;
mod torsion_coeff;
mod zn_modules;

fn main() {
    let m = crate::zn_modules::ZnModule::<2, 2>::new();
    println!("{}", m);

    let a = m.0[1].clone();
    let b = m.0[2].clone();
    let c = a + b;

    let a = m.0[1].clone();
    let b = m.0[2].clone();

    print!("{} + {} = {}", a, b, c);
}*/

mod endo_category;
use endo_category::{ComposeLeft, Morphism};

struct Relation<S, T> {
    s: S,
    t: T,
}

impl<P, Q> Morphism<P, Q> for Relation<P, Q> {}

impl<S, M, T> ComposeLeft<S, M, T, Relation<M, T>> for Relation<S, M> {
    type Output = Relation<S, T>;

    fn compose_left(self, left: Relation<M, T>) -> Self::Output {
        todo!()
    }
}

fn main() {}
