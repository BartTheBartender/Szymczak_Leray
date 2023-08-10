use crate::{
    category::{
        morphism::{Compose, EndoMorphism, Morphism},
        Category, HomSet,
    },
    RECURSION_PARAMETER_SZYMCZAK_FUNCTOR,
};
// use rayon;
use std::{
    collections::HashMap,
    fmt::{self, Display},
    hash::Hash,
    marker::{PhantomData, Send, Sync},
};

type EndoMorphisms<E> = Vec<E>;
type EndoMorphismsWithCycles<E> = HashMap<E, Vec<E>>;

type RawSzymczakClass<E> = HashMap<E, Vec<E>>;
type RawSzymczakClasses<E> = Vec<RawSzymczakClass<E>>;

type SzymczakClass<Object, E> = HashMap<Object, Vec<E>>;
type SzymczakClasses<Object, E> = Vec<SzymczakClass<Object, E>>;

pub struct SzymczakCategory<Object: Eq, M: Morphism<Object, Object>, E: EndoMorphism<Object>> {
    pub szymczak_classes: SzymczakClasses<Object, E>,
    morphisms: PhantomData<M>,
}

impl<
        Object: Eq + PartialEq + Hash + Clone + Sync + Send,
        M: Morphism<Object, Object>
            + Eq
            + Compose<Object, Object, Object, M, Output = M>
            + Compose<Object, Object, Object, E, Output = M>
            + Sync
            + Clone,
        E: EndoMorphism<Object>
            + Sync
            + Send
            + From<M>
            + Compose<Object, Object, Object, M, Output = M>,
    > SzymczakCategory<Object, M, E>
{
    //i dont really know if name of the function below is correct, but i find it cool (i woudl like to know your opinion as well). I dont know also how to parallelize it (i assume that we will have to use parallel Hash structures by rayon)

    pub fn szymczak_functor(category: &Category<Object, M>) -> Self {
        //step 1. Clone all the endomorphisms (we will need them to be owned)

        let endomorphisms: EndoMorphisms<E> = category
            .hom_sets
            .iter()
            .flat_map(|(source, hom_sets_fixed_source)| {
                hom_sets_fixed_source
                    .iter()
                    .filter(move |(target, _)| *target == source)
                    .flat_map(|(_, morphisms)| {
                        morphisms.iter().map(|morphism| E::from(morphism.clone()))
                    })
            })
            .collect();

        //step 2. generate raw szymczak classes (by raw i mean they are unsorted by object and endomorphisms keep their cycles)
        let raw_szymczak_classes = Self::raw_szymczak_functor(endomorphisms, &category.hom_sets);

        //step 3. clean up the szymczak classes
        let szymczak_classes: SzymczakClasses<Object, E> = raw_szymczak_classes
            .into_iter()
            .map(Self::drop_cycles)
            .map(Self::sort_by_object)
            .collect();

        SzymczakCategory {
            szymczak_classes,
            morphisms: PhantomData::<M>,
        }
    }

    //----------------------------------------------------------------------

    fn raw_szymczak_functor(
        mut endomorphisms: EndoMorphisms<E>,
        hom_sets: &HomSet<Object, M>,
    ) -> RawSzymczakClasses<E> {
        if endomorphisms.len() > RECURSION_PARAMETER_SZYMCZAK_FUNCTOR {
            let left_endomorphisms = endomorphisms.split_off(endomorphisms.len() / 2);
            let right_endomorphisms = endomorphisms;

            let (left_raw_szymczak_classes, right_raw_szymczak_classes) = rayon::join(
                || Self::raw_szymczak_functor(left_endomorphisms, hom_sets),
                || Self::raw_szymczak_functor(right_endomorphisms, hom_sets),
            );

            Self::merge_raw_szymczak_classes(
                left_raw_szymczak_classes,
                right_raw_szymczak_classes,
                hom_sets,
            )
        } else {
            Self::raw_szymczak_functor_final_step(endomorphisms, hom_sets)
        }
    }

    fn raw_szymczak_functor_final_step(
        endomorphisms: EndoMorphisms<E>,
        hom_sets: &HomSet<Object, M>,
    ) -> RawSzymczakClasses<E> {
        let mut raw_szymczak_classes = RawSzymczakClasses::<E>::new();

        let endomorphisms_with_cycles: EndoMorphismsWithCycles<E> = endomorphisms
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

    fn merge_raw_szymczak_classes(
        mut left_raw_szymczak_classes: RawSzymczakClasses<E>,
        mut right_raw_szymczak_classes: RawSzymczakClasses<E>,
        hom_sets: &HomSet<Object, M>,
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

    fn drop_cycles(raw_szymczak_class: RawSzymczakClass<E>) -> Vec<E> {
        raw_szymczak_class.into_keys().collect::<Vec<E>>()
    }

    fn sort_by_object(raw_szymczak_class_without_cycles: Vec<E>) -> SzymczakClass<Object, E> {
        let mut szymczak_class = SzymczakClass::<Object, E>::new();

        for endomorphism in raw_szymczak_class_without_cycles.into_iter() {
            szymczak_class
                .entry(endomorphism.source().as_ref().clone()) //this clone is needed to be stored as a key for the hashmap
                .or_default()
                .push(endomorphism)
        }

        szymczak_class
    }

    fn are_szymczak_isomorphic(
        left_endomorphism_with_cycles: (&E, &Vec<E>),
        right_endomorphism_with_cycles: (&E, &Vec<E>),
        hom_sets: &HomSet<Object, M>,
    ) -> bool {
        let (l, l_cycles) = left_endomorphism_with_cycles;
        let (r, r_cycles) = right_endomorphism_with_cycles;

        let morphisms_l_to_r: &Vec<M> = hom_sets
            .get(l.target().as_ref())
            .expect("there is a hom_set with the given object")
            .get(r.source().as_ref())
            .expect("there is a hom_set with the given object");

        let morphisms_r_to_l: &Vec<M> = hom_sets
            .get(r.target().as_ref())
            .expect("there is a hom_set with the given object")
            .get(l.source().as_ref())
            .expect("there is a hom_set with the given object");

        for l_to_r in morphisms_l_to_r.iter() {
            for r_to_l in morphisms_r_to_l.iter() {
                if l_to_r.compose_unchecked(r) == l.compose_unchecked(l_to_r)
                    && r_to_l.compose_unchecked(l) == r.compose_unchecked(r_to_l)
                    && Self::is_identity(&E::from(l_to_r.compose_unchecked(r_to_l)), l_cycles)
                    && Self::is_identity(&E::from(r_to_l.compose_unchecked(l_to_r)), r_cycles)
                {
                    return true;
                }
            }
        }
        false
    }

    fn is_identity(morphism: &E, cycle: &Vec<E>) -> bool {
        for en in cycle.iter() {
            let en_morphism = morphism.compose_unchecked(en);

            for em in cycle.iter() {
                if en_morphism == *em {
                    return true;
                }
            }
        }

        false
    }
}

impl<Object: Eq + Display, M: Morphism<Object, Object>, E: EndoMorphism<Object> + Display> Display
    for SzymczakCategory<Object, M, E>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut string = String::new();

        for szymczak_class in self.szymczak_classes.iter() {
            string.push_str("new szymczak class:\n");
            for (object, endomorphisms) in szymczak_class.iter() {
                string.push_str(&["object:", &object.to_string()].join(" "));
                string.push_str("\n");
                for endomorphism in endomorphisms.iter() {
                    string.push_str(&endomorphism.to_string());
                    string.push_str("\n");
                }
                string.push_str("\n");
            }
        }
        write!(f, "{}", string)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[allow(unused_imports)]
    use crate::{
        category::{relation::Relation, Category, HomSet},
        rmodule::canon::CanonModule,
    };
    // use bitvec::prelude::*;
    // use std::{collections::HashMap, sync::Arc};

    #[test]
    fn zp_category_length() {
        let primes = [2, 3, 5, 7, 11, 13];
        for prime in primes {
            let category = Category::new(prime, 1);
            let szymczak_category = SzymczakCategory::szymczak_functor(&category);
            assert_eq!(szymczak_category.szymczak_classes.len(), prime.into())
        }
    }
}
