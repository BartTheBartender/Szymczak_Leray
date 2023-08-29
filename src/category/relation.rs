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

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
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

        println!("{}\n{}\n", zero, zero_dagger);

        assert_eq!(relations.len(), 0);
        assert_eq!(zero.source(), one.source());
        assert_eq!(one.source(), top.target());
        assert_eq!(zero.matrix_normal, zero_dagger.matrix_transposed);

        for relation in relations.iter() {
            println!("{}", relation);
        }
        println!("------\nafter multiplacation:");
        println!("{}", bottom.compose_unchecked(&top));
        println!("{}", zero_dagger);

        //36
        //assert_eq!(bottom.compose_unchecked(&bottom), bottom);
        //assert_eq!(bottom.compose_unchecked(&zero_dagger), zero_dagger);
        //assert_eq!(bottom.compose_unchecked(&zero), bottom);
        //assert_eq!(bottom.compose_unchecked(&one), bottom);
        //assert_eq!(bottom.compose_unchecked(&two), bottom);
        //assert_eq!(bottom.compose_unchecked(&three), bottom);
        //assert_eq!(bottom.compose_unchecked(&four), bottom);
        assert_eq!(bottom.compose_unchecked(&top), zero_dagger);
    }

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
