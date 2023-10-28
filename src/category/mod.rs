use crate::{
    category::{
        morphism::{Enumerable as EnumerableMorphism, Morphism},
        object::{
            Duplicable as DuplicableObject, Object,
            PartiallyEnumerable as PartiallyEnumerableObject,
        },
    },
    Int,
};
use std::{collections::HashMap, fmt, hash::Hash, sync::Arc};

pub mod functors;
pub mod morphism;
pub mod object;
pub mod relation;

pub type HomSet<Object, M> = HashMap<Object, HashMap<Object, Vec<M>>>;

#[derive(Clone)]
pub struct Category<O: Object, M: Morphism<O>> {
    pub hom_sets: HomSet<O, M>,
}

impl<
        O: Object + Hash + Clone + PartiallyEnumerableObject + DuplicableObject + fmt::Debug,
        M: Morphism<O, B = Arc<O>> + EnumerableMorphism<O> + Clone + fmt::Debug,
    > Category<O, M>
{
    pub fn new(maximal_dimension: Int) -> Self {
        let all_objects: Vec<O> = O::all_by_dimension(0..=maximal_dimension.into()).collect();

        let all_sources: Vec<Arc<O>> = all_objects
            .iter()
            .map(|object| Arc::new(object.duplicate()))
            .collect();

        let all_targets: Vec<Arc<O>> = all_objects
            .into_iter()
            .map(|object| Arc::new(object))
            .collect();

        let hom_sets = all_sources
            .iter()
            .map(|source: &Arc<O>| {
                let hom_sets_fixed_source: HashMap<O, Vec<M>> = all_targets
                    .iter()
                    .map(|target: &Arc<O>| {
                        let hom_set_fixed_source_and_target =
                            M::hom(Arc::clone(source), Arc::clone(target));
                        (
                            target.as_ref().clone(),
                            hom_set_fixed_source_and_target.collect(),
                        )
                    })
                    .collect::<HashMap<O, Vec<M>>>();
                (source.as_ref().clone(), hom_sets_fixed_source)
            })
            .collect::<HomSet<O, M>>();

        Self { hom_sets }
    }

    //why cant i use iterator?
    pub fn into_objects(self) -> Vec<O> {
        self.hom_sets.into_keys().collect::<Vec<O>>()
    }

    //why cant i use iterators? how to enforce the functions to take ownership?
    pub fn into_morphisms(self) -> Vec<M> {
        self.hom_sets
            .into_values()
            .fold(Vec::<M>::new(), |mut morphisms, hom_sets_fixed_source| {
                let mut morphisms_fixed_source = hom_sets_fixed_source.into_values().fold(
                    Vec::<M>::new(),
                    |mut morphisms_fixed_source, mut morphisms_fixed_source_and_target| {
                        morphisms_fixed_source.append(&mut morphisms_fixed_source_and_target);
                        morphisms_fixed_source
                    },
                );
                morphisms.append(&mut morphisms_fixed_source);
                morphisms
            })
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

impl<O: Object + fmt::Debug, M: Morphism<O> + fmt::Debug> fmt::Debug for Category<O, M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut string = String::new();

        for (source, hom_sets_fixed_object) in &self.hom_sets {
            for (target, morphisms) in hom_sets_fixed_object {
                string.push_str(
                    &[
                        "s:",
                        &format!("{source:?}"),
                        "t:",
                        &format!("{target:?}"),
                        "\n",
                    ]
                    .join(" "),
                );
                for morphism in morphisms {
                    string.push_str(&[&format!("{morphism:?}",), "\n"].join(""));
                }
            }
        }

        write!(f, "{string}")
    }
}

pub trait PrettyName {
    const PRETTY_NAME: &'static str;
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        category::relation::{CanonModule, Relation},
        ralg::cgroup::{ideal::CIdeal, C},
    };

    #[test]
    fn objects() {
        use typenum::U5 as N;
        type R = C<N>;
        type I = CIdeal<N>;
        let category = Category::<CanonModule<R, I>, Relation<R, I>>::new(1);

        assert_eq!(category.into_objects().len(), 2);
    }
}
