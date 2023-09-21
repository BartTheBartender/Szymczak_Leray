use crate::category::{
    morphism::{Endo as EndoMorphism, Morphism},
    object::Object,
    Container as Category, HomSet,
};
// use rayon;
use std::{
    borrow::Borrow,
    collections::HashMap,
    fmt::{self, Debug, Display},
    hash::Hash,
    marker::{PhantomData, Send, Sync},
};

type EndoMorphisms<E> = Vec<E>;

//this formal inconsistency with type naming is needed for the Morphism::compose to work in are_szymczak_isomorphic
type EndoMorphismsWithCycles<M, E> = Vec<(M, Vec<E>)>;
type RawSzymczakClass<M, E> = Vec<(M, Vec<E>)>;
type RawSzymczakClasses<M, E> = Vec<RawSzymczakClass<M, E>>;

type SzymczakClass<O, E> = HashMap<O, Vec<E>>;
type SzymczakClasses<O, E> = Vec<SzymczakClass<O, E>>;

#[derive(Debug)]
pub struct SzymczakCategory<O: Object, M: Morphism<O>, E: EndoMorphism<O>> {
    pub szymczak_classes: SzymczakClasses<O, E>,
    morphisms: PhantomData<M>,
}

impl<
        O: Object + Hash + Clone + Sync + Send,
        M: Morphism<O> + Eq + Send + Sync + Clone,
        E: EndoMorphism<O>
            + Debug //to be removed in the future
            + Clone
            + Sync
            + Send
            + PartialEq
            + From<M>
            + Into<M>,
    > SzymczakCategory<O, M, E>
{
    pub fn szymczak_functor<const RECURSION_PARAMETER: usize>(category: &Category<O, M>) -> Self {
        //step 0. If recursion parameter is less than 2, it will lead to the undefined behaviour
        if RECURSION_PARAMETER < 2 {
            panic!("RECURSION_PARAMETER cannot be smaller than 2");
        }
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
        let raw_szymczak_classes = Self::raw_szymczak_functor::<{ RECURSION_PARAMETER }>(
            endomorphisms,
            &category.hom_sets,
        );

        //step 3. clean up the szymczak classes
        let szymczak_classes: SzymczakClasses<O, E> = raw_szymczak_classes
            .into_iter()
            .map(Self::drop_cycles)
            .map(Self::sort_by_object)
            .collect();

        SzymczakCategory {
            szymczak_classes,
            morphisms: PhantomData::<M>,
        }
    }

    fn raw_szymczak_functor<const RECURSION_PARAMETER: usize>(
        mut endomorphisms: EndoMorphisms<E>,
        hom_sets: &HomSet<O, M>,
    ) -> RawSzymczakClasses<M, E> {
        if endomorphisms.len() > RECURSION_PARAMETER {
            let left_endomorphisms = endomorphisms.split_off(endomorphisms.len() / 2);
            let right_endomorphisms = endomorphisms;
            assert_ne!(left_endomorphisms.len(), 0);
            assert_ne!(right_endomorphisms.len(), 0);

            let (left_raw_szymczak_classes, right_raw_szymczak_classes) = rayon::join(
                || {
                    Self::raw_szymczak_functor::<{ RECURSION_PARAMETER }>(
                        left_endomorphisms,
                        hom_sets,
                    )
                },
                || {
                    Self::raw_szymczak_functor::<{ RECURSION_PARAMETER }>(
                        right_endomorphisms,
                        hom_sets,
                    )
                },
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
        hom_sets: &HomSet<O, M>,
    ) -> RawSzymczakClasses<M, E> {
        let endomorphisms_with_cycles: EndoMorphismsWithCycles<M, E> = endomorphisms
            .into_iter()
            .map(|endomorphism| {
                let cycle: Vec<E> = endomorphism.cycle();
                (endomorphism.into(), cycle)
            })
            .collect();

        let raw_szymczak_classes = endomorphisms_with_cycles.into_iter().fold(
            RawSzymczakClasses::<M, E>::new(),
            |mut raw_szymczak_classes, (endomorphism, cycle)| {
                let maybe_raw_szymczak_class: Option<&mut RawSzymczakClass<M, E>> =
                    raw_szymczak_classes.iter_mut().find(|raw_szymczak_class| {
                        Self::are_szymczak_isomorphic(
                            (&endomorphism, &cycle),
                            transform(
                                raw_szymczak_class
                                    .iter()
                                    .next()
                                    .expect("szymczak classes are never empty"),
                            ),
                            hom_sets,
                        )
                    });
                if let Some(raw_szymczak_class) = maybe_raw_szymczak_class {
                    raw_szymczak_class.push((endomorphism, cycle));
                } else {
                    let mut new_raw_szymczak_class = RawSzymczakClass::<M, E>::new();
                    new_raw_szymczak_class.push((endomorphism, cycle));
                    raw_szymczak_classes.push(new_raw_szymczak_class);
                }

                raw_szymczak_classes
            },
        );

        assert!(!raw_szymczak_classes
            .iter()
            .find(|raw_szymczak_class| raw_szymczak_class.len() == 0)
            .is_some());
        raw_szymczak_classes
    }

    fn merge_raw_szymczak_classes(
        mut left_raw_szymczak_classes: RawSzymczakClasses<M, E>,
        mut right_raw_szymczak_classes: RawSzymczakClasses<M, E>,
        hom_sets: &HomSet<O, M>,
    ) -> RawSzymczakClasses<M, E> {
        let mut merged_raw_szymczak_classes: RawSzymczakClasses<M, E> =
            left_raw_szymczak_classes.iter_mut().fold(
                RawSzymczakClasses::<M, E>::new(),
                |mut merged_raw_szymczak_classes, left_raw_szymczak_class| {
                    if let Some(right_raw_szymczak_class) = right_raw_szymczak_classes
                        .iter_mut()
                        .find(|right_raw_szymczak_class| {
                            let left_endo_with_cycle = transform(
                                left_raw_szymczak_class
                                    .iter()
                                    .next()
                                    .expect("szymczak classes are never empty"),
                            );
                            let right_endo_with_cycle = transform(
                                right_raw_szymczak_class
                                    .iter()
                                    .next()
                                    .expect("szymczak classes are never empty"),
                            );

                            Self::are_szymczak_isomorphic(
                                left_endo_with_cycle,
                                right_endo_with_cycle,
                                hom_sets,
                            )
                        })
                    {
                        let mut merged_raw_szymczak_class = RawSzymczakClass::<M, E>::new();
                        merged_raw_szymczak_class.append(left_raw_szymczak_class);
                        merged_raw_szymczak_class.append(right_raw_szymczak_class);
                        merged_raw_szymczak_classes.push(merged_raw_szymczak_class);
                    }

                    right_raw_szymczak_classes
                        .retain(|right_raw_szymczak_class| right_raw_szymczak_class.len() != 0);

                    merged_raw_szymczak_classes
                },
            );
        left_raw_szymczak_classes
            .retain(|left_raw_szymczak_class| left_raw_szymczak_class.len() != 0);

        merged_raw_szymczak_classes.append(&mut left_raw_szymczak_classes);
        merged_raw_szymczak_classes.append(&mut right_raw_szymczak_classes);

        merged_raw_szymczak_classes
    }

    fn drop_cycles(raw_szymczak_class: RawSzymczakClass<M, E>) -> Vec<E> {
        raw_szymczak_class
            .into_iter()
            .map(|(endomorphism, _)| endomorphism.into())
            .collect::<Vec<E>>()
    }

    fn sort_by_object(raw_szymczak_class_without_cycle: Vec<E>) -> SzymczakClass<O, E> {
        let mut szymczak_class = SzymczakClass::<O, E>::new();

        for endomorphism in raw_szymczak_class_without_cycle.into_iter() {
            szymczak_class
                .entry(endomorphism.source().borrow().clone()) //this clone is needed to be stored as a key for the hashmap
                .or_default()
                .push(endomorphism)
        }

        szymczak_class
    }

    //the endomorphism is casted to M on purpouse, to make M::compose make sense
    fn are_szymczak_isomorphic(
        left_endomorphism_with_cycle: (&M, &Vec<E>),
        right_endomorphism_with_cycle: (&M, &Vec<E>),
        hom_sets: &HomSet<O, M>,
    ) -> bool {
        let (l, l_cycle) = left_endomorphism_with_cycle;
        let (r, r_cycle) = right_endomorphism_with_cycle;

        let morphisms_l_to_r: &Vec<M> = hom_sets
            .get(l.target().borrow())
            .expect("there is a hom_set with the given object")
            .get(r.source().borrow())
            .expect("there is a hom_set with the given object");

        let morphisms_r_to_l: &Vec<M> = hom_sets
            .get(r.target().borrow())
            .expect("there is a hom_set with the given object")
            .get(l.source().borrow())
            .expect("there is a hom_set with the given object");

        for l_to_r in morphisms_l_to_r.iter() {
            for r_to_l in morphisms_r_to_l.iter() {
                if l_to_r.compose(l) == r.compose(r_to_l)
                    && r_to_l.compose(l) == r.compose(r_to_l)
                    && Self::is_identity(&E::from(l_to_r.compose(r_to_l)), l_cycle)
                    && Self::is_identity(&E::from(r_to_l.compose(l_to_r)), r_cycle)
                {
                    return true;
                }
            }
        }
        false
    }

    fn is_identity(morphism: &E, cycle: &Vec<E>) -> bool {
        for en in cycle.iter() {
            let en_morphism = morphism.compose(en);

            for em in cycle.iter() {
                if en_morphism == *em {
                    return true;
                }
            }
        }

        false
    }
}

pub fn transform<L, R>(reference_to_tuple: &(L, R)) -> (&L, &R) {
    let (left, right) = reference_to_tuple;
    (&left, &right)
}

impl<O: Object + Display, M: Morphism<O>, E: EndoMorphism<O> + Display> Display
    for SzymczakCategory<O, M, E>
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
        category::{object::Concrete, relation::Relation, Container as Category},
        ralg::{
            cgroup::{ideal::CIdeal, C},
            module::canon::object::Object as CanonModule,
        },
    };

    #[test]
    #[ignore]
    fn szymczak_isomorphism_is_equivalence() {
        use typenum::U5 as N;
        type R = C<N>;
        type I = CIdeal<N>;

        let category = Category::<CanonModule<R, I>, Relation<R, I>>::new(1);

        let hom_set_zn_zn: Vec<Relation<R, I>> = category
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
    #[ignore]
    fn szymczak_isomorphism_isnt_identically_true_nor_false() {
        use typenum::U5 as N;

        type R = C<N>;
        type I = CIdeal<N>;

        let category = Category::<CanonModule<R, I>, Relation<R, I>>::new(1);

        let hom_set_zn_zn: Vec<Relation<R, I>> = category
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
    #[ignore]
    fn szymczak_isomorphism_different_base_objects() {
        use typenum::{Unsigned, U2 as P};
        type R = C<P>;
        type I = CIdeal<P>;
        let p = P::to_usize();

        let category = Category::<CanonModule<R, I>, Relation<R, I>>::new(1);

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
    #[ignore]
    fn is_identity() {
        use typenum::{Unsigned, U2 as P};
        type R = C<P>;
        type I = CIdeal<P>;
        let p = P::to_usize();

        let category = Category::<CanonModule<R, I>, Relation<R, I>>::new(1);

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
            category.hom_set(top_z1.target().borrow(), top_z2.source().borrow());
        let morphisms_top_z2_to_top_z1 =
            category.hom_set(top_z2.target().borrow(), top_z1.source().borrow());

        let mut are_szymczak_isomorphic: bool = false;
        let mut are_there_morphisms: bool = false;

        for top_z1_to_top_z2 in morphisms_top_z1_to_top_z2.iter() {
            for top_z2_to_top_z1 in morphisms_top_z2_to_top_z1.iter() {
                println!("{}\n{}\n---", top_z1_to_top_z2, top_z2_to_top_z1);

                if top_z1.compose(&top_z1_to_top_z2) == top_z1_to_top_z2.compose(&top_z2)
                    && top_z2.compose(&top_z2_to_top_z1) == top_z2_to_top_z1.compose(&top_z1)
                {
                    are_there_morphisms = true;

                    if SzymczakCategory::<
                        CanonModule<R, I>,
                        Relation<R, I>,
                        Relation<R, I>,
                    >::is_identity(
                        &top_z1_to_top_z2.compose(&top_z2_to_top_z1), &top_z1_cycle
                    ) && SzymczakCategory::<
                        CanonModule<R, I>,
                        Relation<R, I>,
                        Relation<R, I>,
                    >::is_identity(
                        &top_z2_to_top_z1.compose(&top_z1_to_top_z2), &top_z2_cycle
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
    #[ignore]
    fn szymczak_classes_for_zp() {
        use typenum::{Unsigned, U7 as P};
        type R = C<P>;
        type I = CIdeal<P>;
        let p = P::to_usize();

        let category = Category::<CanonModule<R, I>, Relation<R, I>>::new(1);
        // println!("{}\n---", category);
        let szymczak_category =
            SzymczakCategory::<CanonModule<R, I>, Relation<R, I>, Relation<R, I>>::szymczak_functor::<
                20,
            >(&category);

        //println!("{}", szymczak_category);

        assert_eq!(szymczak_category.szymczak_classes.len(), p);
    }

    #[test]
    #[ignore]
    fn cycles_generation_for_zp() {
        use typenum::{Unsigned, U11 as P};
        type R = C<P>;
        type I = CIdeal<P>;
        let p = P::to_usize();

        let category = Category::<CanonModule<R, I>, Relation<R, I>>::new(1);

        let endomorphisms: EndoMorphisms<Relation<R, I>> = category
            .hom_sets
            .iter()
            .flat_map(|(source, hom_sets_fixed_source)| {
                hom_sets_fixed_source
                    .iter()
                    .filter(move |(target, _)| *target == source)
                    .flat_map(|(_, morphisms)| {
                        morphisms
                            .iter()
                            .map(|morphism| Relation::<R, I>::from(morphism.clone()))
                    })
            })
            .collect();
        let endomorphisms_len = endomorphisms.len();
        let endomorphisms_with_cycles: EndoMorphismsWithCycles<Relation<R, I>, Relation<R, I>> =
            endomorphisms
                .into_iter()
                .map(|endomorphism| {
                    let cycle: Vec<Relation<R, I>> = endomorphism.cycle();
                    (endomorphism, cycle)
                })
                .collect();

        let endomorphisms_with_cycles_len = endomorphisms_with_cycles.len();

        assert_eq!(p + 4, endomorphisms_len); //udowodnione naukowo
        assert_eq!(endomorphisms_with_cycles_len, endomorphisms_len);
    }

    #[test]
    #[ignore]
    #[ignore]
    fn raw_szymczak_functor_for_zp() {
        use typenum::{Unsigned, U7 as P};
        type R = C<P>;
        type I = CIdeal<P>;
        let p = P::to_usize();

        let category = Category::<CanonModule<R, I>, Relation<R, I>>::new(1);

        let endomorphisms: EndoMorphisms<Relation<R, I>> = category
            .hom_sets
            .iter()
            .flat_map(|(source, hom_sets_fixed_source)| {
                hom_sets_fixed_source
                    .iter()
                    .filter(move |(target, _)| *target == source)
                    .flat_map(|(_, morphisms)| {
                        morphisms
                            .iter()
                            .map(|morphism| Relation::<R, I>::from(morphism.clone()))
                    })
            })
            .collect();

        for endo in endomorphisms.iter() {
            println!("{:?}", endo);
        }

        let endomorphisms_with_cycles: EndoMorphismsWithCycles<Relation<R, I>, Relation<R, I>> =
            endomorphisms
                .into_iter()
                .map(|endomorphism| {
                    let cycle: Vec<Relation<R, I>> = endomorphism.cycle();
                    (endomorphism, cycle)
                })
                .collect();

        let endomorphisms_with_cycles_len = endomorphisms_with_cycles.len();

        let raw_szymczak_classes = RawSzymczakClasses::<Relation<R, I>, Relation<R, I>>::new();

        let raw_szymczak_classes = endomorphisms_with_cycles.into_iter().fold(
            raw_szymczak_classes,
            |mut raw_szymczak_classes, (endomorphism, cycle)| {
                // println!("{:?}", endomorphism);
                let maybe_raw_szymczak_class: Option<
                    &mut RawSzymczakClass<Relation<R, I>, Relation<R, I>>,
                > = raw_szymczak_classes.iter_mut().find(|raw_szymczak_class| {
                    SzymczakCategory::are_szymczak_isomorphic(
                        (&endomorphism, &cycle),
                        transform(
                            raw_szymczak_class
                                .iter()
                                .next()
                                .expect("szymczak classes are never empty"),
                        ),
                        &category.hom_sets,
                    )
                });
                if let Some(raw_szymczak_class) = maybe_raw_szymczak_class {
                    raw_szymczak_class.push((endomorphism, cycle));
                } else {
                    let mut new_raw_szymczak_class =
                        RawSzymczakClass::<Relation<R, I>, Relation<R, I>>::new();
                    new_raw_szymczak_class.push((endomorphism, cycle));
                    raw_szymczak_classes.push(new_raw_szymczak_class);
                }

                raw_szymczak_classes
            },
        );

        println!("\n\nAFTER GENERATION:");
        for raw_szymczak_class in raw_szymczak_classes.iter() {
            println!("new szymczak class:");

            for endo in raw_szymczak_class.iter() {
                println!("endo:{:?}", endo.0);
            }
        }

        assert_eq!(raw_szymczak_classes.len(), p);
    }

    #[test]
    #[ignore]
    fn merge_raw_szymczak_classes() {
        use typenum::{Unsigned, U5 as P};
        type R = C<P>;
        type I = CIdeal<P>;
        let p = P::to_usize();
        const RECURSION_PARAMETER: usize = 2;

        let category = Category::<CanonModule<R, I>, Relation<R, I>>::new(1);

        let szymczak_category =
            SzymczakCategory::<CanonModule<R, I>, Relation<R, I>, Relation<R, I>>::szymczak_functor::<
                { RECURSION_PARAMETER },
            >(&category);
        assert_eq!(szymczak_category.szymczak_classes.len(), p);
    }
}
