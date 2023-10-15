use crate::category::{
    functors::{IsoClasses, Wrapper},
    morphism::Endo as Morphism,
    object::Object,
    Category, PrettyName,
};
use std::{borrow::Borrow, hash::Hash, marker::PhantomData};

pub struct SzymczakWrapper<O: Object + Hash, M: Morphism<O>> {
    morphism: M,
    cycle: Vec<M>,
    object_type: PhantomData<O>,
}

impl<O: Object + Hash, M: Morphism<O>> SzymczakWrapper<O, M> {
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

impl<O: Object + Hash, M: Morphism<O>> Wrapper<O, M> for SzymczakWrapper<O, M> {
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

pub type SzymczakClasses<O, M> = IsoClasses<O, M, SzymczakWrapper<O, M>>;

impl<O: Object + Hash, M: Morphism<O>> PrettyName for SzymczakClasses<O, M> {
    const PRETTY_NAME: &'static str = "Szymczak";
}

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
    type W = SzymczakWrapper<Module<R, I>, Relation<R, I>>;

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

        assert_eq!(top_z1_wrapped.cycle.len(), 1);
        assert_eq!(
            top_z1_wrapped.cycle.get(0).unwrap(),
            &top_z1_wrapped.morphism
        );

        assert_eq!(top_zn_wrapped.cycle.len(), 1);
        assert_eq!(
            top_zn_wrapped.cycle.get(0).unwrap(),
            &top_zn_wrapped.morphism
        );

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
