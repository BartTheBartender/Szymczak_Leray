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
use endo_category::Morphism;

struct Relation<S, T> {
    s: S,
    t: T,
}

impl<P, Q> Morphism<P, Q> for Relation<P, Q> {
    fn compose<S, M, T, L: Relation<S, M>, R: Relation<M, T>, O: Relation<S, T>>(
        left: L,
        right: R,
    ) -> O {
    }
}

fn main() {}
