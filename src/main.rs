#![feature(specialization)]
#![feature(let_chains)]
#[allow(unused_imports)]

type Int = u8;
type TorsionCoeff = Vec<Int>;

mod endocategory;
//mod szymczak_category;

use crate::endocategory::*;
use bitvector::BitVector;
fn main() {
    let source = vec![3];
    let target = vec![2];

    let m = z_module::ZModule::new(source, target);

    /*
    for element in &m.elements {
        println!("order of {:?} is {}", element, m.order(&element));
    }
    */

    let a = m.maximal_cyclic_submodule();

    println!("{:?}", &a.elements);
}
