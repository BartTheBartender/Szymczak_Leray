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

//a trait to store the endomorphisms with additional structure used to determine if two endomorphisms are equivalent
pub trait Wrapper<O: Object + Hash, M: Morphism<O>>: Sized {
    fn from_morphism(morphism: M) -> Option<Self>;
    fn into_morphism(self) -> M;

    fn are_isomorphic(left: &Self, right: &Self, category: &Category<O, M>) -> bool;
}

type Endos<M> = Vec<M>;
type RawIsoClass<W> = Vec<W>;
type IsoClass<O, M> = HashMap<O, Vec<M>>;

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
            W::from_morphism(endo).expect("This morphism should be an endomorphism")
        });

        endos_wrapped.fold(
            Vec::<RawIsoClass<W>>::new(),
            |mut raw_iso_classes, endo_wrapped /*Wrapper*/| {
                let maybe_raw_iso_class: Option<&mut RawIsoClass<W>> =
                    raw_iso_classes.par_iter_mut().find_any(|raw_iso_class| {
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

//-----------------------------------------------------------------------------------------
pub trait WrapperAllIsos<O: Object + Hash, M: Morphism<O>>: Wrapper<O, M> + Clone {
    fn all_isos(left: &Self, right: &Self, category: &Category<O, M>) -> Vec<(M, M)>;
}

pub struct IsomorphicPair<O: Object, M: Morphism<O>> {
    pub left: M,
    pub right: M,
    pub isos: Vec<(M, M)>,
    object_type: PhantomData<O>,
}

pub type IsoClassAllIsos<O, M> = Vec<IsomorphicPair<O, M>>;

pub struct IsoClassesAllIsos<O: Object + Hash, M: Morphism<O>, W: WrapperAllIsos<O, M>> {
    pub buffer: Vec<IsoClassAllIsos<O, M>>,
    wrapper_all_isos: PhantomData<W>,
}

impl<
        O: Object + Hash + Clone + Sync + Send,
        M: Morphism<O> + Sync + Send + IsBij<O>,
        W: WrapperAllIsos<O, M> + Sync + Send,
    > IsoClassesAllIsos<O, M, W>
{
    pub fn all_isos(iso_classes: IsoClasses<O, M, W>, category: &Category<O, M>) -> Self {
        let buffer: Vec<_> = iso_classes
            .buffer
            .into_iter()
            .map(|iso_class: IsoClass<O, M>| Self::all_isos_class(iso_class, category))
            .collect();

        Self {
            buffer,
            wrapper_all_isos: PhantomData::<W>,
        }
    }

    fn all_isos_class(
        iso_class: IsoClass<O, M>,
        category: &Category<O, M>,
    ) -> IsoClassAllIsos<O, M> {
        //
        let endos: Vec<M> = iso_class
            .into_values()
            .flat_map(IntoIterator::into_iter)
            .collect();

        let bijs: Vec<M> = endos
            .iter()
            .filter(|endo| endo.is_a_bijection())
            .map(Clone::clone)
            .collect();

        let endos_wrapped = endos
            .into_par_iter()
            .map(|endo| W::from_morphism(endo))
            .map(|endo_wrapped| endo_wrapped.expect("This morphism should be an endomorphism"));

        let bijs_wrapped: Vec<W> = bijs
            .into_iter()
            .map(|endo| W::from_morphism(endo))
            .map(|endo_wrapped| endo_wrapped.expect("This morphism should be an endomorphism"))
            .collect();

        endos_wrapped
            .flat_map(|endo_wrapped| {
                bijs_wrapped
                    .clone()
                    .into_par_iter()
                    .map(move |bij_wrapped| {
                        let isos = W::all_isos(&endo_wrapped, &bij_wrapped, category);
                        IsomorphicPair {
                            left: W::into_morphism(endo_wrapped.clone()),
                            right: W::into_morphism(bij_wrapped),
                            isos,
                            object_type: PhantomData::<O>,
                        }
                    })
            })
            .collect::<IsoClassAllIsos<O, M>>()
    }
}

impl<
        O: Object + Hash + Display + PrettyName,
        M: Morphism<O> + Debug + PrettyName,
        W: WrapperAllIsos<O, M>,
    > Display for IsoClassesAllIsos<O, M, W>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut string = String::new();

        string.push_str(&format!(
            "Functor name: {}\nObject: {}\nMorphism: {}\nNumber of classes: {}\n===\n",
            Self::PRETTY_NAME,
            O::PRETTY_NAME,
            M::PRETTY_NAME,
            self.buffer.len()
        ));

        for iso_class_all_isos in &self.buffer {
            string.push_str("---\n");
            for iso_pair in iso_class_all_isos {
                string.push_str("--\n");
                for iso_maps in &iso_pair.isos {
                    string.push_str(&format!(
                        "{}:{:?}-{}:{:?}--{:?}-{:?}\n",
                        iso_pair.left.source().borrow(),
                        iso_pair.left,
                        iso_pair.right.source().borrow(),
                        iso_pair.right,
                        iso_maps.0,
                        iso_maps.1
                    ));
                }
            }
        }
        write!(f, "{string}")
    }
}

impl<O: Object + Hash, M: Morphism<O>, W: WrapperAllIsos<O, M>> PrettyName
    for IsoClassesAllIsos<O, M, W>
{
    default const PRETTY_NAME: &'static str = "Not specified";
}
