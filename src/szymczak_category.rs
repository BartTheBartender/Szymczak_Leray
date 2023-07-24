use crate::{
    category::{
        morphism::{EndoMorphism, Morphism},
        Category,
    },
    RECURSION_PARAMETER_SZYMCZAK_FUNCTOR,
};
use std::{collections, hash::Hash};
type SzymczakClassesWithCyclesFixedBase<E> =
    collections::HashSet<collections::HashSet<collections::HashMap<E, Vec<E>>>>;

pub struct SzymczakCategory<Object: Eq, E: EndoMorphism<Object>> {
    szymczak_classes: collections::HashSet<collections::HashMap<Object, collections::HashSet<E>>>,
}

impl<Object: Eq + PartialEq + Hash, E: EndoMorphism<Object>> SzymczakCategory<Object, E> {
    //i dont really know if name of the function below is correct, but i find it cool (i woudl like to know your opinion as well). I dont know also how to parallelize it (i assume that we will have to use parallel Hash structures by rayon)
    pub fn szymczak_functor<M: Morphism<Object, Object>>(category: Category<Object, M>) -> Self {
        //first, copy the endomorphisms (we will need them to be owned)
        let all_endomorphisms: collections::HashMap<Object, collections::HashSet<E>> =
            category.all_endomorphisms::<E>();
        //next, generate their orbits
        let all_endomorphisms_with_cycles: collections::HashMap<
            Object,
            collections::HashSet<collections::HashMap<E, Vec<E>>>,
        > = all_endomorphisms
            .into_iter()
            .map(|(base_object, endomorphisms_fixed_base)| {
                (
                    base_object,
                    Self::cycles_fixed_base(endomorphisms_fixed_base),
                )
            })
            .collect();

        //next, partition the hom-sets of endomorphisms (with the cycles) into szymczak classes
        let all_szymczak_classes_with_cycles_fixed_bases: collections::HashMap<
            Object,
            SzymczakClassesWithCyclesFixedBase<E>,
        > = all_endomorphisms_with_cycles
            .into_iter()
            .map(|(base_object, endomorphisms_with_cycles_fixed_base)| {
                (
                    base_object,
                    Self::szymczak_functor_fixed_base(
                        endomorphisms_with_cycles_fixed_base,
                        &category.hom_sets,
                    ),
                )
            })
            .collect();

        //finally merge all partitions into szymczak classes (and drop the cycles ofc)
        todo!();
    }

    fn cycles_fixed_base(
        endomorphisms_fixed_base: collections::HashSet<E>,
    ) -> collections::HashSet<collections::HashMap<E, Vec<E>>> {
        todo!()
    }

    fn szymczak_functor_fixed_base<M: Morphism<Object, Object>>(
        endomorphisms_with_cycles_fixed_base: collections::HashSet<collections::HashMap<E, Vec<E>>>,
        hom_sets: &collections::HashMap<(Object, Object), collections::HashSet<M>>,
    ) -> SzymczakClassesWithCyclesFixedBase<E> {
        if endomorphisms_with_cycles_fixed_base.len() > RECURSION_PARAMETER_SZYMCZAK_FUNCTOR {
            let mut left_endomorphisms_with_cycles_fixed_base: collections::HashSet<
                collections::HashMap<E, Vec<E>>,
            > = collections::HashSet::new();
            let mut right_endomorphisms_with_cycles_fixed_base: collections::HashSet<
                collections::HashMap<E, Vec<E>>,
            > = collections::HashSet::new();

            //split!!! todo!()

            let left_szymczak_classes_with_cycles_fixed_base = Self::szymczak_functor_fixed_base(
                left_endomorphisms_with_cycles_fixed_base,
                hom_sets,
            );
            let right_szymczak_classes_with_cycles_fixed_base = Self::szymczak_functor_fixed_base(
                right_endomorphisms_with_cycles_fixed_base,
                hom_sets,
            );
            Self::merge_szymczak_classes_fixed_base(
                left_szymczak_classes_with_cycles_fixed_base,
                right_szymczak_classes_with_cycles_fixed_base,
            )
        } else {
            todo!()
        }
    }

    fn merge_szymczak_classes_fixed_base(
        left_szymczak_classes_with_cycles_fixed_base: SzymczakClassesWithCyclesFixedBase<E>,
        right_szymczak_classes_with_cycles_fixed_base: SzymczakClassesWithCyclesFixedBase<E> //add hom_sets
    ) -> SzymczakClassesWithCyclesFixedBase<E> {
        todo!()
    }

    fn merge_szymczak_classes<M : Morphism<Object, Object>>(all_szymczak_classes_with_cycles_fixed_base: collections::HashMap<Object, SzymczakClassesWithCyclesFixedBase, hom_sets: &collections::HashMap<(Object, Object), collections::HashSet<M>>) -> Self {todo!()}
}
