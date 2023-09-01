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
        ring::{Fin, Set, SuperRing},
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
    fmt::{self, Debug, Display},
    iter,
    sync::Arc,
};
use typenum::Unsigned;

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Relation<R: SuperRing> {
    pub source: Arc<CanonModule<R>>,
    pub target: Arc<CanonModule<R>>,
    pub matrix: Matrix<bool>,
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
impl<R: SuperRing> Display for Relation<R> {
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
            matrix: self.matrix.compose_unchecked_bool(&other.matrix),
        }
    }
}

impl<R: SuperRing> From<(&DirectModule<R>, CanonToCanon<R>)> for Relation<R> {
    /**
    the morphism should be mono in order for this conversion to work
    although the implementation neglects to check this

    the morphism should be a submodule of the given module
    */
    fn from(input: (&DirectModule<R>, CanonToCanon<R>)) -> Self {
        let (direct, submodule) = input;
        let n: Int = <R as Set>::Card::to_usize() as Int;

        let mut prod = 1;
        let mut prod_ret = 1;
        let source_index_shift: Vec<Int> = direct
            .left()
            .torsion_coeffs()
            .map(|x| {
                prod_ret = prod;
                prod *= x.get();
                prod_ret
            })
            .collect();
        let cols = prod;

        let source_tc: Vec<Int> = direct
            .left()
            .torsion_coeffs()
            .into_iter()
            .map(|tc| tc.get())
            .collect();

        let mut prod = 1;
        let mut prod_ret = 1;
        let target_index_shift: Vec<Int> = direct
            .right()
            .torsion_coeffs()
            .map(|x| {
                prod_ret = prod;
                prod *= x.get();
                prod_ret
            })
            .collect();
        let rows = prod;

        let target_tc: Vec<Int> = direct
            .right()
            .torsion_coeffs()
            .into_iter()
            .map(|tc| tc.get())
            .collect();

        let mut buffer = vec![false; (rows * cols) as usize];

        for element in submodule.image().into_iter() {
            let source_element: Vec<Int> = direct
                .left_projection
                .evaluate_unchecked(&element)
                .into_values()
                .map(|x| x.get() % n)
                .zip(source_tc.iter())
                .map(|(x, tc)| if *tc != 1 { x % tc } else { x })
                .collect();

            let source_index: Int = source_element
                .iter()
                .zip(source_index_shift.iter())
                .map(|(el, sh)| el * sh)
                .sum::<Int>();

            let target_element: Vec<Int> = direct
                .right_projection
                .evaluate_unchecked(&element)
                .into_values()
                .map(|x| x.get() % n)
                .zip(target_tc.iter())
                .map(|(x, tc)| if *tc != 1 { x % tc } else { x })
                .collect();

            let target_index: Int = target_element
                .iter()
                .zip(target_index_shift.iter())
                .map(|(el, sh)| el * sh)
                .sum::<Int>();

            let index = usize::from(source_index + cols * target_index);

            buffer[index] = true;
        }

        Relation {
            source: direct.left(),
            target: direct.right(),
            matrix: Matrix::from_buffer(buffer, cols as u8, rows as u8),
        }
    }
}

impl<R: SuperRing + std::hash::Hash> EndoMorphism<CanonModule<R>> for Relation<R> {}

impl<R: SuperRing> AllMorphisms<CanonModule<R>> for Relation<R> {
    fn hom_set(source: Arc<CanonModule<R>>, target: Arc<CanonModule<R>>) -> Vec<Relation<R>> {
        let direct = DirectModule::<R>::sumproduct(&source, &target);

        direct
            .submodules_goursat()
            .map(|submodule| Relation::<R>::from((&direct, submodule)))
            .collect::<Vec<Relation<R>>>()
    }
}

#[cfg(test)]
mod test {
    use super::*;
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
            Module,
        },
        util::category_of_relations::HelperData,
        Int,
    };
    use bitvec::prelude::*;
    use std::{
        collections::{HashMap, HashSet},
        sync::Arc,
    };

    #[test]
    fn relation_composition_z5() {
        use typenum::U5 as N;
        type R = Fin<N>;
        let category = Category::<CanonModule<R>, Relation<R>>::new(1);

        let relations: Vec<Relation<R>> = category
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

        let bottom_ok_raw = vec![
            1, 0, 0, 0, 0, /**/ 0, 0, 0, 0, 0, /**/ 0, 0, 0, 0, 0, /**/ 0, 0, 0, 0,
            0, /**/ 0, 0, 0, 0, 0,
        ];
        let zero_ok_raw = vec![
            1, 1, 1, 1, 1, /**/ 0, 0, 0, 0, 0, /**/ 0, 0, 0, 0, 0, /**/ 0, 0, 0, 0,
            0, /**/ 0, 0, 0, 0, 0,
        ];
        let zero_dagger_ok_raw = vec![
            1, 0, 0, 0, 0, /**/ 1, 0, 0, 0, 0, /**/ 1, 0, 0, 0, 0, /**/ 1, 0, 0, 0,
            0, /**/ 1, 0, 0, 0, 0,
        ];
        let one_ok_raw = vec![
            1, 0, 0, 0, 0, /**/ 0, 1, 0, 0, 0, /**/ 0, 0, 1, 0, 0, /**/ 0, 0, 0, 1,
            0, /**/ 0, 0, 0, 0, 1,
        ];
        let two_ok_raw = vec![
            1, 0, 0, 0, 0, /**/ 0, 0, 1, 0, 0, /**/ 0, 0, 0, 0, 1, /**/ 0, 1, 0, 0,
            0, /**/ 0, 0, 0, 1, 0,
        ];
        let three_ok_raw = vec![
            1, 0, 0, 0, 0, /**/ 0, 0, 0, 1, 0, /**/ 0, 1, 0, 0, 0, /**/ 0, 0, 0, 0,
            1, /**/ 0, 0, 1, 0, 0,
        ];
        let four_ok_raw = vec![
            1, 0, 0, 0, 0, /**/ 0, 0, 0, 0, 1, /**/ 0, 0, 0, 1, 0, /**/ 0, 0, 1, 0,
            0, /**/ 0, 1, 0, 0, 0,
        ];
        let top_ok_raw = vec![
            1, 1, 1, 1, 1, /**/ 1, 1, 1, 1, 1, /**/ 1, 1, 1, 1, 1, /**/ 1, 1, 1, 1,
            1, /**/ 1, 1, 1, 1, 1,
        ];

        let bottom_ok: Vec<bool> = bottom_ok_raw.into_iter().map(|entry| entry == 1).collect();
        let zero_ok: Vec<bool> = zero_ok_raw.into_iter().map(|entry| entry == 1).collect();
        let zero_dagger_ok: Vec<bool> = zero_dagger_ok_raw
            .into_iter()
            .map(|entry| entry == 1)
            .collect();
        let one_ok: Vec<bool> = one_ok_raw.into_iter().map(|entry| entry == 1).collect();
        let two_ok: Vec<bool> = two_ok_raw.into_iter().map(|entry| entry == 1).collect();
        let three_ok: Vec<bool> = three_ok_raw.into_iter().map(|entry| entry == 1).collect();
        let four_ok: Vec<bool> = four_ok_raw.into_iter().map(|entry| entry == 1).collect();
        let top_ok: Vec<bool> = top_ok_raw.into_iter().map(|entry| entry == 1).collect();

        let bottom: Relation<R> = relations
            .iter()
            .find(|relation| relation.matrix.buffer() == bottom_ok)
            .expect("there are exactly eight relations")
            .clone();
        let zero: Relation<R> = relations
            .iter()
            .find(|relation| relation.matrix.buffer() == zero_ok)
            .expect("there are exactly eight relations")
            .clone();
        let zero_dagger: Relation<R> = relations
            .iter()
            .find(|relation| relation.matrix.buffer() == zero_dagger_ok)
            .expect("there are exactly eight relations")
            .clone();
        let one: Relation<R> = relations
            .iter()
            .find(|relation| relation.matrix.buffer() == one_ok)
            .expect("there are exactly eight relations")
            .clone();
        let two: Relation<R> = relations
            .iter()
            .find(|relation| relation.matrix.buffer() == two_ok)
            .expect("there are exactly eight relations")
            .clone();
        let three: Relation<R> = relations
            .iter()
            .find(|relation| relation.matrix.buffer() == three_ok)
            .expect("there are exactly eight relations")
            .clone();
        let four: Relation<R> = relations
            .iter()
            .find(|relation| relation.matrix.buffer() == four_ok)
            .expect("there are exactly eight relations")
            .clone();
        let top: Relation<R> = relations
            .iter()
            .find(|relation| relation.matrix.buffer() == top_ok)
            .expect("there are exactly eight relations")
            .clone();

        //36 = 8 + 7 + 6 + 5 + 4 + 3 + 2 + 1

        //8
        assert_eq!(bottom.compose_unchecked(&bottom), bottom);
        //
        assert_eq!(bottom.compose_unchecked(&zero_dagger), zero_dagger);
        //
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
    fn category_step_by_step() {
        use crate::util::matrix::Matrix;
        use typenum::{Unsigned, U3 as N};
        let n: Int = N::to_usize() as Int;
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

        let relations_zn_out: Vec<Relation<R>> = submodules
            .into_iter()
            .map(|submodule| Relation::<R>::from((&direct, submodule)))
            .collect();

        assert_eq!(relations_zn_out.len(), 6);

        let matrices_zn_out: Vec<Matrix<bool>> = relations_zn_out
            .into_iter()
            .map(|relation| relation.matrix)
            .collect();

        let bottom: Vec<Int> = vec![1, 0, 0, 0, 0, 0, 0, 0, 0];
        let zero_dagger: Vec<Int> = vec![1, 0, 0, 1, 0, 0, 1, 0, 0];
        let zero: Vec<Int> = vec![1, 1, 1, 0, 0, 0, 0, 0, 0];
        let one: Vec<Int> = vec![1, 0, 0, 0, 1, 0, 0, 0, 1];
        let two: Vec<Int> = vec![1, 0, 0, 0, 0, 1, 0, 1, 0];
        let top: Vec<Int> = vec![1, 1, 1, 1, 1, 1, 1, 1, 1];

        let matrices_zn_raw = vec![bottom, zero, zero_dagger, one, two, top];

        let matrices_zn_ok = matrices_zn_raw
            .into_iter()
            .map(|raw_buffer| {
                raw_buffer
                    .into_iter()
                    .map(|bool| bool == 1)
                    .collect::<Vec<bool>>()
            })
            .map(|buffer| Matrix::from_buffer(buffer, 3, 3))
            .collect::<Vec<Matrix<bool>>>();

        for matrix_ok in matrices_zn_ok.iter() {
            assert!(matrices_zn_out
                .iter()
                .find(|matrix_out| *matrix_out == matrix_ok)
                .is_some());
        }
        for matrix_out in matrices_zn_out.iter() {
            assert!(matrices_zn_ok
                .iter()
                .find(|matrix_ok| *matrix_ok == matrix_out)
                .is_some());
        }
    }

    #[test]
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
            .map(|submodule| Relation::<R>::from((&direct, submodule)))
            .collect();

        assert_eq!(relations_on_zn.len(), 15);
    }

    #[test]
    fn z3_category_from_function() {
        use crate::util::matrix::Matrix;
        use typenum::U3 as N;
        type R = Fin<N>;

        let category = Category::<CanonModule<R>, Relation<R>>::new(1);

        assert_eq!(category.hom_sets.len(), 2);

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

        let relations_zn_out: Vec<Relation<R>> = hom_sets_fixed_source
            .into_values()
            .find(|relations| {
                relations
                    .iter()
                    .find(|relation| relation.target().cardinality() != 1)
                    .is_some()
            })
            .expect("there is a relation with non-trivial target");

        assert_eq!(relations_zn_out.len(), 6);

        let matrices_zn_out: Vec<Matrix<bool>> = relations_zn_out
            .into_iter()
            .map(|relation| relation.matrix)
            .collect();

        let bottom: Vec<Int> = vec![1, 0, 0, 0, 0, 0, 0, 0, 0];
        let zero_dagger: Vec<Int> = vec![1, 0, 0, 1, 0, 0, 1, 0, 0];
        let zero: Vec<Int> = vec![1, 1, 1, 0, 0, 0, 0, 0, 0];
        let one: Vec<Int> = vec![1, 0, 0, 0, 1, 0, 0, 0, 1];
        let two: Vec<Int> = vec![1, 0, 0, 0, 0, 1, 0, 1, 0];
        let top: Vec<Int> = vec![1, 1, 1, 1, 1, 1, 1, 1, 1];

        let matrices_zn_raw = vec![bottom, zero, zero_dagger, one, two, top];

        let matrices_zn_ok = matrices_zn_raw
            .into_iter()
            .map(|raw_buffer| {
                raw_buffer
                    .into_iter()
                    .map(|bool| bool == 1)
                    .collect::<Vec<bool>>()
            })
            .map(|buffer| Matrix::from_buffer(buffer, 3, 3))
            .collect::<Vec<Matrix<bool>>>();

        for matrix_ok in matrices_zn_ok.iter() {
            assert!(matrices_zn_out
                .iter()
                .find(|matrix_out| *matrix_out == matrix_ok)
                .is_some());
        }
        for matrix_out in matrices_zn_out.iter() {
            assert!(matrices_zn_ok
                .iter()
                .find(|matrix_ok| *matrix_ok == matrix_out)
                .is_some());
        }
    }

    #[test]
    fn no_duplicates() {
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

        let mut hom_set_zn_zn_no_dupes = hom_set_zn_zn.clone();
        hom_set_zn_zn_no_dupes.dedup();

        assert_eq!(hom_set_zn_zn, hom_set_zn_zn_no_dupes);
    }

    #[test]
    fn z1_to_z2_relations() {
        use crate::{
            category::{AllMorphisms, AllObjects},
            util::matrix::Matrix,
        };
        use typenum::{Unsigned, U2 as N};
        let n: Int = N::to_usize() as Int;
        type R = Fin<N>;

        let zn_modules = CanonModule::<R>::all_objects(1);

        let z1 = Arc::new(
            zn_modules
                .iter()
                .find(|module| module.to_string() == "Z1")
                .unwrap()
                .clone(),
        );
        let z2 = Arc::new(
            zn_modules
                .iter()
                .find(|module| module.to_string() == "Z2")
                .unwrap()
                .clone(),
        );

        let direct = DirectModule::<R>::sumproduct(&z1, &z2);

        assert_eq!(direct.submodules_goursat().count(), 2);

        for submodule in direct.submodules_goursat() {
            let mut prod = 1;
            let mut prod_ret = 1;
            let source_index_shift: Vec<Int> = direct
                .left()
                .torsion_coeffs()
                .map(|x| {
                    prod_ret = prod;
                    prod *= x.get();
                    prod_ret
                })
                .collect();
            let cols = prod;

            let source_tc: Vec<Int> = direct
                .left()
                .torsion_coeffs()
                .into_iter()
                .map(|tc| tc.get())
                .collect();

            let mut prod = 1;
            let mut prod_ret = 1;
            let target_index_shift: Vec<Int> = direct
                .right()
                .torsion_coeffs()
                .map(|x| {
                    prod_ret = prod;
                    prod *= x.get();
                    prod_ret
                })
                .collect();
            let rows = prod;

            let target_tc: Vec<Int> = direct
                .right()
                .torsion_coeffs()
                .into_iter()
                .map(|tc| tc.get())
                .collect();

            let mut buffer = vec![false; (rows * cols) as usize];

            for element in submodule.image().into_iter() {
                let source_element: Vec<Int> = direct
                    .left_projection
                    .evaluate_unchecked(&element)
                    .into_values()
                    .map(|x| x.get() % n)
                    .zip(source_tc.iter())
                    .map(|(x, tc)| if *tc != 1 { x % tc } else { x })
                    .collect();

                let source_index: Int = source_element
                    .iter()
                    .zip(source_index_shift.iter())
                    .map(|(el, sh)| el * sh)
                    .sum::<Int>();

                let target_element: Vec<Int> = direct
                    .right_projection
                    .evaluate_unchecked(&element)
                    .into_values()
                    .map(|x| x.get() % n)
                    .zip(target_tc.iter())
                    .map(|(x, tc)| if *tc != 1 { x % tc } else { x })
                    .collect();

                let target_index: Int = target_element
                    .iter()
                    .zip(target_index_shift.iter())
                    .map(|(el, sh)| el * sh)
                    .sum::<Int>();

                let index = usize::from(source_index + cols * target_index);

                buffer[index] = true;
            }
        }
    }
}
