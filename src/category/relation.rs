#[allow(unused_imports)] // DELETE LATER
use crate::{
    category::{
        morphism::{Compose, EndoMorphism, Morphism},
        Category, HomSet,
    },
    rmodule::{
        canon::CanonModule, direct::DirectModule, map::CanonToCanon, ring::SuperRing,
        torsion::CoeffTree, Module,
    },
    util, Int,
};

use bitvec::prelude::*;
use rayon::prelude::*;
use std::{
    collections::{HashMap, HashSet},
    fmt::{self, Display},
    sync::Arc,
};

#[derive(Clone, Debug, Hash, Eq)]
pub struct Relation<R: SuperRing> {
    pub source: Arc<CanonModule<R>>,
    pub target: Arc<CanonModule<R>>,
    pub matrix_normal: BitVec,
    pub matrix_transposed: BitVec,
}

impl<R: SuperRing> Relation<R> {
    pub fn krakowian_product_unchecked(
        left: &BitVec,
        right: &BitVec,
        column_size: usize,
    ) -> BitVec {
        let left_columns = left.chunks(column_size);
        let right_columns = right.chunks(column_size);

        right_columns
            .flat_map(|right_column| {
                left_columns.clone().map(|left_column| {
                    let mut dot_prod = false;
                    for index in 0..column_size {
                        if unsafe {
                            *left_column.get_unchecked(index) && *right_column.get_unchecked(index)
                        } {
                            dot_prod = true;
                            break;
                        }
                    }
                    dot_prod
                })
            })
            .collect::<BitVec>()
    }
}

impl<R: SuperRing> PartialEq for Relation<R> {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source
            && self.target == other.target
            && self.matrix_normal == other.matrix_normal
            && self.matrix_transposed == other.matrix_transposed //to be removed in the future
    }
}

impl<R: SuperRing> Display for Relation<R> {
    //again, iterators...
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rows = self.matrix_transposed.chunks(self.source.cardinality());
        let mut output = String::new();

        for row in rows {
            for bit in row.iter() {
                if *bit {
                    output.push('1')
                } else {
                    output.push('0')
                }
            }
            output.push('\n');
        }

        write!(f, "{}", output)
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
        //consider switching from Rc to Arc and implementing it as below:
        /*
        rayon::join(
            || {
                Relation::krakowian_product_unchecked(
                    other.matrix_transpose,
                    self.matrix_normal,
                    self.target_size(),
                )
            },
            || todo!(),
        );
        */

        let column_size = self.target.cardinality();

        let output_normal = Relation::<R>::krakowian_product_unchecked(
            other.matrix_transposed.as_ref(),
            self.matrix_normal.as_ref(),
            column_size,
        );

        let output_transposed = Relation::<R>::krakowian_product_unchecked(
            self.matrix_normal.as_ref(),
            other.matrix_transposed.as_ref(),
            column_size,
        );

        Relation {
            source: Arc::clone(&self.source),
            target: Arc::clone(&other.target),
            matrix_normal: output_normal,
            matrix_transposed: output_transposed,
        }
    }
}

impl<R: SuperRing>
    TryFrom<(
        &DirectModule<R>,
        CanonToCanon<R>,
        &Vec<usize>,
        &Vec<usize>,
        &usize,
    )> for Relation<R>
{
    type Error = &'static str;
    /**
    the morphism should be mono in order for this conversion to work
    although the implementation neglects to check this

    the morphism should be a submodule of the given module
    */
    fn try_from(
        raw_data: (
            &DirectModule<R>,
            CanonToCanon<R>,
            &Vec<usize>,
            &Vec<usize>,
            &usize,
        ),
    ) -> Result<Self, Self::Error> {
        let (direct, submodule, helper_indices_normal, helper_indices_transposed, helper_capacity) =
            raw_data;

        let elements = submodule.image();

        let mut matrix_normal = BitVec::with_capacity(*helper_capacity);
        let mut matrix_transposed = BitVec::with_capacity(*helper_capacity);

        unsafe {
            matrix_normal.set_len(*helper_capacity);
            matrix_transposed.set_len(*helper_capacity);
        }

        for element in elements.iter() {
            //unsafe{
            let index_normal = element
                .coeffs()
                .map(|x| x.into())
                .zip(helper_indices_normal.iter())
                .map(|(x, y)| x * y)
                .sum::<usize>();

            matrix_normal.set(index_normal, true);

            let index_transposed = element
                .coeffs()
                .map(|x| x.into())
                .zip(helper_indices_transposed.iter())
                .map(|(x, y)| x * y)
                .sum::<usize>();

            matrix_transposed.set(index_transposed, true);
            //}
        }

        Ok(Relation::<R> {
            source: direct.left(),
            target: direct.right(),
            matrix_normal,
            matrix_transposed,
        })
    }
}

impl<R: SuperRing + std::hash::Hash> EndoMorphism<CanonModule<R>> for Relation<R> {}

impl<R: SuperRing> Category<CanonModule<R>, Relation<R>> {
    pub fn new(maximal_dimension: Int) -> Self {
        let all_canon_rmodules: HashSet<Arc<CanonModule<R>>> =
            CoeffTree::<R, ()>::all_torsion_coeffs(maximal_dimension)
                .into_iter()
                .map(CanonModule::<R>::new)
                .map(Arc::new)
                .collect();

        let hom_sets = all_canon_rmodules
            .iter()
            .map(|source| {
                (
                    source.as_ref().clone(),
                    all_canon_rmodules
                        .iter()
                        .map(|target| {
                            (
                                target.as_ref().clone(),
                                Self::hom_set(Arc::clone(&source), Arc::clone(&target)),
                            )
                        })
                        .collect::<HashMap<CanonModule<R>, Vec<Relation<R>>>>(),
                )
            })
            .collect::<HomSet<CanonModule<R>, Relation<R>>>();

        Category { hom_sets }
    }

    fn hom_set(source: Arc<CanonModule<R>>, target: Arc<CanonModule<R>>) -> Vec<Relation<R>> {
        let direct = DirectModule::<R>::sumproduct(&source, &target);
        let (helper_indices_normal, helper_indices_transposed, helper_length) =
            unsafe { util::category_of_relations::calculate_helper_indices_and_capacity(&direct) };
        direct
            .submodules_goursat()
            .filter_map(|submodule| {
                Relation::<R>::try_from((
                    &direct,
                    submodule,
                    &helper_indices_normal,
                    &helper_indices_transposed,
                    &helper_length,
                ))
                .ok()
            })
            .collect::<Vec<Relation<R>>>()
    }
}

#[cfg(test)]
mod test {
    use crate::{
        category::{relation::Relation, Category},
        error::Error,
        rmodule::{
            canon::CanonModule,
            direct::DirectModule,
            map::CanonToCanon,
            ring::{Fin, Ring, Set},
            torsion::CoeffTree,
        },
        util,
    };
    use bitvec::prelude::*;
    use std::sync::Arc;

    /*
        #[test]
        fn krakowian_product() {
            let v = bitvec![1, 0, 0, 0];
            let u = bitvec![1, 0, 0, 1];

            let w = Relation::krakowian_product_unchecked(&v, &u, 2);

            assert_eq!(w, v);
        }

        #[test]
        fn relation_product_1() {
            let v = bitvec![1, 0, 0, 0];
            let canon_zmodule = Arc::new(CanonZModule::new_unchecked(vec![2]));
    :x
    :z:x
            let r = Relation {
                source: Arc::clone(&canon_zmodule),
                target: Arc::clone(&canon_zmodule),
                matrix_normal: v.clone(),
                matrix_transposed: v.clone(),
            };

            let s = r.compose_unchecked(&r);

            assert_eq!(r, s);
        }

        #[test]
        fn relation_product_2() {
            let v = bitvec![1, 1, 1, 1];
            let canon_zmodule = Arc::new(CanonZModule::new_unchecked(vec![2]));

            let r = Relation {
                source: Arc::clone(&canon_zmodule),
                target: Arc::clone(&canon_zmodule),
                matrix_normal: v.clone(),
                matrix_transposed: v.clone(),
            };

            let s = r.compose_unchecked(&r);

            assert_eq!(r, s);
        }

        #[test]
        fn relation_product_3() {
            let v = bitvec![1, 0, 0, 1];
            let canon_zmodule = Arc::new(CanonZModule::new_unchecked(vec![2]));

            let r = Relation {
                source: Arc::clone(&canon_zmodule),
                target: Arc::clone(&canon_zmodule),
                matrix_normal: v.clone(),
                matrix_transposed: v.clone(),
            };

            let s = r.compose_unchecked(&r);

            assert_eq!(r, s);
        }

        #[test]
        fn relation_product_error_1() {
            let v = bitvec![1, 0, 0, 1];
            let u = bitvec![1, 0, 0, 0, 0, 0, 1, 1, 1];
            let canon_zmodule_v = Arc::new(CanonZModule::new_unchecked(vec![2]));
            let canon_zmodule_u = Arc::new(CanonZModule::new_unchecked(vec![3]));

            let r = Relation {
                source: Arc::clone(&canon_zmodule_v),
                target: Arc::clone(&canon_zmodule_v),
                matrix_normal: v.clone(),
                matrix_transposed: v.clone(),
            };
            let s = Relation {
                source: Arc::clone(&canon_zmodule_u),
                target: Arc::clone(&canon_zmodule_u),
                matrix_normal: u.clone(),
                matrix_transposed: u.clone(),
            };

            let error = s.compose(&r);

            assert_eq!(error, Err(Error::SourceTargetMismatch));
        }
    */
    #[test]
    fn relation_from_direct() {
        use typenum::U2 as N;
        type R = Fin<N>;

        let mut tc = CoeffTree::<R, ()>::all_torsion_coeffs(3);

        let torsion_coeffs_zn = tc.next().unwrap();

        //assert_eq!(torsion_coeffs_zn.len(), 1);

        let zn_module_arc = Arc::new(CanonModule::<R>::new(torsion_coeffs_zn));
        //assert_eq!(zn_module_arc.cardinality(), 2);

        let direct = DirectModule::<R>::sumproduct(&zn_module_arc, &zn_module_arc);
        let submodules: Vec<CanonToCanon<R>> = direct.submodules_goursat().collect();
        let direct = DirectModule::<R>::sumproduct(&zn_module_arc, &zn_module_arc);
        let (helper_indices_normal, helper_indices_transposed, helper_capacity) =
            util::category_of_relations::calculate_helper_indices_and_capacity(&direct);

        let submodules_elements: Vec<_> = submodules
            .into_iter()
            .map(|submodule| submodule.image())
            .collect();

        assert_eq!(submodules_elements.len(), 5);
        assert_eq!(helper_capacity, 4);

        for submodule_elements in submodules_elements {
            let mut matrix_normal = BitVec::<usize, Lsb0>::with_capacity(helper_capacity);
            let mut matrix_transposed = BitVec::<usize, Lsb0>::with_capacity(helper_capacity);

            unsafe {
                matrix_normal.set_len(helper_capacity);
                matrix_transposed.set_len(helper_capacity);
            }

            assert_eq!(matrix_normal.len(), helper_capacity);
            assert_eq!(matrix_transposed.len(), helper_capacity);

            for element in submodule_elements {
                let index_normal = element
                    .coeffs()
                    .map(|x| x.get() as usize)
                    .zip(helper_indices_normal.iter())
                    .map(|(x, y)| x * y)
                    .sum::<usize>();
                assert!(index_normal < helper_capacity);
                matrix_normal.set(index_normal, true);

                let index_transposed = element
                    .coeffs()
                    .map(|x| x.get() as usize)
                    .zip(helper_indices_transposed.iter())
                    .map(|(x, y)| x * y)
                    .sum::<usize>();
                assert!(index_transposed < helper_capacity);
                matrix_transposed.set(index_transposed, true);
            }
        }
    }

    #[test]
    fn zn_category() {
        use typenum::U2 as N;
        type R = Fin<N>;

        let category = Category::<CanonModule<R>, Relation<R>>::new(2);

        assert_eq!(
            category
                .hom_sets
                .values()
                .map(|hom_set_fixed_source| hom_set_fixed_source.values().count())
                .sum::<usize>(),
            5
        );
    }
}
