#![feature(specialization)]
#[allow(unused_imports)]

type Int = u8;
type TorsionCoeff = Vec<Int>;

mod endocategory;
//mod szymczak_category;

use crate::endocategory::*;

fn main() {
    let base = 9;
    let tc = torsion_coeff::torsion_coeff(base, 3);
    let source = &tc[&3][2];
    let target = &tc[&3][3];

    let hom_set = map::Map::hom_set(base, source, target);

    for m in &hom_set {
        println!("{:?}", m.entries);
    }
    println!("{}", hom_set.len());
}
