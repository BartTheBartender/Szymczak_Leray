//pub mod map;
pub mod morphism;
pub mod relation;
// pub mod torsion_coeff;

use crate::{
    category::{
        morphism::{Endomorphism, Morphism},
        relation::Relation,
    },
    util::category_of_relations::calculate_helper_indices,
    zmodule::{
        canon::{self, CanonZModule},
        map::CanonToCanon,
    },
    Int, TorsionCoeff,
};
use std::{
    collections::{HashMap, HashSet},
    fmt::{self, Display},
    sync::Arc,
};

pub type HomSet<Object, M> = HashMap<Object, HashMap<Object, Vec<M>>>;

pub struct Category<Object: Eq, M: Morphism<Object, Object>> {
    pub hom_sets: HomSet<Object, M>,
}

impl Category<CanonZModule, Relation> {
    pub fn new(base: Int, max_dimension: Int) -> Self {
        let all_canon_zmodules: HashSet<Arc<CanonZModule>> =
            canon::all_torsion_coeffs(base, max_dimension)
                .into_iter()
                .map(CanonZModule::new)
                .map(Arc::new)
                .collect();

        let hom_sets = all_canon_zmodules
            .iter()
            .map(|source| {
                (
                    source.as_ref().clone(),
                    all_canon_zmodules
                        .iter()
                        .map(|target| {
                            (
                                target.as_ref().clone(),
                                Self::hom_set(Arc::clone(&source), Arc::clone(&target)),
                            )
                        })
                        .collect::<HashMap<CanonZModule, Vec<Relation>>>(),
                )
            })
            .collect::<HomSet<CanonZModule, Relation>>();

        Category { hom_sets }
    }

    fn hom_set(source: Arc<CanonZModule>, target: Arc<CanonZModule>) -> Vec<Relation> {
        let (helper_indices_normal, helper_indices_transposed, helper_capacity) =
            calculate_helper_indices(source.as_ref(), target.as_ref());

        CanonZModule::product(source.as_ref().clone(), target.as_ref().clone())
            .submodules()
            .into_iter()
            .map(|canon_to_canon| canon_to_canon.image())
            .map(|elements| {
                Relation::new_unchecked(
                    elements,
                    &helper_indices_normal,
                    &helper_indices_transposed,
                    &helper_capacity,
                    Arc::clone(&source),
                    Arc::clone(&target),
                )
            })
            .collect::<Vec<Relation>>()
    }
}

impl<Object: Eq + Display, M: Morphism<Object, Object> + Display> fmt::Display
    for Category<Object, M>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut string = String::new();

        for (source, hom_sets_fixed_object) in self.hom_sets.iter() {
            for (target, morphisms) in hom_sets_fixed_object.iter() {
                string.push_str(
                    &[
                        "source:",
                        &source.to_string(),
                        "target:",
                        &target.to_string(),
                        "\n",
                    ]
                    .join(" "),
                );
                for morphism in morphisms.iter() {
                    string.push_str(&[&morphism.to_string(), "\n"].join(""));
                }
            }
        }

        write!(f, "{}", string)
    }
}
