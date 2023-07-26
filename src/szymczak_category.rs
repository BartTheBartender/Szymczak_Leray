use crate::{
    category::{
        morphism::{self, Endomorphism, Morphism},
        Category,
    },
    RECURSION_PARAMETER_SZYMCZAK_FUNCTOR,
};
use std::{collections::HashMap, hash::Hash};

type Endomorphisms<E> = Vec<E>;
type EndomorphismsWithCycles<E> = HashMap<E, Vec<E>>;

type RawSzymczakClass<E> = HashMap<E, Vec<E>>;
type RawSzymczakClasses<E> = Vec<RawSzymczakClass<E>>;

type SzymczakClass<Object, E> = HashMap<Object, Vec<E>>;
type SzymczakClasses<Object, E> = Vec<SzymczakClass<Object, E>>;

pub struct SzymczakCategory<Object: Eq, E: Endomorphism<Object>> {
    szymczak_classes: SzymczakClasses<Object, E>,
}

impl<Object: Eq + PartialEq + Hash + Clone, E: Endomorphism<Object>> SzymczakCategory<Object, E> {
    //i dont really know if name of the function below is correct, but i find it cool (i woudl like to know your opinion as well). I dont know also how to parallelize it (i assume that we will have to use parallel Hash structures by rayon)
    pub fn szymczak_functor<M: Morphism<Object, Object>>(category: &Category<Object, M>) -> Self {
        //step 1. Clone all the endomorphisms (we will need them to be owned)

        let endomorphisms: Endomorphisms<E> = category
            .hom_sets
            .iter()
            .filter(|((source, target), _)| source == target)
            .flat_map(|(_, morphisms)| {
                morphisms
                    .iter()
                    .map(|morphism| Endomorphism::from_morphism(morphism))
            })
            .collect();

        //step 2. generate raw szymczak classes (by raw i mean they are unsorted and endomorphisms keep their cycles)
        let raw_szymczak_classes = Self::raw_szymczak_functor(endomorphisms, &category.hom_sets);

        //step 3. clean up the szymczak classes
        let szymczak_classes: SzymczakClasses<Object, E> = raw_szymczak_classes
            .into_iter()
            .map(Self::drop_cycles)
            .map(Self::sort_by_object)
            .collect();

        todo!()
    }

    //----------------------------------------------------------------------

    fn raw_szymczak_functor<M: Morphism<Object, Object>>(
        mut endomorphisms: Endomorphisms<E>,
        hom_sets: &HashMap<(Object, Object), Vec<M>>,
    ) -> RawSzymczakClasses<E> {
        if endomorphisms.len() > RECURSION_PARAMETER_SZYMCZAK_FUNCTOR {
            let left_endomorphisms = endomorphisms.split_off(endomorphisms.len() / 2);
            let right_endomorphisms = endomorphisms;

            let left_raw_szymczak_classes =
                Self::raw_szymczak_functor::<M>(left_endomorphisms, hom_sets);
            let right_raw_szymczak_classes =
                Self::raw_szymczak_functor::<M>(right_endomorphisms, hom_sets);

            Self::merge_raw_szymczak_classes::<M>(
                left_raw_szymczak_classes,
                right_raw_szymczak_classes,
                hom_sets,
            )
        } else {
            Self::raw_szymczak_functor_final_step(endomorphisms, hom_sets)
        }
    }

    fn raw_szymczak_functor_final_step<M: Morphism<Object, Object>>(
        mut endomorphisms: Endomorphisms<E>,
        hom_sets: &HashMap<(Object, Object), Vec<M>>,
    ) -> RawSzymczakClasses<E> {
        let mut raw_szymczak_classes = RawSzymczakClasses::<E>::new();

        let endomorphisms_with_cycles: EndomorphismsWithCycles<E> = endomorphisms
            .into_iter()
            .map(|endomorphism| {
                let cycle: Vec<E> = endomorphism.cycle();
                (endomorphism, cycle)
            })
            .collect();

        for (endomorphism, cycle) in endomorphisms_with_cycles.into_iter() {
            let maybe_raw_szymczak_class: Option<&mut RawSzymczakClass<E>> =
                raw_szymczak_classes.iter_mut().find(|raw_szymczak_class| {
                    Self::are_szymczak_isomorphic(
                        (&endomorphism, &cycle),
                        raw_szymczak_class
                            .iter()
                            .next()
                            .expect("szymczak classes are never empty"),
                        hom_sets,
                    )
                });

            if let Some(raw_szymczak_class) = maybe_raw_szymczak_class {
                raw_szymczak_class.insert(endomorphism, cycle);
            } else {
                let mut new_raw_szymczak_class = RawSzymczakClass::<E>::new();
                new_raw_szymczak_class.insert(endomorphism, cycle);
                raw_szymczak_classes.push(new_raw_szymczak_class);
            }
        }

        raw_szymczak_classes
    }

    fn merge_raw_szymczak_classes<M: Morphism<Object, Object>>(
        mut left_raw_szymczak_classes: RawSzymczakClasses<E>,
        mut right_raw_szymczak_classes: RawSzymczakClasses<E>,
        hom_sets: &HashMap<(Object, Object), Vec<M>>,
    ) -> RawSzymczakClasses<E> {
        let mut merged_raw_szymczak_classes = RawSzymczakClasses::<E>::new();

        let mut left_index = 0;
        let mut left_merged = false;
        while left_index < left_raw_szymczak_classes.len() {
            let mut right_index = 0;
            let mut right_merged = false;

            while right_index < right_raw_szymczak_classes.len() {
                if Self::are_szymczak_isomorphic(
                    left_raw_szymczak_classes[left_index]
                        .iter()
                        .next()
                        .expect("szymczak classes are never empty"),
                    right_raw_szymczak_classes[right_index]
                        .iter()
                        .next()
                        .expect("szymczak classes are never empty"),
                    hom_sets,
                ) {
                    let mut merged_raw_szymczak_class =
                        left_raw_szymczak_classes.swap_remove(left_index);

                    merged_raw_szymczak_class
                        .extend(right_raw_szymczak_classes.swap_remove(right_index));

                    merged_raw_szymczak_classes.push(merged_raw_szymczak_class);

                    left_merged = true;
                    right_merged = true;
                }

                if right_merged {
                    //it means that at the right_index we find another raw_szymczak_class - we know that at left_index as well, hence we can keep the indices
                    right_merged = false;
                } else {
                    right_index += 1; //if not, move to the next class
                }
            }
            //similairly for the left part
            if left_merged {
                left_merged = false;
            } else {
                left_index += 1;
            }
        }

        merged_raw_szymczak_classes.extend(left_raw_szymczak_classes);
        merged_raw_szymczak_classes.extend(right_raw_szymczak_classes);
        merged_raw_szymczak_classes
    }

    fn drop_cycles(mut raw_szymczak_class: RawSzymczakClass<E>) -> Vec<E> {
        raw_szymczak_class.into_keys().collect::<Vec<E>>()
    }

    fn sort_by_object(mut raw_szymczak_class_without_cycles: Vec<E>) -> SzymczakClass<Object, E> {
        let mut szymczak_class = SzymczakClass::<Object, E>::new();

        for endomorphism in raw_szymczak_class_without_cycles.into_iter() {
            szymczak_class
                .entry(endomorphism.source().as_ref().clone())
                .or_default()
                .push(endomorphism)
        }

        szymczak_class
    }

    fn are_szymczak_isomorphic<M: Morphism<Object, Object>>(
        left_endomorphism_with_cycles: (&E, &Vec<E>),
        right_endomorphism_with_cycles: (&E, &Vec<E>),
        hom_sets: &HashMap<(Object, Object), Vec<M>>,
    ) -> bool {
        todo!() // a lot!
    }
}
