#[allow(unused_imports)] // DELETE LATER
use crate::{
    category::{
        morphism::{Compose, EndoMorphism, Morphism},
        AllMorphisms, Category, Duplicate, HomSet,
    },
    rmodule::{
        canon::CanonModule, direct::DirectModule, map::CanonToCanon, ring::SuperRing,
        torsion::CoeffTree, Module,
    },
    util::category_of_relations::HelperData,
    Int,
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
        let (source, target, submodule, helper_data) = input;

        let mut matrix_normal = BitVec::with_capacity(helper_data.capacity as usize);
        let mut matrix_transposed = BitVec::with_capacity(helper_data.capacity as usize);

        unsafe {
            matrix_normal.set_len(helper_data.capacity as usize);
            matrix_transposed.set_len(helper_data.capacity as usize);
        }

        for element in submodule.image().into_iter() {
            let element: Vec<Int> = element
                .into_values()
                .map(|x| x.get())
                .zip(helper_data.torsion_coeffs_vec.iter())
                .map(|(x, y)| x % y)
                .collect();

            let index_normal = element
                .iter()
                .zip(helper_data.indices_normal.iter())
                .map(|(x, y)| x * y)
                .sum::<Int>();
            // unsafe {
            matrix_normal.set(index_normal as usize, true);
            //}

            let index_transposed = element
                .iter()
                .zip(helper_data.indices_transposed.iter())
                .map(|(x, y)| x * y)
                .sum::<Int>();
            //unsafe{
            matrix_transposed.set(index_transposed as usize, true);
            //
        }

        Relation::<R> {
            source,
            target,
            matrix_normal,
            matrix_transposed,
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
        category::{morphism::Morphism, relation::Relation, Category, Duplicate},
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

        let bottom = bitvec![1, 0, 0, 0, 0, 0, 0, 0, 0];
        let zero_dagger = bitvec![1, 1, 1, 0, 0, 0, 0, 0, 0];
        let zero = bitvec![1, 0, 0, 1, 0, 0, 1, 0, 0];
        let one = bitvec![1, 0, 0, 0, 1, 0, 0, 0, 1];
        let two = bitvec![1, 0, 0, 0, 0, 1, 0, 1, 0];
        let top = bitvec![1, 1, 1, 1, 1, 1, 1, 1, 1];

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
            .map(|submodule| {
                Relation::<R>::from((direct.left(), direct.right(), submodule, &helper_data))
            })
            .collect();

        assert_eq!(relations_on_zn.len(), 15);
    }

    #[test]
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
    }
}
