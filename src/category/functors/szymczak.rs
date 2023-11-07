use crate::category::{
    functors::{IsoClasses, IsoClassesFull, IsoPair, Wrapper, WrapperFull},
    morphism::Endo as Morphism,
    object::Object,
    Category, PrettyName,
};
use std::{
    borrow::Borrow,
    fmt::{self, Debug, Display},
    hash::Hash,
    marker::PhantomData,
};

pub struct Szymczak<O: Object + Hash, M: Morphism<O>> {
    morphism: M,
    cycle: Vec<M>,
    object_type: PhantomData<O>,
}

impl<O: Object + Hash, M: Morphism<O>> Szymczak<O, M> {
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

impl<O: Object + Hash, M: Morphism<O>> Wrapper<O, M> for Szymczak<O, M> {
    fn from_morphism(morphism: M) -> Option<Self> {
        morphism.try_cycle().map(|cycle| Self {
            morphism,
            cycle,
            object_type: PhantomData::<O>,
        })
    }
    fn into_morphism(self) -> M {
        self.morphism
    }

    fn are_isomorphic(left: &Self, right: &Self, category: &Category<O, M>) -> bool {
        let l: &M = &left.morphism;
        let r: &M = &right.morphism;

        let morphisms_l_to_r: &Vec<M> = category
            .hom_sets
            .get(l.target().borrow())
            .expect("There are hom-sets with the given target")
            .get(r.source().borrow())
            .expect("There is a hom-set with a given source");

        let morphisms_r_to_l: &Vec<M> = category
            .hom_sets
            .get(r.target().borrow())
            .expect("There are hom-sets with the given target")
            .get(l.source().borrow())
            .expect("There is a hom-set with a given source");

        for l_to_r in morphisms_l_to_r {
            for r_to_l in morphisms_r_to_l {
                if
                //l -> r
                l_to_r.compose(r) == l.compose(l_to_r)
            //r -> l
            && r_to_l.compose(l) == r.compose(r_to_l)
            //identity on l
            && Self::is_identity(&l_to_r.compose(r_to_l), &left.cycle)
            //identity on r
            && Self::is_identity(&r_to_l.compose(l_to_r), &right.cycle)
                {
                    return true;
                }
            }
        }
        false
    }
}

pub type SzymczakClasses<O, M> = IsoClasses<O, M, Szymczak<O, M>>;

impl<O: Object + Hash, M: Morphism<O>> PrettyName for SzymczakClasses<O, M> {
    const PRETTY_NAME: &'static str = "Szymczak";
}
//-----------------------------------------------------------------------------------------

impl<O: Object + Hash + Clone, M: Morphism<O> + Clone> Clone for Szymczak<O, M> {
    fn clone(&self) -> Self {
        Self {
            morphism: self.morphism.clone(),
            cycle: self.cycle.clone(),
            object_type: PhantomData::<O>,
        }
    }
}

impl<O: Object + Hash, M: Morphism<O>> Szymczak<O, M> {
    fn is_identity_full(morphism: &M, cycle: &Vec<M>) -> Option<(usize, usize)> {
        for (n, en) in cycle.iter().enumerate() {
            let en_morphism = morphism.compose(en);

            for (m, em) in cycle.iter().enumerate() {
                if en_morphism == *em {
                    return Some((n, m));
                }
            }
        }

        None
    }
}

impl<O: Object + Hash + Clone + Send + Sync, M: Morphism<O> + Clone + Send + Sync> WrapperFull<O, M>
    for Szymczak<O, M>
{
    /*
       isos = Vec<((phi, psi), (k,l), (k', l'))> such that:
       * phi left = right phi
       * psi right = left psi
       * psi phi left^k = left^l
       * phi psi right^k' = right^l'
       where (k,l), (k',l') are minimal for (phi, psi)
    */
    type Isos = Vec<((M, M), (usize, usize), (usize, usize))>;
    fn all_isos(left: &Self, right: &Self, category: &Category<O, M>) -> Self::Isos {
        let l: &M = &left.morphism;
        let r: &M = &right.morphism;

        let morphisms_l_to_r: &Vec<M> = category
            .hom_sets
            .get(l.target().borrow())
            .expect("There are hom-sets with the given target")
            .get(r.source().borrow())
            .expect("There is a hom-set with a given source");

        let morphisms_r_to_l: &Vec<M> = category
            .hom_sets
            .get(r.target().borrow())
            .expect("There are hom-sets with the given target")
            .get(l.source().borrow())
            .expect("There is a hom-set with a given source");

        morphisms_l_to_r
            .iter()
            .fold(Self::Isos::new(), |mut isos: Self::Isos, l_to_r| {
                let good_morphisms_r_to_l = morphisms_r_to_l
                    .iter()
                    .filter(|r_to_l| {
                        l.compose(l_to_r) == l_to_r.compose(r)
                            && r.compose(r_to_l) == r_to_l.compose(l)
                    })
                    .map(|r_to_l| {
                        //obvious optimalization will be needed later
                        (
                            r_to_l,
                            Self::is_identity_full(&l_to_r.compose(r_to_l), &left.cycle),
                        )
                    })
                    .filter(|(_, id_l)| id_l.is_some())
                    .map(|(r_to_l, id_l)| {
                        (
                            r_to_l,
                            id_l,
                            Self::is_identity_full(&r_to_l.compose(l_to_r), &right.cycle),
                        )
                    })
                    .filter(|(_, _, id_r)| id_r.is_some());
                for (r_to_l, id_l, id_r) in good_morphisms_r_to_l {
                    isos.push((
                        (l_to_r.clone(), r_to_l.clone()),
                        id_l.expect("I have just checked it"),
                        id_r.expect("I have just checked it"),
                    ));
                }
                isos
            })
    }
}

pub type SzymczakClassesFull<O, M> = IsoClassesFull<O, M, Szymczak<O, M>>;

impl<O: Object + Hash + Clone + Send + Sync, M: Morphism<O> + Send + Sync> PrettyName
    for SzymczakClassesFull<O, M>
{
    const PRETTY_NAME: &'static str = "Szymczak (with all isomorphisms explicitly)";
}

impl<O: Object + Hash + Display + Clone + Send + Sync, M: Morphism<O> + Debug + Send + Sync> Display
    for IsoPair<O, M, Szymczak<O, M>>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut string = String::new();
        string.push_str(
            format!(
                "{}-{}-{:?}-{}--{}-{}-{:?}-{}#\n",
                self.left.source().borrow(),
                self.left.target().borrow(),
                self.left,
                self.left
                    .try_cycle()
                    .expect("it should be an endomorphism")
                    .len(),
                self.right.source().borrow(),
                self.right.target().borrow(),
                self.right,
                self.right
                    .try_cycle()
                    .expect("it should be an endomorphism")
                    .len()
            )
            .as_str(),
        );

        for tuple in &self.isos {
            string.push_str(
                format!(
                    "{}-{}-{:?}--{}-{}-{:?}---{}-{}--{}-{}\n",
                    tuple.0 .0.source().borrow(),
                    tuple.0 .0.target().borrow(),
                    tuple.0 .0,
                    tuple.0 .1.source().borrow(),
                    tuple.0 .1.target().borrow(),
                    tuple.0 .1,
                    tuple.1 .0,
                    tuple.1 .1,
                    tuple.2 .0,
                    tuple.2 .1
                )
                .as_str(),
            );
        }
        write!(f, "{string}")
    }
}

//-----------------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        category::{morphism::Morphism, object::Concrete, relation::Relation, Category},
        ralg::{
            cgroup::{ideal::CIdeal, C},
            module::canon::object::Object as Module,
        },
    };
    use typenum::{Unsigned, U5 as N};

    type R = C<N>;
    type I = CIdeal<N>;
    type W = Szymczak<Module<R, I>, Relation<R, I>>;

    #[test]
    fn szymczak_isomorphism_is_equivalence() {
        use typenum::{Unsigned, U5 as N};

        let category = Category::<Module<R, I>, Relation<R, I>>::new(1);
        let zn: Module<R, I> = category
            .clone()
            .into_objects()
            .into_iter()
            .find(|object| object.cardinality() == N::to_usize())
            .expect("there is a module of given cardinality");
        let hom_set_zn_zn: Vec<Relation<R, I>> = category.hom_set(&zn, &zn);

        assert_eq!(
            hom_set_zn_zn.get(0).unwrap().source().as_ref(),
            hom_set_zn_zn.get(0).unwrap().target().as_ref()
        );

        //reflexive
        for endo in &hom_set_zn_zn {
            let endo_wrapped = W::from_morphism(endo.clone()).unwrap();
            assert!(W::are_isomorphic(&endo_wrapped, &endo_wrapped, &category));
        }

        //symmetric
        for endo_0 in &hom_set_zn_zn {
            let endo_0_wrapped = W::from_morphism(endo_0.clone()).unwrap();

            for endo_1 in &hom_set_zn_zn {
                let endo_1_wrapped = W::from_morphism(endo_1.clone()).unwrap();

                if W::are_isomorphic(&endo_0_wrapped, &endo_1_wrapped, &category) {
                    assert!(W::are_isomorphic(
                        &endo_1_wrapped,
                        &endo_0_wrapped,
                        &category
                    ));
                }
            }
        }

        //transitive
        for endo_0 in &hom_set_zn_zn {
            let endo_0_wrapped = W::from_morphism(endo_0.clone()).unwrap();

            for endo_1 in &hom_set_zn_zn {
                let endo_1_wrapped = W::from_morphism(endo_1.clone()).unwrap();

                for endo_2 in &hom_set_zn_zn {
                    let endo_2_wrapped = W::from_morphism(endo_2.clone()).unwrap();

                    if W::are_isomorphic(&endo_0_wrapped, &endo_1_wrapped, &category)
                        && W::are_isomorphic(&endo_1_wrapped, &endo_2_wrapped, &category)
                    {
                        assert!(W::are_isomorphic(
                            &endo_1_wrapped,
                            &endo_2_wrapped,
                            &category,
                        ));
                    }
                }
            }
        }
    }

    #[test]
    fn szymczak_isomorphism_isnt_identically_true_nor_false() {
        let category = Category::<Module<R, I>, Relation<R, I>>::new(1);
        let zn: Module<R, I> = category
            .clone()
            .into_objects()
            .into_iter()
            .find(|object| object.cardinality() == N::to_usize())
            .expect("there is a module of given cardinality");
        let hom_set_zn_zn: Vec<Relation<R, I>> = category.hom_set(&zn, &zn);

        assert_eq!(hom_set_zn_zn.len(), N::to_usize() + 3);

        for morphism in &hom_set_zn_zn {
            assert_eq!(morphism.source(), morphism.target());
        }

        let mut is_sometimes_true: bool = false;
        let mut is_sometimes_false: bool = false;

        for endo_0 in &hom_set_zn_zn {
            let endo_0_wrapped = W::from_morphism(endo_0.clone()).unwrap();
            for endo_1 in &hom_set_zn_zn {
                let endo_1_wrapped = W::from_morphism(endo_1.clone()).unwrap();

                if endo_0 != endo_1 {
                    if W::are_isomorphic(&endo_0_wrapped, &endo_1_wrapped, &category) {
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
        let category = Category::<Module<R, I>, Relation<R, I>>::new(1);

        let all_objects = category.clone().into_objects();

        let z1 = all_objects
            .iter()
            .find(|object| object.cardinality() == 1)
            .expect("there is a trivial module")
            .clone();
        let zn = all_objects
            .iter()
            .find(|object| object.cardinality() == N::to_usize())
            .expect("there is zn module")
            .clone();

        let mut z1_to_z1 = category.hom_set(&z1, &z1);
        let zn_to_zn = category.hom_set(&zn, &zn);

        let top_z1 = z1_to_z1.pop().expect("there is only top relation on z1");
        let top_zn = zn_to_zn
            .iter()
            .find(|endo| endo.matrix.buffer() == vec![true; N::to_usize() * N::to_usize()])
            .expect("there is the top relation on zn")
            .clone();

        let top_z1_wrapped = W::from_morphism(top_z1).unwrap();
        let top_zn_wrapped = W::from_morphism(top_zn).unwrap();

        let morphisms_top_z1_to_top_zn = category.hom_set(
            top_z1_wrapped.morphism.target().borrow(),
            top_zn_wrapped.morphism.source().borrow(),
        );
        let morphisms_top_zn_to_top_z1 = category.hom_set(
            top_zn_wrapped.morphism.target().borrow(),
            top_z1_wrapped.morphism.source().borrow(),
        );

        let mut are_szymczak_isomorphic: bool = false;
        let mut are_there_morphisms: bool = false;

        for top_z1_to_top_zn in &morphisms_top_z1_to_top_zn {
            for top_zn_to_top_z1 in &morphisms_top_zn_to_top_z1 {
                if top_z1_wrapped.morphism.compose(top_z1_to_top_zn)
                    == top_z1_to_top_zn.compose(&top_zn_wrapped.morphism)
                    && top_zn_wrapped.morphism.compose(top_zn_to_top_z1)
                        == top_zn_to_top_z1.compose(&top_z1_wrapped.morphism)
                {
                    are_there_morphisms = true;

                    if W::is_identity(
                        &top_z1_to_top_zn.compose(top_zn_to_top_z1),
                        &top_z1_wrapped.cycle,
                    ) && W::is_identity(
                        &top_zn_to_top_z1.compose(top_z1_to_top_zn),
                        &top_zn_wrapped.cycle,
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
        let category = Category::<Module<R, I>, Relation<R, I>>::new(1);

        let all_objects = category.clone().into_objects();
        assert_eq!(all_objects.len(), 2);

        let z1 = all_objects
            .iter()
            .find(|object| object.cardinality() == 1)
            .expect("there is z1 module")
            .clone();
        let zn = all_objects
            .iter()
            .find(|object| object.cardinality() == N::to_usize())
            .expect("there is zn module")
            .clone();

        let mut z1_to_z1 = category.hom_set(&z1, &z1);
        let zn_to_zn = category.hom_set(&zn, &zn);

        let top_z1 = z1_to_z1.pop().expect("there is only top relation on z1");
        let top_zn = zn_to_zn
            .iter()
            .find(|endo| endo.matrix.buffer() == vec![true; N::to_usize() * N::to_usize()])
            .expect("there is the top relation on zn")
            .clone();

        assert_eq!(top_z1.matrix.buffer(), vec![true]);
        assert_eq!(
            top_zn.matrix.buffer(),
            vec![true; N::to_usize() * N::to_usize()]
        );

        let top_z1_wrapped = W::from_morphism(top_z1).unwrap();
        let top_zn_wrapped = W::from_morphism(top_zn).unwrap();

        assert!(W::are_isomorphic(
            &top_z1_wrapped,
            &top_zn_wrapped,
            &category
        ));
    }

    macro_rules! generate_test_szymczak_functor_zp {
        ($name:ident, $p:ident) => {
            #[test]
            fn $name() {
                use typenum::{$p, Unsigned};
                type R = C<$p>;
                type I = CIdeal<$p>;
                let p = $p::to_usize();

                let category = Category::<Module<R, I>, Relation<R, I>>::new(1);
                let szymczak_classes =
                    SzymczakClasses::<Module<R, I>, Relation<R, I>>::functor::<20>(&category);
                assert_eq!(szymczak_classes.buffer.len(), p);
            }
        };
    }

    generate_test_szymczak_functor_zp!(szymczak_functor_z2, U2);
    generate_test_szymczak_functor_zp!(szymczak_functor_z3, U3);
    generate_test_szymczak_functor_zp!(szymczak_functor_z5, U5);
    generate_test_szymczak_functor_zp!(szymczak_functor_z7, U7);
    generate_test_szymczak_functor_zp!(szymczak_functor_z11, U11);
}
