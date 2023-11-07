#![feature(specialization)]
#![feature(return_position_impl_trait_in_trait)]
#![feature(generic_const_exprs)]
#![feature(let_chains)]
#![feature(btree_extract_if)]
#![feature(iterator_try_collect)]
#![feature(extract_if)]
#![feature(trait_alias)]
#![feature(associated_type_bounds)]
#![feature(arc_unwrap_or_clone)]
#![feature(lint_reasons)]
/* this feature is necessary to constrain matrices,
however, a bug inside it prevents using type aliases for other types
*/
// #![feature(lazy_type_alias)]
#![feature(slice_group_by)]
// visual separator
#![allow(incomplete_features, reason = "we need nightly features")]
#![allow(dead_code, reason = "to be removed later")] // REMOVE THIS LATER

// - - -
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
    //clippy::expect_used,
    //clippy::panic,
    clippy::print_stderr,
    //clippy::print_stdout,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::use_debug,
    //clippy::unwrap_used,

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
#![allow(clippy::match_bool, reason = "i find it more readable")]
#![allow(clippy::module_name_repetitions, reason = "this is a dumb rule")]
/* clippy end */

mod category;
mod ralg;
mod util;

// - - -

use crate::{
    category::{
        functors::szymczak::{SzymczakClasses, SzymczakClassesFull},
        relation::Relation,
        Category,
    },
    ralg::{
        cgroup::{ideal::CIdeal, C},
        module::canon::object::Object as Module,
    },
};
use std::{fs, time::Instant};
// parameters for the code
use typenum::{Unsigned, U4 as N};
type Int = u16;
type R = C<N>;
type I = CIdeal<N>;
const DIM: Int = 2;
const RECURSION_PARAMETER: usize = 8;

fn main() -> std::io::Result<()> {
    //
    let category_time = Instant::now();
    let category = Category::<Module<R, I>, Relation<R, I>>::new(DIM);
    let category_time_elapsed = category_time.elapsed();

    let szymczak_classes_time = Instant::now();
    let szymczak_classes = SzymczakClasses::<Module<R, I>, Relation<R, I>>::functor::<
        { RECURSION_PARAMETER },
    >(&category);
    let szymczak_classes_time_elapsed = szymczak_classes_time.elapsed();

    //warning: it is assumed that the file is run from directory "szymczak_leray"
    fs::write(format!("results/szymczak-wide/txt/dim{}/Z{}-dim-{}", DIM, N::to_usize(), DIM), format!("{}===\nCategory generated after: {}\nIsomorphisms classes generated after: {}\nParameter of the recursion: {}\n", szymczak_classes, category_time_elapsed.as_secs_f64(), szymczak_classes_time_elapsed.as_secs_f64(), RECURSION_PARAMETER))?;

    let szymczak_classes_full_time = Instant::now();
    let szymczak_classes_full =
        SzymczakClassesFull::<Module<R, I>, Relation<R, I>>::all_isos(szymczak_classes, &category);
    let szymczak_classes_full_time_elapsed = szymczak_classes_full_time.elapsed();

    fs::write(format!("results/szymczak-wide-full/txt/dim{}/Z{}-dim-{}", DIM, N::to_usize(), DIM), format!("{}===\nCategory generated after: {}\nIsomorphisms classes generated after: {}\nAll isomorphisms added after: {}\nParameter of the recursion: {}\n", szymczak_classes_full, category_time_elapsed.as_secs_f64(), szymczak_classes_time_elapsed.as_secs_f64(), szymczak_classes_full_time_elapsed.as_secs_f64(), RECURSION_PARAMETER))?;

    println!("Category generated after: {}\nIsomorphisms classes generated after: {}\nAll isomorphisms added after: {}\nParameter of the recursion: {}", category_time_elapsed.as_secs_f64(), szymczak_classes_time_elapsed.as_secs_f64(), szymczak_classes_full_time_elapsed.as_secs_f64(), RECURSION_PARAMETER);

    Ok(())
}
