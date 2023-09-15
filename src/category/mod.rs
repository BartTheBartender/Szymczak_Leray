use crate::{
    category::{
        morphism::{Enumerable as EnumerableMorphism, Morphism},
        object::{Object, PartiallyEnumerable as PartiallyEnumerableObject},
    },
    Int,
};
use std::{collections::HashMap, fmt, hash::Hash, sync::Arc};

pub mod morphism;
pub mod object;

pub type HomSet<Object, M> = HashMap<Object, HashMap<Object, Vec<M>>>;

#[derive(Clone)]
pub struct Container<O: Object, M: Morphism<O>> {
    pub hom_sets: HomSet<O, M>,
}

impl<
        O: Object + Hash + Clone + PartiallyEnumerableObject,
        M: Morphism<O, B = Arc<O>> + EnumerableMorphism<O> + Clone,
    > Container<O, M>
{
    pub fn new(maximal_dimension: Int) -> Self {
        let all_objects: Vec<O> = O::all_by_dimension(0..maximal_dimension.into()).collect();

        let all_sources: Vec<Arc<O>> = all_objects
            .iter()
            .map(|object| Arc::new(object.clone()))
            .collect();

        let all_targets: Vec<Arc<O>> = all_objects
            .iter()
            // .map(|object| Arc::new(object.duplicate())) // na chuj tutaj duplikować?
            .map(|object| Arc::new(object.clone()))
            .collect();

        let hom_sets = all_sources
            .iter()
            .map(|source: &Arc<O>| {
                let hom_sets_fixed_source: HashMap<O, Vec<M>> = all_targets
                    .iter()
                    .map(|target: &Arc<O>| {
                        let hom_set = M::hom(Arc::clone(source), Arc::clone(target));
                        (target.as_ref().clone(), hom_set.collect())
                    })
                    .collect::<HashMap<O, Vec<M>>>();
                (source.as_ref().clone(), hom_sets_fixed_source)
            })
            .collect::<HomSet<O, M>>();

        Self { hom_sets }
    }

    //why cant i use iterator?
    pub fn objects(self) -> Vec<O> {
        self.hom_sets.into_keys().collect::<Vec<O>>()
    }

    //why cant i use iterators?
    pub fn morphisms(self) -> Vec<M> {
        todo!()
    }

    pub fn hom_set(&self, source: &O, target: &O) -> Vec<M> {
        self.hom_sets
            .get(source)
            .expect("source should be an object in the category")
            .get(target)
            .expect("target should be an object in the category")
            .to_vec()
    }
}

impl<O: Object + fmt::Display, M: Morphism<O> + fmt::Display> fmt::Display for Container<O, M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut string = String::new();

        for (source, hom_sets_fixed_object) in &self.hom_sets {
            for (target, morphisms) in hom_sets_fixed_object {
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
                for morphism in morphisms {
                    string.push_str(&[&morphism.to_string(), "\n"].join(""));
                }
            }
        }

        write!(f, "{string}")
    }
}
