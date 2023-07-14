const BASE: usize = 5;
const MAX_DIMENSION: usize = 2;

type Int = u8;

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
}
