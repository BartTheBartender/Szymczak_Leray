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

type EndomorphismsWithCyclesFixedObject<E> = Vec<HashMap<E, Vec<E>>>;
type EndomorphismsWithCycles<Object, E> = HashMap<Object, EndomorphismsWithCyclesFixedObject<E>>;

type SzymczakClassesWithCyclesFixedObject<E> = Vec<Vec<HashMap<E, Vec<E>>>>;
type SzymczakClassesWithCycles<Object, E> =
    HashMap<Object, SzymczakClassesWithCyclesFixedObject<E>>;

type SzymczakClassesFixedObject<E> = Vec<Vec<E>>;
type SzymczakClasses<Object, E> = HashMap<Object, SzymczakClassesFixedObject<E>>;

pub struct SzymczakCategory<Object: Eq, E: EndoMorphism<Object>> {
    szymczak_classes: SzymczakClasses<Object, E>,
}

impl<Object: Eq + PartialEq + Hash, E: EndoMorphism<Object>> SzymczakCategory<Object, E> {
    //i dont really know if name of the function below is correct, but i find it cool (i woudl like to know your opinion as well). I dont know also how to parallelize it (i assume that we will have to use parallel Hash structures by rayon)
    pub fn szymczak_functor<M: Morphism<Object, Object>>(category: Category<Object, M>) -> Self {
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
            let left_endomorphisms_with_cycles_fixed_object: EndomorphismsWithCyclesFixedObject<E> =
                endomorphisms_with_cycles_fixed_object
                    .split_off(endomorphisms_with_cycles_fixed_object.len() / 2);
            let right_endomorphisms_with_cycles_fixed_object =
                endomorphisms_with_cycles_fixed_object;

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
        endomorphisms_with_cycles_fixed_object: EndomorphismsWithCyclesFixedObject<E>,
        hom_sets: &HashMap<(Object, Object), Vec<M>>,
    ) -> SzymczakClassesWithCyclesFixedObject<E> {
        let mut szymczak_classes_with_cycles_fixed_object =
            SzymczakClassesWithCyclesFixedObject::<E>::new();

        //this construction was to complex for me to follow borrow checker rules at 1:00 pm
        szymczak_classes_with_cycles_fixed_object
    }

    fn merge_szymczak_classes_fixed_object<M: Morphism<Object, Object>>(
        mut left_szymczak_classes_with_cycles_fixed_object: SzymczakClassesWithCyclesFixedObject<E>,
        mut right_szymczak_classes_with_cycles_fixed_object: SzymczakClassesWithCyclesFixedObject<
            E,
        >,
        hom_sets: &HashMap<(Object, Object), Vec<M>>,
    ) -> SzymczakClassesWithCyclesFixedObject<E> {
        let mut output_szymczak_classes_with_cycles_fixed_object =
            SzymczakClassesWithCyclesFixedObject::<E>::new();
        /* help plz
        for mut left_szymczak_class_with_cycles_fixed_object in
            left_szymczak_classes_with_cycles_fixed_object.iter_mut()
        {
            for mut right_szymczak_class_with_cycles_fixed_object in
                right_szymczak_classes_with_cycles_fixed_object.iter_mut()
            {
                if Self::are_szymczak_isomorphic(
                    left_szymczak_class_with_cycles_fixed_object
                        .first()
                        .unwrap(),
                    right_szymczak_class_with_cycles_fixed_object
                        .first()
                        .unwrap(),
                    hom_sets,
                ) {
                    left_szymczak_class_with_cycles_fixed_object
                        .extend(right_szymczak_class_with_cycles_fixed_object);

                    output_szymczak_classes_with_cycles_fixed_object
                        .push(right_szymczak_class_with_cycles_fixed_object);
                }
            }
        }

        output_szymczak_classes_with_cycles_fixed_object
            .extend(left_szymczak_classes_with_cycles_fixed_object)
            .extend(right_szymczak_classes_with_cycles_fixed_object);
        */
        output_szymczak_classes_with_cycles_fixed_object
    }

    fn merge_szymczak_classes<M: Morphism<Object, Object>>(
        mut szymczak_clases_with_cycles: SzymczakClassesWithCycles<Object, E>,
    ) -> SzymczakClasses<Object, E> {
        todo!()
    }

    fn are_szymczak_isomorphic<M: Morphism<Object, Object>>(
        left_endomorphism_with_cycles: &HashMap<E, Vec<E>>,
        right_endomorphism_with_cycles: &HashMap<E, Vec<E>>,
        hom_sets: &HashMap<(Object, Object), Vec<M>>,
    ) -> bool {
        todo!() // a lot!
    }
}
