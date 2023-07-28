#![feature(let_chains)]
#![allow(dead_code)]
#![feature(associated_type_bounds)]

#[allow(unused_imports)]
mod category;
mod error;
mod szymczak_category;
mod util;
mod zmodule;

pub type Int = u8;
pub type TorsionCoeff = Vec<Int>;
pub const RECURSION_PARAMETER_SZYMCZAK_FUNCTOR: usize = 100; //idk xD

use zmodule::canon::CanonZModule;

fn main() {
    //let x = category::Category::new(4, 5);

    let x = util::category_of_relations::calculate_helper_indices(
        &CanonZModule::new(vec![2, 2]),
        &CanonZModule::new(vec![4, 4]),
    );

    print!("{:?}", x);
}
