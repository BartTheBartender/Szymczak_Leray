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
mod szymczak_category;
mod util;

use crate::category::relation::*;
use bitvec::prelude::*;
use std::rc::Rc;

pub type Int = u16;
pub type TorsionCoeff = Vec<Int>;
pub const RECURSION_PARAMETER_SZYMCZAK_FUNCTOR: usize = 100; //idk xD

fn main() {}
