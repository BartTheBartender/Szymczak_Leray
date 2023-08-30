#[allow(unused_imports)] // DELETE LATER
use crate::{
    category::{
        morphism::{Compose, EndoMorphism, Morphism},
        AllMorphisms, Category, Duplicate, HomSet,
    },
    rmodule::{
        canon::CanonModule,
        direct::DirectModule,
        map::CanonToCanon,
        ring::{Fin, SuperRing},
        torsion::CoeffTree,
        Module,
    },
    util::{category_of_relations::HelperData, matrix::Matrix},
    Int,
};

use bitvec::prelude::*;
use rayon::prelude::*;
use std::{
    collections::{HashMap, HashSet},
    fmt::{self, Debug},
    sync::Arc,
};
use typenum;

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Relation<R: SuperRing> {
    pub source: Arc<CanonModule<R>>,
    pub target: Arc<CanonModule<R>>,
    pub matrix: Matrix<Fin<typenum::U2>>,
}

impl<R: SuperRing> Debug for Relation<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "s:{:?}, t:{:?}, {:?}",
            self.source, self.target, self.matrix
        )
    }
}

impl<R: SuperRing> Morphism<CanonModule<R>, CanonModule<R>> for Relation<R> {
    fn source(&self) -> Arc<CanonModule<R>> {
        Arc::clone(&self.source)
    }

    fn target(&self) -> Arc<CanonModule<R>> {
        Arc::clone(&self.target)
    }
}

//other * self
impl<R: SuperRing> Compose<CanonModule<R>, CanonModule<R>, CanonModule<R>, Relation<R>>
    for Relation<R>
{
    type Output = Relation<R>;

    fn compose_unchecked(&self, other: &Relation<R>) -> Self::Output {
        Relation {
            source: Arc::clone(&self.source),
            target: Arc::clone(&other.target),
            matrix: self.matrix.compose_unchecked(&other.matrix),
        }
    }
}

impl<R: SuperRing>
    From<(
        Arc<CanonModule<R>>,
        Arc<CanonModule<R>>,
        CanonToCanon<R>,
        &HelperData<R>,
    )> for Relation<R>
{
    /**
    the morphism should be mono in order for this conversion to work
    although the implementation neglects to check this

    the morphism should be a submodule of the given module
    */
    fn from(
        input: (
            Arc<CanonModule<R>>,
            Arc<CanonModule<R>>,
            CanonToCanon<R>,
            &HelperData<R>,
        ),
    ) -> Self {
        type Bool = Fin<typenum::U2>; //tymczasowo
        let (source, target, submodule, helper_data) = input;

        let mut buffer: Vec<Bool> = vec![Bool::zero(); helper_data.capacity.into()]; //to

        let elements = submodule.image().into_iter().map(|element| {
            helper_data
                .torsion_coeffs_vec
                .iter()
                .zip(element.into_values())
                .map(|(tc, x)| x.get() % tc)
                .collect::<Vec<Int>>()
        });

        let _ = elements.map(|element| {
            let buffer_index: Int = helper_data
                .indices
                .iter()
                .zip(element.into_iter())
                .map(|(index, x)| x * index)
                .sum::<Int>();

            buffer[buffer_index as usize] = Bool::one();
        });

        Relation {
            source,
            target,
            matrix: Matrix::from_buffer(buffer, helper_data.cols as u8, helper_data.rows as u8),
        }
    }
}

impl<R: SuperRing + std::hash::Hash> EndoMorphism<CanonModule<R>> for Relation<R> {}

impl<R: SuperRing> AllMorphisms<CanonModule<R>> for Relation<R> {
    fn hom_set(source: Arc<CanonModule<R>>, target: Arc<CanonModule<R>>) -> Vec<Relation<R>> {
        let direct = DirectModule::<R>::sumproduct(&source, &target);

        let helper_data = HelperData::<R>::new(&direct);
        direct
            .submodules_goursat()
            .map(|submodule| {
                Relation::<R>::from((
                    Arc::clone(&source),
                    Arc::clone(&target),
                    submodule,
                    &helper_data,
                ))
            })
            .collect::<Vec<Relation<R>>>()
    }
}

#[cfg(test)]
mod test {
    use crate::{
        category::{
            morphism::{Compose, Morphism},
            relation::Relation,
            Category, Duplicate,
        },
        error::Error,
        rmodule::{
            canon::CanonModule,
            direct::DirectModule,
            map::CanonToCanon,
            ring::{Fin, Ring, Set},
            torsion::CoeffTree,
        },
        util::category_of_relations::HelperData,
        Int,
    };
    use bitvec::prelude::*;
    use std::{collections::HashMap, sync::Arc};

    #[test]
    fn relation_composition_z5() {
        use typenum::U5 as N;
        type R = Fin<N>;
        let category = Category::<CanonModule<R>, Relation<R>>::new(1);

        let mut relations: Vec<Relation<R>> = category
            .hom_sets
            .iter()
            .filter(|(source, _)| source.cardinality() > 1)
            .map(|(_, hom_sets_fixed_source)| hom_sets_fixed_source)
            .next()
            .expect("there is non-trivial source")
            .iter()
            .filter(|(target, _)| target.cardinality() > 1)
            .map(|(_, relations_iter)| relations_iter)
            .next()
            .expect("there is non-trivial target")
            .to_vec();

        assert_eq!(relations.len(), 8);
        let top = relations.pop().expect("there are exactly eight relations");
        let four = relations.pop().expect("there are exactly eight relations");
        let three = relations.pop().expect("there are exactly eight relations");
        let two = relations.pop().expect("there are exactly eight relations");
        let one = relations.pop().expect("there are exactly eight relations");
        let zero = relations.pop().expect("there are exactly eight relations");
        let zero_dagger = relations.pop().expect("there are exactly eight relations");
        let bottom = relations.pop().expect("there are exactly eight relations");

        assert_eq!(relations.len(), 0);
        assert_eq!(zero.source(), one.source());
        assert_eq!(one.source(), top.target());

        //36 = 8 + 7 + 6 + 5 + 4 + 3 + 2 + 1

        //8
        assert_eq!(bottom.compose_unchecked(&bottom), bottom);
        assert_eq!(bottom.compose_unchecked(&zero_dagger), zero_dagger);
        assert_eq!(bottom.compose_unchecked(&zero), bottom);
        assert_eq!(bottom.compose_unchecked(&one), bottom);
        assert_eq!(bottom.compose_unchecked(&two), bottom);
        assert_eq!(bottom.compose_unchecked(&three), bottom);
        assert_eq!(bottom.compose_unchecked(&four), bottom);
        assert_eq!(bottom.compose_unchecked(&top), zero_dagger);

        //7
        assert_eq!(zero_dagger.compose_unchecked(&zero_dagger), zero_dagger);
        assert_eq!(zero_dagger.compose_unchecked(&zero), bottom);
        assert_eq!(zero_dagger.compose_unchecked(&one), zero_dagger);
        assert_eq!(zero_dagger.compose_unchecked(&two), zero_dagger);
        assert_eq!(zero_dagger.compose_unchecked(&three), zero_dagger);
        assert_eq!(zero_dagger.compose_unchecked(&four), zero_dagger);
        assert_eq!(zero_dagger.compose_unchecked(&top), zero_dagger);

        //6
        assert_eq!(zero.compose_unchecked(&zero), zero);
        assert_eq!(zero.compose_unchecked(&one), zero);
        assert_eq!(zero.compose_unchecked(&two), zero);
        assert_eq!(zero.compose_unchecked(&three), zero);
        assert_eq!(zero.compose_unchecked(&four), zero);
        assert_eq!(zero.compose_unchecked(&top), top);

        //5
        assert_eq!(one.compose_unchecked(&one), one);
        assert_eq!(one.compose_unchecked(&two), two);
        assert_eq!(one.compose_unchecked(&three), three);
        assert_eq!(one.compose_unchecked(&four), four);
        assert_eq!(one.compose_unchecked(&top), top);

        //4
        assert_eq!(two.compose_unchecked(&two), four);
        assert_eq!(two.compose_unchecked(&three), one);
        assert_eq!(two.compose_unchecked(&four), three);
        assert_eq!(two.compose_unchecked(&top), top);

        //3
        assert_eq!(three.compose_unchecked(&three), four);
        assert_eq!(three.compose_unchecked(&four), two);
        assert_eq!(three.compose_unchecked(&top), top);

        //2
        assert_eq!(four.compose_unchecked(&four), one);
        assert_eq!(four.compose_unchecked(&top), top);

        //1
        assert_eq!(top.compose_unchecked(&top), top);
    }

    #[test]
    #[ignore]
    fn z3_category_step_by_step() {
        use typenum::U3 as N;
        type R = Fin<N>;

        let zn_module: Arc<CanonModule<R>> = Arc::new(
            CoeffTree::<R, ()>::all_torsion_coeffs(1)
                .into_iter()
                .map(|torsion_coeffs| CanonModule::new(torsion_coeffs))
                .next()
                .unwrap(),
        );

        let direct = DirectModule::<R>::sumproduct(
            &Arc::clone(&zn_module),
            &Arc::new(zn_module.duplicate()),
        );

        let submodules = direct.submodules_goursat();
        let helper_data = HelperData::<R>::new(&direct);

        let relations_on_zn: Vec<Relation<R>> = submodules
            .into_iter()
            .map(|submodule| {
                Relation::<R>::from((direct.left(), direct.right(), submodule, &helper_data))
            })
            .collect();

        let bottom = vec![1, 0, 0, 0, 0, 0, 0, 0, 0];
        let zero_dagger = vec![1, 0, 0, 1, 0, 0, 1, 0, 0];
        let zero = vec![1, 1, 1, 0, 0, 0, 0, 0, 0];
        let one = vec![1, 0, 0, 0, 1, 0, 0, 0, 1];
        let two = vec![1, 0, 0, 0, 0, 1, 0, 1, 0];
        let top = vec![1, 1, 1, 1, 1, 1, 1, 1, 1];

        assert!(relations_on_zn
            .iter()
            .find(|relation| relation
                .matrix
                .buffer()
                .iter()
                .map(|x| x.get())
                .collect::<Vec<Int>>()
                == bottom)
            .is_some());
        /*assert!(relations_on_zn
            .iter()
            .find(|relation| relation.matrix.buffer() == zero_dagger)
            .is_some());
        assert!(relations_on_zn
            .iter()
            .find(|relation| relation.matrix.buffer() == zero)
            .is_some());
        assert!(relations_on_zn
            .iter()
            .find(|relation| relation.matrix.buffer() == one)
            .is_some());
        assert!(relations_on_zn
            .iter()
            .find(|relation| relation.matrix.buffer() == two)
            .is_some());
        assert!(relations_on_zn
            .iter()
            .find(|relation| relation.matrix.buffer() == top)
            .is_some());*/
    }

    #[test]
    #[ignore]
    fn z4_category_just_length() {
        use typenum::U4 as N;
        type R = Fin<N>;

        let zn_module: Arc<CanonModule<R>> = Arc::new(
            CoeffTree::<R, ()>::all_torsion_coeffs(2)
                .into_iter()
                .map(|torsion_coeffs| CanonModule::new(torsion_coeffs))
                .find(|zn_module| zn_module.cardinality() == 4 && zn_module.dimension() == 1)
                .unwrap(),
        );

        let direct = DirectModule::<R>::sumproduct(
            &Arc::clone(&zn_module),
            &Arc::new(zn_module.duplicate()),
        );

        let submodules = direct.submodules_goursat();
        let helper_data = HelperData::<R>::new(&direct);

        let relations_on_zn: Vec<Relation<R>> = submodules
            .into_iter()
            .map(|submodule| {
                Relation::<R>::from((direct.left(), direct.right(), submodule, &helper_data))
            })
            .collect();

        //assert_eq!(relations_on_zn.len(), 15);
    }

    #[test]
    #[ignore]
    fn z3_category_from_function() {
        use typenum::U3 as N;
        type R = Fin<N>;

        let category = Category::<CanonModule<R>, Relation<R>>::new(1);

        assert_eq!(category.hom_sets.len(), 1); // nie uwględnia modułu trywialnego, do poprawy w przyszłości

        let hom_sets_fixed_source = category
            .hom_sets
            .into_values()
            .find(|hom_set_fixed_source| {
                hom_set_fixed_source
                    .clone()
                    .into_values()
                    .find(|relations| {
                        relations
                            .iter()
                            .find(|relation| relation.source().cardinality() != 1)
                            .is_some()
                    })
                    .is_some()
            })
            .expect("there is a relation with non-trivial source");

        let relations_on_zn: Vec<Relation<R>> = hom_sets_fixed_source
            .into_values()
            .find(|relations| {
                relations
                    .iter()
                    .find(|relation| relation.target().cardinality() != 1)
                    .is_some()
            })
            .expect("there is a relation with non-trivial target");

        assert_eq!(relations_on_zn.len(), 6);

        let bottom = bitvec![1, 0, 0, 0, 0, 0, 0, 0, 0];
        let zero_dagger = bitvec![1, 1, 1, 0, 0, 0, 0, 0, 0];
        let zero = bitvec![1, 0, 0, 1, 0, 0, 1, 0, 0];
        let one = bitvec![1, 0, 0, 0, 1, 0, 0, 0, 1];
        let two = bitvec![1, 0, 0, 0, 0, 1, 0, 1, 0];
        let top = bitvec![1, 1, 1, 1, 1, 1, 1, 1, 1];

        /*
        assert!(relations_on_zn
            .iter()
            .find(|relation| relation.matrix_normal == bottom)
            .is_some());
        assert!(relations_on_zn
            .iter()
            .find(|relation| relation.matrix_normal == zero_dagger)
            .is_some());
        assert!(relations_on_zn
            .iter()
            .find(|relation| relation.matrix_normal == zero)
            .is_some());
        assert!(relations_on_zn
            .iter()
            .find(|relation| relation.matrix_normal == one)
            .is_some());
        assert!(relations_on_zn
            .iter()
            .find(|relation| relation.matrix_normal == two)
            .is_some());
        assert!(relations_on_zn
            .iter()
            .find(|relation| relation.matrix_normal == top)
            .is_some());
        */
    }
}
