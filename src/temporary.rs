#[allow(unused_imports)]
use crate::{
    category::{relation::Relation, Category},
    rmodule::{canon::CanonModule, map::CanonToCanon, ring::Fin, torsion::CoeffTree},
};
use typenum::*;
type R = Fin<U2>;

pub fn temporary_z2_category() -> Category<CanonModule<R>, Relation<R>> {
    let all_torsion_coeffs = CoeffTree::all_torsion_coeff(1);

    todo!()
}
