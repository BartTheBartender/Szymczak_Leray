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

use bitvec::prelude::*;

fn main() {}
