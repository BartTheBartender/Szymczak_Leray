use crate::{
    category::{
        morphism::{EndoMorphism, Morphism},
        Category,
    },
    RECURSION_PARAMETER_SZYMCZAK_FUNCTOR,
};
use std::{collections::HashMap, hash::Hash};

type EndomorphismsFixedObject<E> = Vec<E>;
type Endomorphisms<Object, E> = HashMap<Object, EndomorphismsFixedObject<E>>;

type EndomorphismsWithCyclesFixedObject<E> = HashMap<E, Vec<E>>;
type EndomorphismsWithCycles<Object, E> = HashMap<Object, EndomorphismsWithCyclesFixedObject<E>>;

type SzymczakClassWithCyclesFixedObject<E> = HashMap<E, Vec<E>>;
type SzymczakClassesWithCyclesFixedObject<E> = Vec<SzymczakClassWithCyclesFixedObject<E>>;
type SzymczakClassesWithCycles<Object, E> =
    HashMap<Object, SzymczakClassesWithCyclesFixedObject<E>>;

type SzymczakClassFixedObject<E> = Vec<E>;
type SzymczakClassesFixedObject<E> = Vec<SzymczakClassFixedObject<E>>;
type SzymczakClasses<Object, E> = HashMap<Object, SzymczakClassesFixedObject<E>>;

pub struct SzymczakCategory<Object: Eq, E: EndoMorphism<Object>> {
    szymczak_classes: SzymczakClasses<Object, E>,
}

impl<Object: Eq + PartialEq + Hash, E: EndoMorphism<Object>> SzymczakCategory<Object, E> {
    //i dont really know if name of the function below is correct, but i find it cool (i woudl like to know your opinion as well). I dont know also how to parallelize it (i assume that we will have to use parallel Hash structures by rayon)
    pub fn szymczak_functor<M: Morphism<Object, Object>>(category: &Category<Object, M>) -> Self {
        //first, clone the endomorphisms (we will need them to be owned)
        let endomorphisms: Endomorphisms<Object, E> = Self::endomorphisms(&category.hom_sets);
        //next, generate their orbits
        let endomorphisms_with_cycles: EndomorphismsWithCycles<Object, E> = endomorphisms
            .into_iter()
            .map(|(object, endomorphisms_fixed_object)| {
                (
                    object,
                    Self::cycles_fixed_object(endomorphisms_fixed_object),
                )
            })
            .collect();

        //next, partition the hom-sets of endomorphisms (with the cycles) into szymczak classes (keeping the cycles for the final merge)
        let szymczak_classes_with_cycles: SzymczakClassesWithCycles<Object, E> =
            endomorphisms_with_cycles
                .into_iter()
                .map(|(object, szymczak_classes_with_cycles_fixed_object)| {
                    (
                        object,
                        Self::szymczak_functor_fixed_object(
                            szymczak_classes_with_cycles_fixed_object,
                            &category.hom_sets,
                        ),
                    )
                })
                .collect();

        //finally merge all partitions into szymczak classes (and drop the cycles ofc)
        SzymczakCategory {
            szymczak_classes: Self::merge_szymczak_classes::<M>(szymczak_classes_with_cycles),
        }
    }

    fn endomorphisms<M: Morphism<Object, Object>>(
        hom_sets: &HashMap<(Object, Object), Vec<M>>,
    ) -> Endomorphisms<Object, E> {
        /* plz help
        hom_sets
            .iter()
            .filter(|((source, target), _)| source == target)
            .flat_map(|((source, _), morphisms)| {
                morphisms.iter().map(move |_| (source.clone(), todo!())) //cast M into E and clone
            })
            .collect::<Endomorphisms<Object, E>>();*/

        todo!()
    }

    fn cycles_fixed_object(
        //idk if we really need it
        endomorphisms_fixed_object: EndomorphismsFixedObject<E>,
    ) -> EndomorphismsWithCyclesFixedObject<E> {
        todo!()
    }

    fn szymczak_functor_fixed_object<M: Morphism<Object, Object>>(
        mut endomorphisms_with_cycles_fixed_object: EndomorphismsWithCyclesFixedObject<E>,
        hom_sets: &HashMap<(Object, Object), Vec<M>>,
    ) -> SzymczakClassesWithCyclesFixedObject<E> {
        if endomorphisms_with_cycles_fixed_object.len() > RECURSION_PARAMETER_SZYMCZAK_FUNCTOR {
            let (
                left_endomorphisms_with_cycles_fixed_object,
                right_endomorphisms_with_cycles_fixed_object,
            ) = Self::split_in_half(endomorphisms_with_cycles_fixed_object);

            let left_szymczak_classes_with_cycles_fixed_object =
                Self::szymczak_functor_fixed_object(
                    left_endomorphisms_with_cycles_fixed_object,
                    hom_sets,
                );
            let right_szymczak_classes_with_cycles_fixed_object =
                Self::szymczak_functor_fixed_object(
                    right_endomorphisms_with_cycles_fixed_object,
                    hom_sets,
                );

            Self::merge_szymczak_classes_fixed_object(
                left_szymczak_classes_with_cycles_fixed_object,
                right_szymczak_classes_with_cycles_fixed_object,
                hom_sets,
            )
        } else {
            Self::szymczak_functor_fixed_object_final_step(
                endomorphisms_with_cycles_fixed_object,
                hom_sets,
            )
        }
    }

    fn szymczak_functor_fixed_object_final_step<M: Morphism<Object, Object>>(
        mut endomorphisms_with_cycles_fixed_object: EndomorphismsWithCyclesFixedObject<E>,
        hom_sets: &HashMap<(Object, Object), Vec<M>>,
    ) -> SzymczakClassesWithCyclesFixedObject<E> {
        let mut szymczak_classes_with_cycles_fixed_object =
            SzymczakClassesWithCyclesFixedObject::<E>::new();

        for endomorphism_with_cycles in endomorphisms_with_cycles_fixed_object {
            let maybe_szymczak_class_with_cycles_fixed_object: Option<
                &mut SzymczakClassWithCyclesFixedObject<E>,
            > = szymczak_classes_with_cycles_fixed_object.iter_mut().find(
                |szymczak_class_with_cycles_fixed_object| {
                    Self::are_szymczak_isomorphic(
                        (&endomorphism_with_cycles.0, &endomorphism_with_cycles.1),
                        szymczak_class_with_cycles_fixed_object
                            .iter()
                            .next()
                            .expect("szymczak classes are non-empty"),
                        hom_sets,
                    )
                },
            );

            if let Some(szymczak_class_with_cycles_fixed_object) =
                maybe_szymczak_class_with_cycles_fixed_object
            {
                //i hope it works as intended - it modifies the output
                szymczak_class_with_cycles_fixed_object
                    .insert(endomorphism_with_cycles.0, endomorphism_with_cycles.1);
            } else {
                let mut new_szymczak_class_with_cycles_fixed_object =
                    SzymczakClassWithCyclesFixedObject::<E>::new();

                new_szymczak_class_with_cycles_fixed_object
                    .insert(endomorphism_with_cycles.0, endomorphism_with_cycles.1);
                szymczak_classes_with_cycles_fixed_object
                    .push(new_szymczak_class_with_cycles_fixed_object);
            }
        }

        szymczak_classes_with_cycles_fixed_object
    }

    fn merge_szymczak_classes_fixed_object<M: Morphism<Object, Object>>(
        mut left_szymczak_classes_with_cycles_fixed_object: SzymczakClassesWithCyclesFixedObject<E>,
        mut right_szymczak_classes_with_cycles_fixed_object: SzymczakClassesWithCyclesFixedObject<
            E,
        >,
        hom_sets: &HashMap<(Object, Object), Vec<M>>,
    ) -> SzymczakClassesWithCyclesFixedObject<E> {
        let mut merged_szymczak_classes_with_cycles_fixed_object =
            SzymczakClassesWithCyclesFixedObject::<E>::new();

        //lengths must be updated every time a pair is merged (a bit unsafe construction, but it works properly)
        let mut left_index = 0;
        'left_loop: while left_index < left_szymczak_classes_with_cycles_fixed_object.len() {
            let mut right_index = 0;
            while right_index < right_szymczak_classes_with_cycles_fixed_object.len() {
                if Self::are_szymczak_isomorphic(
                    left_szymczak_classes_with_cycles_fixed_object[left_index]
                        .iter()
                        .next()
                        .expect("szymczak classes in merging are non-empty"),
                    right_szymczak_classes_with_cycles_fixed_object[right_index]
                        .iter()
                        .next()
                        .expect("szymczak classes in merging are non-empty"),
                    hom_sets,
                ) {
                    //a bit of syntatic sugar (i know it is possibly a bit slower than extend)
                    let merged_szymczak_class_with_cycles_fixed_object: SzymczakClassWithCyclesFixedObject<E> = Self::merge_hashmaps(
                    left_szymczak_classes_with_cycles_fixed_object.swap_remove(left_index),
                    right_szymczak_classes_with_cycles_fixed_object.swap_remove(right_index));
                    merged_szymczak_classes_with_cycles_fixed_object
                        .push(merged_szymczak_class_with_cycles_fixed_object);

                    continue 'left_loop;
                }
                right_index += 1;
            }
            left_index += 1;
        }

        //merge possible classes not paired before
        merged_szymczak_classes_with_cycles_fixed_object
            .extend(left_szymczak_classes_with_cycles_fixed_object);
        merged_szymczak_classes_with_cycles_fixed_object
            .extend(right_szymczak_classes_with_cycles_fixed_object);

        merged_szymczak_classes_with_cycles_fixed_object
    }

    fn merge_szymczak_classes<M: Morphism<Object, Object>>(
        mut szymczak_classes_with_cycles: SzymczakClassesWithCycles<Object, E>,
    ) -> SzymczakClasses<Object, E> {
        let mut merged_szymczak_classes_with_cycles = SzymczakClassesWithCycles::<Object, E>::new();
        Self::drop_cycles(merged_szymczak_classes_with_cycles)
    }

    fn merge_hashmaps<K: Eq + PartialEq + Hash, V: Sized>(
        mut left_hashmap: HashMap<K, V>,
        mut right_vector: HashMap<K, V>,
    ) -> HashMap<K, V> {
        todo!()
    }

    fn drop_cycles(
        mut szymczak_classes_with_cycles: SzymczakClassesWithCycles<Object, E>,
    ) -> SzymczakClasses<Object, E> {
        todo!()
    }

    fn split_in_half(
        endomorphisms_with_cycles_fixed_object: EndomorphismsWithCyclesFixedObject<E>,
    ) -> (
        EndomorphismsWithCyclesFixedObject<E>,
        EndomorphismsWithCyclesFixedObject<E>,
    ) {
        todo!()
    }

    fn are_szymczak_isomorphic<M: Morphism<Object, Object>>(
        left_endomorphism_with_cycles: (&E, &Vec<E>),
        right_endomorphism_with_cycles: (&E, &Vec<E>),
        hom_sets: &HashMap<(Object, Object), Vec<M>>,
    ) -> bool {
        todo!() // a lot!
    }
}
