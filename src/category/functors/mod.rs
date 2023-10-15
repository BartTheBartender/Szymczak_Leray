use crate::category::{
    morphism::{Endo as Morphism, IsBij, IsMap}, //i leave to you implementation of try_cycle for arbitrry morphism, afterwards it will be removed. CanonToCanon should implement the Hash trait if we want to put it in the functor
    object::Object,
    Category,
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

pub mod szymczak;

type Endos<M> = Vec<M>;
type RawIsoClass<W> = Vec<W>;
type IsoClass<O, E> = HashMap<O, Vec<E>>;

//a trait to store the endomorphisms with additional structure used to determine if two endomorphisms are equivalent
pub trait Wrapper<O: Object + Hash, M: Morphism<O>>: Sized {
    fn from_morphism(morphism: M) -> Option<Self>;
    fn into_morphism(self) -> M;

    fn are_isomorphic(left: &Self, right: &Self, category: &Category<O, M>) -> bool;
}

#[derive(Debug)]
pub struct IsoClasses<O: Object + Hash, M: Morphism<O>, W: Wrapper<O, M>> {
    pub buffer: Vec<IsoClass<O, M>>,
    pub wrapper: PhantomData<W>,
}

impl<
        O: Object + Hash + Clone + Sync + Send,
        M: Morphism<O> + Sync + Send,
        W: Wrapper<O, M> + Sync + Send,
    > IsoClasses<O, M, W>
{
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

        Self {
            buffer,
            wrapper: PhantomData::<W>,
        }
    }

    fn raw_functor<const RECURSION_PARAMETER: usize>(
        mut endos: Endos<M>,
        category: &Category<O, M>,
    ) -> Vec<RawIsoClass<W>> {
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

    fn raw_functor_final_step(endos: Endos<M>, category: &Category<O, M>) -> Vec<RawIsoClass<W>> {
        let endos_wrapped = endos.into_iter().map(move |endo| {
            /*
            let cycle: Vec<M> = endo.try_cycle().expect("It should be an endomorphism"); //Wrapper::from(endo)
            (endo, cycle)
            */
            W::from_morphism(endo).expect("This morphism should be an endomorphism")
        });

        endos_wrapped.fold(
            Vec::<RawIsoClass<W>>::new(),
            |mut raw_iso_classes, endo_wrapped /*Wrapper*/| {
                let maybe_raw_iso_class: Option<&mut RawIsoClass<W>> =
                    raw_iso_classes.par_iter_mut().find_any(|raw_iso_class| {
                        /*
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
                        */
                        W::are_isomorphic(
                            &endo_wrapped,
                            raw_iso_class.get(0).expect("RawIsoClass is never empty"),
                            category,
                        )
                    });
                if let Some(raw_iso_class) = maybe_raw_iso_class {
                    raw_iso_class.push(endo_wrapped); //Wrapper
                } else {
                    let new_raw_iso_class: RawIsoClass<W> = vec![endo_wrapped]; //Wrapper
                    raw_iso_classes.push(new_raw_iso_class);
                }

                raw_iso_classes
            },
        )
    }

    fn merge_raw_iso_classes(
        mut left_raw_iso_classes: Vec<RawIsoClass<W>>,
        mut right_raw_iso_classes: Vec<RawIsoClass<W>>,
        category: &Category<O, M>,
    ) -> Vec<RawIsoClass<W>> {
        let mut merged_raw_iso_classes: Vec<RawIsoClass<W>> = left_raw_iso_classes.iter_mut().fold(
            Vec::<RawIsoClass<W>>::new(),
            |mut merged_raw_iso_classes, left_raw_iso_class| {
                if let Some(right_raw_iso_class) =
                    right_raw_iso_classes
                        .par_iter_mut()
                        .find_any(|right_raw_iso_class| {
                            W::are_isomorphic(
                                left_raw_iso_class
                                    .get(0)
                                    .expect("RawIsoClass is never empty"),
                                right_raw_iso_class
                                    .get(0)
                                    .expect("RawIsoClass is never empty"),
                                category,
                            )
                        })
                {
                    let mut merged_raw_iso_class = RawIsoClass::<W>::new();
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

    fn clean(raw_iso_class: RawIsoClass<W>) -> IsoClass<O, M> {
        raw_iso_class.into_iter().map(W::into_morphism).fold(
            //Wrapper
            IsoClass::<O, M>::new(),
            |mut iso_class: IsoClass<O, M>, endo: M| {
                iso_class
                    .entry(endo.source().borrow().clone())
                    .or_default()
                    .push(endo);
                iso_class
            },
        )
    }
}

impl<
        O: Object + Hash + Display + PrettyName,
        M: Morphism<O> + Debug + IsMap<O> + IsBij<O> + PrettyName,
        W: Wrapper<O, M>,
    > Display for IsoClasses<O, M, W>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let number_of_endos = self.buffer.iter().fold(0, |curr_no_out, iso_class| {
            curr_no_out
                + iso_class
                    .values()
                    .fold(0, |curr_no_in, endos: &Vec<M>| curr_no_in + endos.len())
        });

        let mut string = String::new();

        string.push_str(&format!("Functor name: {}\nObject: {}\nMorphism: {}\nNumber of endomorphisms: {}\nNumber of classes: {}\nEvery class has a map: {}\nEvery class has a bijection: {}\nEvery class has exactly one bijection: {}\n===\n", Self::PRETTY_NAME, O::PRETTY_NAME, M::PRETTY_NAME, number_of_endos, self.buffer.len(), self.map_in_every_class(), self.bijection_in_every_class(), self.one_bijection_in_every_class()));

        for iso_class in &self.buffer {
            string.push_str("---\n");
            for (object, endomorphisms) in iso_class {
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

impl<O: Object + Hash, M: Morphism<O>, W: Wrapper<O, M>> PrettyName for IsoClasses<O, M, W> {
    default const PRETTY_NAME: &'static str = "Not specified";
}

impl<O: Object + Hash, M: Morphism<O> + IsMap<O>, W: Wrapper<O, M>> IsoClasses<O, M, W> {
    pub fn map_in_every_class(&self) -> bool {
        self.buffer.iter().all(|iso_class| {
            iso_class
                .values()
                .any(|endos| endos.iter().any(IsMap::<O>::is_a_map))
        })
    }
}

impl<O: Object + Hash, M: Morphism<O> + IsBij<O>, W: Wrapper<O, M>> IsoClasses<O, M, W> {
    pub fn bijection_in_every_class(&self) -> bool {
        self.buffer.iter().all(|iso_class| {
            iso_class
                .values()
                .any(|endos| endos.iter().any(IsBij::<O>::is_a_bijection))
        })
    }

    pub fn one_bijection_in_every_class(&self) -> bool {
        self.buffer.iter().all(|iso_class| {
            iso_class
                .values()
                .flat_map(|endos| endos.iter())
                .filter(|endo| endo.is_a_bijection())
                .count()
                == 1
        })
    }
}
