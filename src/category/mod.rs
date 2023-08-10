pub mod morphism;
pub mod relation;

#[allow(unused_imports)]
use crate::{
    category::{
        morphism::{EndoMorphism, Morphism},
        relation::Relation,
    },
    rmodule::{
        canon::{self, CanonModule},
        direct::DirectModule,
        map::CanonToCanon,
        ring::{Ring, SuperRing},
    },
    util::category_of_relations::calculate_helper_indices,
    Int, TorsionCoeff,
};
use std::{
    collections::HashMap,
    fmt::{self, Display},
    sync::Arc,
};

pub type HomSet<Object, M> = HashMap<Object, HashMap<Object, Vec<M>>>;

pub struct Category<Object: Eq, M: Morphism<Object, Object>> {
    pub hom_sets: HomSet<Object, M>,
}

impl<R: SuperRing> Category<CanonModule<R>, Relation<R>> {
    pub fn new(base: Int, max_dimension: Int) -> Self {
        todo!()
        /*
        let all_canon_rmodules: HashSet<Arc<CanonModule<R>>> =
            canon::all_torsion_coeffs(base, max_dimension)
                .into_iter()
                .map(CanonModule::<R>::new)
                .map(Arc::new)
                .collect();

        let hom_sets = all_canon_rmodules
            .iter()
            .map(|source| {
                (
                    source.as_ref().clone(),
                    all_canon_rmodules
                        .iter()
                        .map(|target| {
                            (
                                target.as_ref().clone(),
                                Self::hom_set(Arc::clone(&source), Arc::clone(&target)),
                            )
                        })
                        .collect::<HashMap<CanonModule<R>, Vec<Relation<R>>>>(),
                )
            })
            .collect::<HomSet<CanonModule<R>, Relation<R>>>();

        Category { hom_sets }
        */
    }

    fn hom_set(source: Arc<CanonModule<R>>, target: Arc<CanonModule<R>>) -> Vec<Relation<R>> {
        let direct = DirectModule::<R>::sumproduct(source, target);
        direct
            .submodules_goursat()
            .into_iter()
            .filter_map(|submodule| Relation::<R>::try_from((&direct, &submodule)).ok())
            .collect::<Vec<Relation<R>>>()
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
