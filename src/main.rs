#![feature(specialization)]
#![feature(return_position_impl_trait_in_trait)]
#![feature(generic_const_exprs)]
#![feature(let_chains)]
#![feature(btree_extract_if)]
#![feature(extract_if)]
#![feature(trait_alias)]
#![feature(associated_type_bounds)]
#![feature(arc_unwrap_or_clone)]
#![allow(dead_code)] // REMOVE THIS LATER
#![allow(incomplete_features)]

#[allow(unused_imports)]
mod category;
mod error;
mod matrix;
mod rmodule;
mod util;
//mod szymczak_category;

// use crate::category::relation::*; UNCOMMENT THIS
// use bitvec::prelude::*;
// use std::rc::Rc;

pub type Int = u8;
pub type TorsionCoeff = Vec<Int>;

fn main() {
    // UNCOMMENT THIS
    // let left_krakowian = bitvec![1, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 1];
    // let right_krakowian = bitvec![1, 0, 1, 1, 0, 1, 1, 0];
    // let column_size: usize = 4;

    // {
    //     let result_krakowian =
    //         Relation::krakowian_product_unchecked(&left_krakowian, &right_krakowian, column_size);

    //     print!("{}", &result_krakowian);
    // }
}
