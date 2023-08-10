#![feature(specialization)]
#![feature(return_position_impl_trait_in_trait)]
#![feature(generic_const_exprs)]
#![feature(let_chains)]
#![feature(btree_extract_if)]
#![feature(extract_if)]
#![feature(trait_alias)]
#![feature(associated_type_bounds)]
#![feature(arc_unwrap_or_clone)]
// visual separator
#![allow(incomplete_features)]
#![allow(dead_code)] // REMOVE THIS LATER

mod category;
mod error;
mod matrix;
mod rmodule;
mod szymczak_category;
mod util;

#[allow(unused_imports)]
use {crate::category::relation::*, bitvec::prelude::*, std::rc::Rc};

pub type Int = u16;
pub type TorsionCoeff = Vec<Int>; // this is probably unused nw
pub const RECURSION_PARAMETER_SZYMCZAK_FUNCTOR: usize = 100; //idk xD

fn main() {}
