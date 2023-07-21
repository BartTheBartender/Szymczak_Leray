#![feature(specialization)]
#![feature(let_chains)]
#![allow(dead_code)]
#![feature(associated_type_bounds)]

#[allow(unused_imports)]
mod category;
mod error;
mod util;
mod zmodule;
//mod szymczak_category;

use crate::{category::relation::*, zmodule::canon::*};
use bitvec::prelude::*;
use std::rc::Rc;

pub type Int = u8;
pub type TorsionCoeff = Vec<Int>;

fn main() {
    let left_krakowian = bitvec![1, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 1];
    let right_krakowian = bitvec![1, 0, 1, 1, 0, 1, 1, 0];
    let column_size: usize = 4;

    let result_krakowian =
        Relation::krakowian_product_unchecked(&left_krakowian, &right_krakowian, column_size);

    print!("{}", &result_krakowian);
}
