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
    fmt::{self, Debug, Display},
    hash::Hash,
    sync::Arc,
};

pub type HomSet<Object, M> = HashMap<Object, HashMap<Object, Vec<M>>>;

#[derive(Clone)]
pub struct Category<Object: Eq, M: Morphism<Object, Object>> {
    pub hom_sets: HomSet<Object, M>,
}

pub trait AllObjects: Sized + Eq + PartialEq + Send + Sync {
    fn all_objects(maximal_dimension: Int) -> Vec<Self>;
}

pub trait AllMorphisms<Object: AllObjects>: Sized {
    fn hom_set(source: Arc<Object>, target: Arc<Object>) -> Vec<Self>;
}

pub trait Duplicate {
    /**
    returns a module isomorphic to self,
    but with *different* coefficient uuids
    */
    fn duplicate(&self) -> Self;
}

impl<
        Object: Eq + PartialEq + Hash + Clone + AllObjects + Duplicate,
        M: Morphism<Object, Object> + AllMorphisms<Object> + Clone,
    > Category<Object, M>
{
    pub fn new(maximal_dimension: Int) -> Self {
        let all_objects: Vec<Object> = AllObjects::all_objects(maximal_dimension);

        let all_sources: Vec<Arc<Object>> = all_objects
            .iter()
            .map(|object| Arc::new(object.clone()))
            .collect();

        let all_targets: Vec<Arc<Object>> = all_objects
            .iter()
            .map(|object| Arc::new(object.duplicate()))
            .collect();

        let hom_sets = all_sources
            .iter()
            .map(|source: &Arc<Object>| {
                let hom_sets_fixed_source: HashMap<Object, Vec<M>> = all_targets
                    .iter()
                    .map(|target: &Arc<Object>| {
                        let hom_set = M::hom_set(Arc::clone(source), Arc::clone(target));
                        (target.as_ref().clone(), hom_set)
                    })
                    .collect::<HashMap<Object, Vec<M>>>();
                (source.as_ref().clone(), hom_sets_fixed_source)
            })
            .collect::<HomSet<Object, M>>();

        Category { hom_sets }
    }

    //why cant i use iterator?
    pub fn objects(self) -> Vec<Object> {
        self.hom_sets.into_keys().collect::<Vec<Object>>()
    }

    //why cant i use iterators?
    pub fn morphisms(self) -> Vec<M> {
        todo!()
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
