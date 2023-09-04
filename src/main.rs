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
/* clippy begin */
#![warn(
    // regular groups
    clippy::all, // just in case
    clippy::nursery,
    clippy::pedantic,
    clippy::style,
    clippy::complexity,
    clippy::perf,

    // debugging remnants
    clippy::dbg_macro,
    clippy::expect_used,
    clippy::panic,
    clippy::print_stderr,
    clippy::print_stdout,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::use_debug,
    clippy::unwrap_used,

    // restricions
    clippy::arithmetic_side_effects,
    clippy::clone_on_ref_ptr,
    clippy::else_if_without_else,
    clippy::float_cmp_const,
    clippy::fn_to_numeric_cast_any,
    clippy::if_then_some_else_none,
    clippy::let_underscore_must_use,
    clippy::map_err_ignore,
    clippy::missing_assert_message,
    clippy::multiple_inherent_impl,
    clippy::multiple_unsafe_ops_per_block,
    clippy::mutex_atomic,
    clippy::pattern_type_mismatch,
    clippy::rc_buffer,
    clippy::rc_mutex,
    clippy::same_name_method,
    clippy::shadow_reuse,
    clippy::shadow_same,
    clippy::shadow_unrelated,
    clippy::single_char_lifetime_names,
    clippy::str_to_string,
    clippy::string_slice,
    clippy::string_to_string,
    clippy::verbose_file_reads,

    // style
    clippy::decimal_literal_representation,
    clippy::format_push_string,
    clippy::tests_outside_test_module,
)]
#![deny(
    clippy::correctness,

    // restrictions
    clippy::as_conversions,
    clippy::allow_attributes_without_reason,
    clippy::default_numeric_fallback,
    clippy::exit,
    clippy::indexing_slicing,
    clippy::lossy_float_literal,
    clippy::mem_forget,
    clippy::string_add,
    clippy::try_err,

    // style
    clippy::empty_structs_with_brackets,
    clippy::impl_trait_in_params,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::self_named_module_files,
    clippy::semicolon_inside_block,
    clippy::unnecessary_self_imports,
    clippy::unneeded_field_pattern,
    clippy::unseparated_literal_suffix,
)]
#![allow(clippy::match_bool)]
/* clippy end */

mod category;
mod error;
mod rmodule;
mod szymczak_category;
mod util;

#[allow(unused_imports)]
use crate::{
    category::{relation::Relation, Category},
    rmodule::{canon::CanonModule, map::CanonToCanon, ring::Fin, torsion::CoeffTree},
    szymczak_category::SzymczakCategory,
};
use std::time::{Duration, Instant};

//parameters for the code
use typenum::U3 as N;
pub const MAX_DIMENSION: Int = 2;
pub type Int = u16;

type R = Fin<N>;
fn main() {
    let category_time = Instant::now();
    let category = Category::<CanonModule<R>, Relation<R>>::new(MAX_DIMENSION);
    println!("category generated after {:?}", category_time.elapsed());
    let szymczak_classes_time = Instant::now();
    let szymczak_category = SzymczakCategory::szymczak_functor::<20>(&category);
    println!(
        "szymczak classes generated after {:?}",
        szymczak_classes_time.elapsed()
    );
}
