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
        let endomorphisms_with_cycles: EndoMorphismsWithCycles<E> = endomorphisms
            .into_iter()
            .map(|endomorphism| {
                let cycle: Vec<E> = endomorphism.cycle();
                (endomorphism, cycle)
            })
            .collect();

        let raw_szymczak_classes = RawSzymczakClasses::<E>::new();

        endomorphisms_with_cycles.into_iter().fold(
            raw_szymczak_classes,
            |mut raw_szymczak_classes, (endomorphism, cycle)| {
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

                raw_szymczak_classes
            },
        )
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

    fn sort_by_object(raw_szymczak_class_without_cycle: Vec<E>) -> SzymczakClass<Object, E> {
        let mut szymczak_class = SzymczakClass::<Object, E>::new();

        for endomorphism in raw_szymczak_class_without_cycle.into_iter() {
            szymczak_class
                .entry(endomorphism.source().as_ref().clone()) //this clone is needed to be stored as a key for the hashmap
                .or_default()
                .push(endomorphism)
        }

        szymczak_class
    }

    fn are_szymczak_isomorphic(
        left_endomorphism_with_cycle: (&E, &Vec<E>),
        right_endomorphism_with_cycle: (&E, &Vec<E>),
        hom_sets: &HomSet<Object, M>,
    ) -> bool {
        let (l, l_cycle) = left_endomorphism_with_cycle;
        let (r, r_cycle) = right_endomorphism_with_cycle;

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
                    && Self::is_identity(&E::from(l_to_r.compose_unchecked(r_to_l)), l_cycle)
                    && Self::is_identity(&E::from(r_to_l.compose_unchecked(l_to_r)), r_cycle)
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
    use crate::{
        category::{relation::Relation, Category},
        rmodule::{canon::CanonModule, ring::Fin},
        SzymczakCategory,
    };

    #[test]
    fn szymczak_isomorphism_is_equivalence() {
        use typenum::{Unsigned, U5 as N};
        type R = Fin<N>;

        let category = Category::<CanonModule<R>, Relation<R>>::new(1);

        let hom_set_zn_zn: Vec<Relation<R>> = category
            .clone()
            .hom_sets
            .into_iter()
            .find(|(source, _)| source.cardinality() > 1)
            .expect("there is a hom_set with non-trivial source")
            .1
            .into_iter()
            .find(|(target, _)| target.cardinality() > 1)
            .expect("there is a hom_set with non-trivial target")
            .1;

        //reflexive
        for index in 0..hom_set_zn_zn.len() {
            let endo = &hom_set_zn_zn[index];

            let endo_with_cycle = (endo, &endo.cycle());

            assert!(SzymczakCategory::are_szymczak_isomorphic(
                endo_with_cycle,
                endo_with_cycle,
                &category.hom_sets
            ));
        }

        //symmetric
        for index_0 in 0..hom_set_zn_zn.len() {
            let endo_0 = &hom_set_zn_zn[index_0];
            let endo_with_cycle_0 = (endo_0, &endo_0.cycle());

            for index_1 in 0..hom_set_zn_zn.len() {
                let endo_1 = &hom_set_zn_zn[index_1];
                let endo_with_cycle_1 = (endo_1, &endo_1.cycle());

                if SzymczakCategory::are_szymczak_isomorphic(
                    endo_with_cycle_0,
                    endo_with_cycle_1,
                    &category.hom_sets,
                ) {
                    assert!(SzymczakCategory::are_szymczak_isomorphic(
                        endo_with_cycle_1,
                        endo_with_cycle_0,
                        &category.hom_sets,
                    ));
                }
            }
        }

        //transitive
        for index_0 in 0..hom_set_zn_zn.len() {
            let endo_0 = &hom_set_zn_zn[index_0];
            let endo_with_cycle_0 = (endo_0, &endo_0.cycle());

            for index_1 in 0..hom_set_zn_zn.len() {
                let endo_1 = &hom_set_zn_zn[index_1];
                let endo_with_cycle_1 = (endo_1, &endo_1.cycle());
                for index_2 in 0..hom_set_zn_zn.len() {
                    let endo_2 = &hom_set_zn_zn[index_2];
                    let endo_with_cycle_2 = (endo_2, &endo_2.cycle());

                    if SzymczakCategory::are_szymczak_isomorphic(
                        endo_with_cycle_0,
                        endo_with_cycle_1,
                        &category.hom_sets,
                    ) && SzymczakCategory::are_szymczak_isomorphic(
                        endo_with_cycle_1,
                        endo_with_cycle_2,
                        &category.hom_sets,
                    ) {
                        assert!(SzymczakCategory::are_szymczak_isomorphic(
                            endo_with_cycle_0,
                            endo_with_cycle_2,
                            &category.hom_sets,
                        ));
                    }
                }
            }
        }
    }

    #[test]
    fn szymczak_isomorphism_isnt_identically_true_nor_false() {
        use typenum::{Unsigned, U5 as N};

        type R = Fin<N>;

        let category = Category::<CanonModule<R>, Relation<R>>::new(1);

        let hom_set_zn_zn: Vec<Relation<R>> = category
            .clone()
            .hom_sets
            .into_iter()
            .find(|(source, _)| source.cardinality() > 1)
            .expect("there is a hom_set with non-trivial source")
            .1
            .into_iter()
            .find(|(target, _)| target.cardinality() > 1)
            .expect("there is a hom_set with non-trivial target")
            .1;

        assert_eq!(hom_set_zn_zn.len(), 8);

        for morphism in hom_set_zn_zn.iter() {
            assert_eq!(morphism.source(), morphism.target());
        }

        let mut is_sometimes_true: bool = false;
        let mut is_sometimes_false: bool = false;

        for endo_0 in hom_set_zn_zn.iter() {
            let endo_with_cycle_0 = (endo_0, &endo_0.cycle());
            for endo_1 in hom_set_zn_zn.iter() {
                let endo_with_cycle_1 = (endo_1, &endo_1.cycle());

                if endo_0 != endo_1 {
                    if SzymczakCategory::are_szymczak_isomorphic(
                        endo_with_cycle_0,
                        endo_with_cycle_1,
                        &category.hom_sets,
                    ) {
                        is_sometimes_true = true;
                    } else {
                        is_sometimes_false = true;
                    }
                }
            }
        }

        assert!(is_sometimes_false);
        assert!(is_sometimes_true);
    }

    #[test]
    fn szymczak_isomorphism_different_base_objects() {
        use typenum::{Unsigned, U2 as P};
        type R = Fin<P>;
        let p = P::to_usize();

        let category = Category::<CanonModule<R>, Relation<R>>::new(1);

        let all_objects = category.clone().objects();
        assert_eq!(all_objects.len(), 2);

        let z1 = all_objects
            .iter()
            .find(|object| object.cardinality() == 1)
            .expect("there is z1 module")
            .clone();
        let z2 = all_objects
            .iter()
            .find(|object| object.cardinality() == p)
            .expect("there is z2 module")
            .clone();

        assert_eq!(z2.to_string(), "Z2");
        assert_eq!(z1.to_string(), "Z1");

        let mut z1_to_z1 = category.hom_set(&z1, &z1);
        let z2_to_z2 = category.hom_set(&z2, &z2);

        let top_z1 = z1_to_z1.pop().expect("there is only top relation on z1");
        let top_z2 = z2_to_z2
            .iter()
            .find(|endo| endo.matrix.buffer() == vec![true, true, true, true])
            .expect("there is the top relation on z2");

        assert_eq!(top_z1.matrix.buffer(), vec![true]);
        assert_eq!(top_z2.matrix.buffer(), vec![true, true, true, true]);

        let top_z1_with_cycle = (&top_z1, &top_z1.cycle());
        let top_z2_with_cycle = (top_z2, &top_z2.cycle());

        assert!(SzymczakCategory::are_szymczak_isomorphic(
            top_z1_with_cycle,
            top_z2_with_cycle,
            &category.hom_sets
        ));
    }

    #[test]
    fn is_identity() {
        use typenum::{Unsigned, U2 as P};
        type R = Fin<P>;
        let p = P::to_usize();

        let category = Category::<CanonModule<R>, Relation<R>>::new(1);

        let all_objects = category.clone().objects();
        assert_eq!(all_objects.len(), 2);

        let z1 = all_objects
            .iter()
            .find(|object| object.cardinality() == 1)
            .expect("there is z1 module")
            .clone();
        let z2 = all_objects
            .iter()
            .find(|object| object.cardinality() == p)
            .expect("there is z2 module")
            .clone();

        assert_eq!(z2.to_string(), "Z2");
        assert_eq!(z1.to_string(), "Z1");

        let mut z1_to_z1 = category.hom_set(&z1, &z1);
        let z2_to_z2 = category.hom_set(&z2, &z2);

        let top_z1 = z1_to_z1.pop().expect("there is only top relation on z1");
        let top_z2 = z2_to_z2
            .iter()
            .find(|endo| endo.matrix.buffer() == vec![true, true, true, true])
            .expect("there is the top relation on z2");

        let top_z1_cycle = top_z1.cycle();
        let top_z2_cycle = top_z2.cycle();

        let morphisms_top_z1_to_top_z2 =
            category.hom_set(top_z1.target().as_ref(), top_z2.source().as_ref());
        let morphisms_top_z2_to_top_z1 =
            category.hom_set(top_z2.target().as_ref(), top_z1.source().as_ref());

        let mut are_szymczak_isomorphic: bool = false;
        let mut are_there_morphisms: bool = false;

        for top_z1_to_top_z2 in morphisms_top_z1_to_top_z2.iter() {
            for top_z2_to_top_z1 in morphisms_top_z2_to_top_z1.iter() {
                println!("{}\n{}\n---", top_z1_to_top_z2, top_z2_to_top_z1);

                if top_z1.compose_unchecked(&top_z1_to_top_z2)
                    == top_z1_to_top_z2.compose_unchecked(&top_z2)
                    && top_z2.compose_unchecked(&top_z2_to_top_z1)
                        == top_z2_to_top_z1.compose_unchecked(&top_z1)
                {
                    are_there_morphisms = true;

                    if SzymczakCategory::is_identity(
                        &top_z1_to_top_z2.compose_unchecked(&top_z2_to_top_z1),
                        &top_z1_cycle,
                    ) && SzymczakCategory::is_identity(
                        &top_z2_to_top_z1.compose_unchecked(&top_z1_to_top_z2),
                        &top_z2_cycle,
                    ) {
                        are_szymczak_isomorphic = true;
                    }
                }
            }
        }

        assert!(are_there_morphisms);
        assert!(are_szymczak_isomorphic);
    }

    #[test]
    fn szymczak_classes_for_zp() {
        use typenum::{Unsigned, U7 as P};
        type R = Fin<P>;
        let p = P::to_usize();

        let category = Category::<CanonModule<R>, Relation<R>>::new(1);
        // println!("{}\n---", category);
        let szymczak_category = SzymczakCategory::szymczak_functor(&category);

        //println!("{}", szymczak_category);

        assert_eq!(szymczak_category.szymczak_classes.len(), p);
    }

    #[test]
    fn cycles_generation_for_zp() {
        use typenum::{Unsigned, U11 as P};
        type R = Fin<P>;
        let p = P::to_usize();

        let category = Category::<CanonModule<R>, Relation<R>>::new(1);

        let endomorphisms: EndoMorphisms<Relation<R>> = category
            .hom_sets
            .iter()
            .flat_map(|(source, hom_sets_fixed_source)| {
                hom_sets_fixed_source
                    .iter()
                    .filter(move |(target, _)| *target == source)
                    .flat_map(|(_, morphisms)| {
                        morphisms
                            .iter()
                            .map(|morphism| Relation::<R>::from(morphism.clone()))
                    })
            })
            .collect();
        let endomorphisms_len = endomorphisms.len();
        let mut count = 0;
        let endomorphisms_with_cycles: EndoMorphismsWithCycles<Relation<R>> = endomorphisms
            .into_iter()
            .map(|endomorphism| {
                println!("{}", count);
                count += 1;
                let cycle: Vec<Relation<R>> = endomorphism.cycle();
                (endomorphism, cycle)
            })
            .collect();

        let endomorphisms_with_cycles_len = endomorphisms_with_cycles.len();

        assert_eq!(p + 4, endomorphisms_len); //udowodnione naukowo
        assert_eq!(endomorphisms_with_cycles_len, endomorphisms_len);
    }

    #[test]
    fn raw_szymczak_functor_for_zp() {
        use typenum::{Unsigned, U11 as P};
        type R = Fin<P>;
        let p = P::to_usize();

        let category = Category::<CanonModule<R>, Relation<R>>::new(1);

        let endomorphisms: EndoMorphisms<Relation<R>> = category
            .hom_sets
            .iter()
            .flat_map(|(source, hom_sets_fixed_source)| {
                hom_sets_fixed_source
                    .iter()
                    .filter(move |(target, _)| *target == source)
                    .flat_map(|(_, morphisms)| {
                        morphisms
                            .iter()
                            .map(|morphism| Relation::<R>::from(morphism.clone()))
                    })
            })
            .collect();
        let endomorphisms_len = endomorphisms.len();

        let endomorphisms_with_cycles: EndoMorphismsWithCycles<Relation<R>> = endomorphisms
            .into_iter()
            .map(|endomorphism| {
                let cycle: Vec<Relation<R>> = endomorphism.cycle();
                (endomorphism, cycle)
            })
            .collect();

        let endomorphisms_with_cycles_len = endomorphisms_with_cycles.len();

        let mut count_out = 0;

        let raw_szymczak_classes = RawSzymczakClasses::<Relation<R>>::new();

        let raw_szymczak_classes = endomorphisms_with_cycles.into_iter().fold(
            raw_szymczak_classes,
            |mut raw_szymczak_classes, (endomorphism, cycle)| {
                count_out += 1;
                let maybe_raw_szymczak_class: Option<&mut RawSzymczakClass<Relation<R>>> =
                    raw_szymczak_classes.iter_mut().find(|raw_szymczak_class| {
                        SzymczakCategory::are_szymczak_isomorphic(
                            (&endomorphism, &cycle),
                            raw_szymczak_class
                                .iter()
                                .next()
                                .expect("szymczak classes are never empty"),
                            &category.hom_sets,
                        )
                    });
                if let Some(raw_szymczak_class) = maybe_raw_szymczak_class {
                    raw_szymczak_class.insert(endomorphism, cycle);
                    println!("inserted into a szymczak class");
                } else {
                    let mut new_raw_szymczak_class = RawSzymczakClass::<Relation<R>>::new();
                    new_raw_szymczak_class.insert(endomorphism, cycle);
                    raw_szymczak_classes.push(new_raw_szymczak_class);
                    println!("created a new szymczak class");
                }

                raw_szymczak_classes
            },
        );

        assert_eq!(endomorphisms_with_cycles_len, endomorphisms_len);
        assert_eq!(endomorphisms_with_cycles_len, count_out);
        assert_eq!(raw_szymczak_classes.len(), p);
    }
}
