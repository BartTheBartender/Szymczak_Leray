use crate::category::{
    morphism::{Endo as Morphism, IsBij, IsMap}, //i leave to you implementation of try_cycle for arbitrry morphism, afterwards it will be removed. CanonToCanon should implement the Hash trait if we want to put it in the functor
    object::Object,
    Category,
    HomSet,
    PrettyName,
};
use rayon::prelude::*;
use std::{
    borrow::Borrow,
    collections::HashMap,
    fmt::{self, Debug, Display},
    hash::Hash,
    marker::{PhantomData, Send, Sync},
};

type Endos<M> = Vec<M>;
type RawIsoClass<M> = Vec<(M, Vec<M>)>;
type IsoClass<O, E> = HashMap<O, Vec<E>>;

#[derive(Debug)]
pub struct IsoClasses<O: Object, M: Morphism<O>> {
    pub buffer: Vec<IsoClass<O, M>>,
}

impl<O: Object + Hash + Clone + Sync + Send, M: Morphism<O> + Sync + Send> IsoClasses<O, M> {
    pub fn functor<const RECURSION_PARAMETER: usize>(category: &Category<O, M>) -> Self {
        //step 0. If recursion parameter is less than 2, it will lead to the undefined behaviour
        assert!(
            RECURSION_PARAMETER >= 2,
            "parameter of recursion cannot be less that 2!"
        );
        //step 1. Clone all the endomorphisms (we will need them to be owned)

        let endos: Endos<M> = category
            .hom_sets
            .par_iter()
            .flat_map(|(source, hom_sets_fixed_source)| {
                hom_sets_fixed_source
                    .par_iter()
                    .filter(move |(target, _)| *target == source)
                    .flat_map(|(_, morphisms)| {
                        morphisms.par_iter().map(|morphism| morphism.clone())
                    })
            })
            .collect();

        //step 2. generate raw szymczak classes (by raw i mean they are unsorted by object and endomorphisms keep their cycles)
        let raw_iso_classes = Self::raw_functor::<{ RECURSION_PARAMETER }>(endos, category);

        //step 3. clean up the szymczak classes
        let buffer: Vec<IsoClass<O, M>> =
            raw_iso_classes.into_par_iter().map(Self::clean).collect();

        Self { buffer }
    }

    fn raw_functor<const RECURSION_PARAMETER: usize>(
        mut endos: Endos<M>,
        category: &Category<O, M>,
    ) -> Vec<RawIsoClass<M>> {
        if endos.len() > RECURSION_PARAMETER {
            let left_endos = endos.split_off(endos.len() / 2);
            let right_endos = endos;
            let (left_raw_iso_classes, right_raw_iso_classes) = rayon::join(
                || Self::raw_functor::<{ RECURSION_PARAMETER }>(left_endos, category),
                || Self::raw_functor::<{ RECURSION_PARAMETER }>(right_endos, category),
            );

            Self::merge_raw_iso_classes(left_raw_iso_classes, right_raw_iso_classes, category)
        } else {
            Self::raw_functor_final_step(endos, category)
        }
    }

    fn raw_functor_final_step(endos: Endos<M>, category: &Category<O, M>) -> Vec<RawIsoClass<M>> {
        let endos_wrapped = endos.into_iter().map(move |endo| {
            let cycle: Vec<M> = endo.try_cycle().expect("It should be an endomorphism"); //Wrapper::from(endo)
            (endo, cycle)
        });

        endos_wrapped.fold(
            Vec::<RawIsoClass<M>>::new(),
            |mut raw_iso_classes, (endo, cycle) /*Wrapper*/| {
                let maybe_raw_iso_class: Option<&mut RawIsoClass<M>> =
                    raw_iso_classes.par_iter_mut().find_any(|raw_iso_class| {
                        Self::are_szymczak_isomorphic(
                            //Wrapper
                            (&endo, &cycle),
                            util::transform(
                                raw_iso_class
                                    .get(0)
                                    .expect("szymczak classes are never empty"),
                            ),
                            category,
                        )
                    });
                if let Some(raw_iso_class) = maybe_raw_iso_class {
                    raw_iso_class.push((endo, cycle)); //Wrapper
                } else {
                    let new_raw_iso_class: RawIsoClass<M> = vec![(endo, cycle)]; //Wrapper
                    raw_iso_classes.push(new_raw_iso_class);
                }

                raw_iso_classes
            },
        )
    }

    fn merge_raw_iso_classes(
        mut left_raw_iso_classes: Vec<RawIsoClass<M>>,
        mut right_raw_iso_classes: Vec<RawIsoClass<M>>,
        category: &Category<O, M>,
    ) -> Vec<RawIsoClass<M>> {
        let mut merged_raw_iso_classes: Vec<RawIsoClass<M>> = left_raw_iso_classes.iter_mut().fold(
            Vec::<RawIsoClass<M>>::new(),
            |mut merged_raw_iso_classes, left_raw_iso_class| {
                if let Some(right_raw_iso_class) =
                    right_raw_iso_classes
                        .iter_mut()
                        .find(|right_raw_iso_class| {
                            let left_endo_wrapped = util::transform(
                                left_raw_iso_class
                                    .iter()
                                    .next()
                                    .expect("szymczak classes are never empty"),
                            );
                            let right_endo_wrapped = util::transform(
                                right_raw_iso_class
                                    .iter()
                                    .next()
                                    .expect("szymczak classes are never empty"),
                            );

                            Self::are_szymczak_isomorphic(
                                left_endo_wrapped,
                                right_endo_wrapped,
                                category,
                            )
                        })
                {
                    let mut merged_raw_iso_class = RawIsoClass::<M>::new();
                    merged_raw_iso_class.append(left_raw_iso_class);
                    merged_raw_iso_class.append(right_raw_iso_class);
                    merged_raw_iso_classes.push(merged_raw_iso_class);
                }

                right_raw_iso_classes.retain(|right_raw_iso_class| !right_raw_iso_class.is_empty());

                merged_raw_iso_classes
            },
        );
        left_raw_iso_classes.retain(|left_raw_szymczak_class| !left_raw_szymczak_class.is_empty());

        merged_raw_iso_classes.append(&mut left_raw_iso_classes);
        merged_raw_iso_classes.append(&mut right_raw_iso_classes);

        merged_raw_iso_classes
    }

    fn clean(raw_iso_class: RawIsoClass<M>) -> IsoClass<O, M> {
        raw_iso_class.into_iter().map(|(endo, _)| endo).fold(
            //Wrapper
            IsoClass::<O, M>::new(),
            |mut iso_class, endo: M| {
                iso_class
                    .entry(endo.source().borrow().clone())
                    .or_default()
                    .push(endo);
                iso_class
            },
        )
    }

    //the endomorphism is casted to M on purpose, to make M::compose make sense
    fn are_szymczak_isomorphic(
        left_endomorphism_with_cycle: (&M, &Vec<M>),
        right_endomorphism_with_cycle: (&M, &Vec<M>),
        category: &Category<O, M>,
    ) -> bool {
        let (l, l_cycle) = left_endomorphism_with_cycle;
        let (r, r_cycle) = right_endomorphism_with_cycle;

        let morphisms_l_to_r: &Vec<M> = category
            .hom_sets
            .get(l.target().borrow())
            .expect("there is a hom_set with the given object")
            .get(r.source().borrow())
            .expect("there is a hom_set with the given object");

        let morphisms_r_to_l: &Vec<M> = category
            .hom_sets
            .get(r.target().borrow())
            .expect("there is a hom_set with the given object")
            .get(l.source().borrow())
            .expect("there is a hom_set with the given object");

        for l_to_r in morphisms_l_to_r {
            for r_to_l in morphisms_r_to_l {
                /*
                println!(
                    "l_to_r: {:?}\nr_to_l: {:?}\n{:?} == {:?}\n{:?} == {:?}\nl_cycle: {:?}\nr_cycle: {:?}\n",
                    l_to_r,
                    r_to_l,
                    l_to_r.compose(r),
                    l.compose(l_to_r),
                    r_to_l.compose(l),
                    r.compose(r_to_l), l_cycle, r_cycle
                );
                */
                if
                //l -> r
                l_to_r.compose(r) == l.compose(l_to_r)
                //r -> l
                && r_to_l.compose(l) == r.compose(r_to_l)
                //identity on l
                && Self::is_identity(&l_to_r.compose(r_to_l), l_cycle)
                //identity on r
                && Self::is_identity(&r_to_l.compose(l_to_r), r_cycle)
                {
                    return true;
                }
            }
        }
        false
    }

    fn is_identity(morphism: &M, cycle: &Vec<M>) -> bool {
        for en in cycle {
            let en_morphism = morphism.compose(en);

            for em in cycle {
                if en_morphism == *em {
                    return true;
                }
            }
        }

        false
    }
}
mod util {

    pub const fn transform<L, R>(reference_to_tuple: &(L, R)) -> (&L, &R) {
        let (left, right) = reference_to_tuple;
        (&left, &right)
    }
}

/*
impl<
        O: Object + Display + PrettyName,
        M: Morphism<O>,
        E: EndoMorphism<O> + Debug + IsMap<O> + IsBij<O> + PrettyName,
    > Display for SzymczakCategory<O, M, E>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let number_of_endomorphisms =
            self.szymczak_classes
                .iter()
                .fold(0, |curr_no_out, szymczak_class| {
                    curr_no_out
                        + szymczak_class
                            .values()
                            .fold(0, |curr_no_in, endos: &Vec<E>| curr_no_in + endos.len())
                });

        let mut string = String::new();

        string.push_str(&format!("Functor name: {}\nObject: {}\nEndomorphism: {}\nNumber of endomorphisms: {}\nNumber of classes: {}\nEvery class has a map: {}\nEvery class has a bijection: {}\nEvery class has exactly one bijection: {}\n===\n", Self::PRETTY_NAME, O::PRETTY_NAME, E::PRETTY_NAME, number_of_endomorphisms, self.szymczak_classes.len(), self.map_in_every_class(), self.bijection_in_every_class(), self.one_bijection_in_every_class()));

        for szymczak_class in &self.szymczak_classes {
            string.push_str("---\n");
            for (object, endomorphisms) in szymczak_class {
                string.push_str(&format!("-\n{object}:\n"));
                for endomorphism in endomorphisms {
                    string.push_str(&format!("{endomorphism:?}"));
                    string.push('\n');
                }
            }
        }
        write!(f, "{string}")
    }
}

impl<O: Object, M: Morphism<O>, E: EndoMorphism<O>> PrettyName for SzymczakCategory<O, M, E> {
    const PRETTY_NAME: &'static str = "Szymczak";
}


impl<O: Object, M: Morphism<O>, E: EndoMorphism<O> + IsMap<O>> SzymczakCategory<O, M, E> {
    pub fn map_in_every_class(&self) -> bool {
        self.szymczak_classes.iter().all(|szymczak_class| {
            szymczak_class
                .values()
                .any(|endomorphisms| endomorphisms.iter().any(IsMap::<O>::is_a_map))
        })
    }
}

impl<O: Object, M: Morphism<O>, E: EndoMorphism<O> + IsBij<O>> SzymczakCategory<O, M, E> {
    pub fn bijection_in_every_class(&self) -> bool {
        self.szymczak_classes.iter().all(|szymczak_class| {
            szymczak_class
                .values()
                .any(|endomorphisms| endomorphisms.iter().any(IsBij::<O>::is_a_bijection))
        })
    }

    pub fn one_bijection_in_every_class(&self) -> bool {
        self.szymczak_classes.iter().all(|szymczak_class| {
            szymczak_class.values().any(|endomorphisms| {
                endomorphisms
                    .iter()
                    .filter(|endomorphism| endomorphism.is_a_bijection())
                    .count()
                    == 1
            })
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        category::{object::Concrete, relation::Relation, Category},
        ralg::{
            cgroup::{ideal::CIdeal, C},
            module::canon::object::Object as CanonModule,
        },
    };

    #[test]
    fn szymczak_isomorphism_is_equivalence() {
        use typenum::{Unsigned, U5 as N};
        type R = C<N>;
        type I = CIdeal<N>;

        let category = Category::<CanonModule<R, I>, Relation<R, I>>::new(1);
        let zn: CanonModule<R, I> = category
            .clone()
            .into_objects()
            .into_iter()
            .find(|object| object.cardinality() == N::to_usize())
            .expect("there is a module of given cardinality");
        let hom_set_zn_zn: Vec<Relation<R, I>> = category.hom_set(&zn, &zn);

        assert_eq!(
            hom_set_zn_zn.get(0).unwrap().source(),
            hom_set_zn_zn.get(0).unwrap().target()
        );

        //reflexive
        for index in 0..hom_set_zn_zn.len() {
            let endo = hom_set_zn_zn.get(index).unwrap();

            let endo_with_cycle = (endo, &endo.cycle());

            assert!(SzymczakCategory::are_szymczak_isomorphic(
                endo_with_cycle,
                endo_with_cycle,
                &category.hom_sets
            ));
        }

        //symmetric
        for index_0 in 0..hom_set_zn_zn.len() {
            let endo_0 = hom_set_zn_zn.get(index_0).unwrap();
            let endo_with_cycle_0 = (endo_0, &endo_0.cycle());

            for index_1 in 0..hom_set_zn_zn.len() {
                let endo_1 = hom_set_zn_zn.get(index_1).unwrap();
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
            let endo_0 = hom_set_zn_zn.get(index_0).unwrap();
            let endo_with_cycle_0 = (endo_0, &endo_0.cycle());

            for index_1 in 0..hom_set_zn_zn.len() {
                let endo_1 = hom_set_zn_zn.get(index_1).unwrap();
                let endo_with_cycle_1 = (endo_1, &endo_1.cycle());
                for index_2 in 0..hom_set_zn_zn.len() {
                    let endo_2 = hom_set_zn_zn.get(index_2).unwrap();
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

        type R = C<N>;
        type I = CIdeal<N>;

        let category = Category::<CanonModule<R, I>, Relation<R, I>>::new(1);
        let zn: CanonModule<R, I> = category
            .clone()
            .into_objects()
            .into_iter()
            .find(|object| object.cardinality() == N::to_usize())
            .expect("there is a module of given cardinality");
        let hom_set_zn_zn: Vec<Relation<R, I>> = category.hom_set(&zn, &zn);

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
    fn is_identity() {
        use typenum::{Unsigned, U2 as P};
        type R = C<P>;
        type I = CIdeal<P>;
        let p = P::to_usize();

        let category = Category::<CanonModule<R, I>, Relation<R, I>>::new(1);

        let all_objects = category.clone().into_objects();

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

        let mut z1_to_z1 = category.hom_set(&z1, &z1);
        let z2_to_z2 = category.hom_set(&z2, &z2);

        let top_z1 = z1_to_z1
            .pop()
            .expect("there is only top relation on z1")
            .clone();
        let top_z2 = z2_to_z2
            .iter()
            .find(|endo| endo.matrix.buffer() == vec![true, true, true, true])
            .expect("there is the top relation on z2")
            .clone();

        let top_z1_cycle = top_z1.cycle();
        let top_z2_cycle = top_z2.cycle();

        assert_eq!(top_z1_cycle.len(), 1);
        assert_eq!(*top_z1_cycle.get(0).unwrap(), top_z1);

        assert_eq!(top_z2_cycle.len(), 1);
        assert_eq!(*top_z2_cycle.get(0).unwrap(), top_z2);

        let morphisms_top_z1_to_top_z2 =
            category.hom_set(top_z1.target().borrow(), top_z2.source().borrow());
        let morphisms_top_z2_to_top_z1 =
            category.hom_set(top_z2.target().borrow(), top_z1.source().borrow());

        let mut are_szymczak_isomorphic: bool = false;
        let mut are_there_morphisms: bool = false;

        for top_z1_to_top_z2 in &morphisms_top_z1_to_top_z2 {
            for top_z2_to_top_z1 in &morphisms_top_z2_to_top_z1 {
                if top_z1.compose(top_z1_to_top_z2) == top_z1_to_top_z2.compose(&top_z2)
                    && top_z2.compose(top_z2_to_top_z1) == top_z2_to_top_z1.compose(&top_z1)
                {
                    are_there_morphisms = true;

                    if SzymczakCategory::<
                        CanonModule<R, I>,
                        Relation<R, I>,
                        Relation<R, I>,
                    >::is_identity(
                        &top_z1_to_top_z2.compose(top_z2_to_top_z1), &top_z1_cycle
                    ) && SzymczakCategory::<
                        CanonModule<R, I>,
                        Relation<R, I>,
                        Relation<R, I>,
                    >::is_identity(
                        &top_z2_to_top_z1.compose(top_z1_to_top_z2), &top_z2_cycle
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
    fn szymczak_isomorphism_different_base_objects() {
        use typenum::{Unsigned, U2 as P};
        type R = C<P>;
        type I = CIdeal<P>;
        let p = P::to_usize();

        let category = Category::<CanonModule<R, I>, Relation<R, I>>::new(1);

        let all_objects = category.clone().into_objects();
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

        let mut z1_to_z1 = category.hom_set(&z1, &z1);
        let z2_to_z2 = category.hom_set(&z2, &z2);

        let top_z1 = z1_to_z1.pop().expect("there is only top relation on z1");
        let top_z2 = z2_to_z2
            .iter()
            .find(|endo| endo.matrix.buffer() == vec![true, true, true, true])
            .expect("there is the top relation on z2")
            .clone();

        assert_eq!(top_z1.matrix.buffer(), vec![true]);
        assert_eq!(top_z2.matrix.buffer(), vec![true, true, true, true]);

        let top_z1_with_cycle = (&top_z1, &top_z1.cycle());
        let top_z2_with_cycle = (&top_z2, &top_z2.cycle());

        assert_eq!(top_z1_with_cycle.1.len(), 1);
        assert_eq!(top_z1_with_cycle.0, top_z1_with_cycle.1.get(0).unwrap());
        assert_eq!(top_z2_with_cycle.1.len(), 1);
        assert_eq!(top_z2_with_cycle.0, top_z2_with_cycle.1.get(0).unwrap());

        assert!(SzymczakCategory::are_szymczak_isomorphic(
            top_z1_with_cycle,
            top_z2_with_cycle,
            &category.hom_sets
        ));
    }

    #[test]
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
        /*
        for endo in endomorphisms.iter() {
            println!("{:?}", endo);
        }
        */

        let endomorphisms_with_cycles: EndoMorphismsWithCycles<Relation<R, I>, Relation<R, I>> =
            endomorphisms
                .into_iter()
                .map(|endomorphism| {
                    let cycle: Vec<Relation<R, I>> = endomorphism.cycle();
                    (endomorphism, cycle)
                })
                .collect();

        //let endomorphisms_with_cycles_len = endomorphisms_with_cycles.len();

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
                        util::transform(
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

        /*
        println!("\n\nAFTER GENERATION:");
        for raw_szymczak_class in raw_szymczak_classes.iter() {
            println!("new szymczak class:");

            for endo in raw_szymczak_class.iter() {
                println!("endo:{:?}", endo.0);
            }
        }
        */
        assert_eq!(raw_szymczak_classes.len(), p);
    }

    #[test]
    fn merge_raw_szymczak_classes() {
        use typenum::{Unsigned, U5 as P};
        type R = C<P>;
        type I = CIdeal<P>;
        const RECURSION_PARAMETER: usize = 2;
        let p = P::to_usize();

        let category = Category::<CanonModule<R, I>, Relation<R, I>>::new(1);

        let szymczak_category =
            SzymczakCategory::<CanonModule<R, I>, Relation<R, I>, Relation<R, I>>::szymczak_functor::<
                { RECURSION_PARAMETER },
            >(&category);
        assert_eq!(szymczak_category.szymczak_classes.len(), p);
    }

    macro_rules! generate_test_szymczak_functor_zp {
        ($name:ident, $p:ident) => {
            #[test]
            fn $name() {
                use typenum::{$p, Unsigned};
                type R = C<$p>;
                type I = CIdeal<$p>;
                let p = $p::to_usize();

                let category = Category::<CanonModule<R, I>, Relation<R, I>>::new(1);
                let szymczak_category = SzymczakCategory::<
                    CanonModule<R, I>,
                    Relation<R, I>,
                    Relation<R, I>,
                >::szymczak_functor::<20>(&category);
                assert_eq!(szymczak_category.szymczak_classes.len(), p);
            }
        };
    }

    generate_test_szymczak_functor_zp!(szymczak_functor_z2, U2);
    generate_test_szymczak_functor_zp!(szymczak_functor_z3, U3);
    generate_test_szymczak_functor_zp!(szymczak_functor_z5, U5);
    generate_test_szymczak_functor_zp!(szymczak_functor_z7, U7);
    generate_test_szymczak_functor_zp!(szymczak_functor_z11, U11);
}

*/
