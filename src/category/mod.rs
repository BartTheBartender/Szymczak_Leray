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
    Int,
};
use std::{
    collections::HashMap,
    fmt::{self, Display},
    hash::Hash,
    sync::Arc,
};

pub type HomSet<Object, M> = HashMap<Object, HashMap<Object, Vec<M>>>;

pub struct Category<Object: Eq, M: Morphism<Object, Object>> {
    pub hom_sets: HomSet<Object, M>,
}

pub trait AllObjects: Sized + Eq + PartialEq + Send + Sync {
    fn all_objects(maximal_dimension: Int) -> Vec<Self>;
}

pub trait AllMorphisms<Object: AllObjects>: Sized {
    fn hom_set(source: Arc<Object>, target: Arc<Object>) -> Vec<Self>;
}

impl<
        Object: Eq + PartialEq + Hash + Clone + AllObjects,
        M: Morphism<Object, Object> + AllMorphisms<Object>,
    > Category<Object, M>
{
    pub fn new(maximal_dimension: Int) -> Self {
        let all_objects: Vec<Arc<Object>> = AllObjects::all_objects(maximal_dimension)
            .into_iter()
            .map(|object| Arc::new(object))
            .collect();

        let hom_sets = all_objects
            .clone()
            .into_iter()
            .map(|source: Arc<Object>| {
                (
                    (*source).clone(),
                    all_objects
                        .clone()
                        .into_iter()
                        .map(|target: Arc<Object>| {
                            (
                                (*target).clone(),
                                M::hom_set(Arc::clone(&source), Arc::clone(&target)),
                            )
                        })
                        .collect::<HashMap<Object, Vec<M>>>(),
                )
            })
            .collect::<HomSet<Object, M>>();

        Category { hom_sets }
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
