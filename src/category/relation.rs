pub use crate::{
    category::{
        morphism::{
            Concrete as ConcreteMorphism, Endo as EndoMorphism, Enumerable as EnumerableMorphism,
            IsBij, IsMap, IsMatching, IsWide, Morphism,
        },
        object::Concrete as ConcreteObject,
        PrettyName,
    },
    ralg::{
        cgroup::{ideal::CIdeal, Radix, C},
        matrix::Matrix,
        module::{
            canon::object::Object as CanonModule, direct::Object as DirectModule, map::CanonToCanon,
        },
        ring::{
            ideal::{Ideal, Principal as PrincipalIdeal},
            Ring,
        },
    },
};

use std::{fmt, hash, sync::Arc};
use typenum::{IsGreater, U1};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Relation<R: Ring, I: Ideal<Parent = R> + Ord> {
    pub source: Arc<CanonModule<R, I>>,
    pub target: Arc<CanonModule<R, I>>,
    pub matrix: Matrix<bool>,
}

impl<R: Ring + fmt::Debug, I: Ideal<Parent = R> + Ord + fmt::Debug> fmt::Debug for Relation<R, I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut matrix_out = String::new();

        for ind_col in 0..self.matrix.nof_cols {
            for ind_row in 0..self.matrix.nof_rows {
                match self
                    .matrix
                    .get(ind_col, ind_row)
                    .expect("the indices are in proper bounds")
                {
                    true => matrix_out.push('1'),
                    false => matrix_out.push('0'),
                }
            }
            //  matrix_out.push('\n');
        }
        /*
        write!(
            f,
            "s:{:?}, t:{:?}, Mtx({}x{}):{}",
            self.source, self.target, self.matrix.nof_rows, self.matrix.nof_cols, matrix_out
        )
        */
        write!(f, "{matrix_out}")
    }
}
impl<R: Ring + fmt::Display, I: Ideal<Parent = R> + Ord + fmt::Display> fmt::Display
    for Relation<R, I>
where
    CanonModule<R, I>: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut matrix_out = String::new();

        for ind_col in 0..self.matrix.nof_cols {
            for ind_row in 0..self.matrix.nof_rows {
                match self
                    .matrix
                    .get(ind_col, ind_row)
                    .expect("the indices are in proper bounds")
                {
                    true => matrix_out.push('1'),
                    false => matrix_out.push('0'),
                }
            }
        }
        write!(
            f,
            "source: {}, target: {}, matrix:\n{matrix_out}",
            self.source(),
            self.target()
        )
    }
}

impl<R: Ring, I: Ideal<Parent = R> + Ord> PrettyName for Relation<R, I> {
    const PRETTY_NAME: &'static str = "Relation";
}

//important code  - generation of the category of R-modules and relations
impl<R: Ring, I: Ideal<Parent = R> + Ord> Morphism<CanonModule<R, I>> for Relation<R, I> {
    type B = Arc<CanonModule<R, I>>;

    fn source(&self) -> Self::B {
        Arc::clone(&self.source)
    }

    fn target(&self) -> Self::B {
        Arc::clone(&self.target)
    }

    // other * self
    fn compose(&self, other: &Self) -> Self {
        Self {
            source: Arc::clone(&self.source),
            target: Arc::clone(&other.target),
            matrix: self.matrix.compose_unchecked_bool(&other.matrix),
        }
    }
}

impl<R: Ring + Copy + hash::Hash, I: Ideal<Parent = R> + Ord + hash::Hash>
    EndoMorphism<CanonModule<R, I>> for Relation<R, I>
{
    fn identity(object: Arc<CanonModule<R, I>>) -> Self {
        let card = object.cardinality();

        let buffer = (0..card).flat_map(move |i| (0..card).map(move |j| j == i));

        Self {
            source: Arc::clone(&object),
            target: Arc::clone(&object),
            matrix: Matrix::<bool>::from_buffer(buffer, card, card),
        }
    }
}

impl<R: Ring + Copy + Into<u16>, I: PrincipalIdeal<Parent = R> + Ord>
    From<(&DirectModule<R, I>, CanonToCanon<R, I>)> for Relation<R, I>
{
    /**
    the morphism should be mono in order for this conversion to work
    although the implementation neglects to check this

    the morphism should be a submodule of the given module
    */
    #[allow(
        clippy::arithmetic_side_effects,
        reason = "fuck this, i am not refactoring that"
    )]
    #[allow(clippy::expect_used, reason = "fuck this, i am not refactoring that")]
    fn from(input: (&DirectModule<R, I>, CanonToCanon<R, I>)) -> Self {
        let (direct, submodule) = input;
        let n: u16 = R::cardinality().try_into().expect("bigger int needed");

        let mut cols: u16 = 1;
        let mut cols_ret: u16 = 1;
        let source_index_shift: Vec<u16> = direct
            .left()
            .torsion_coeffs_as_u16()
            .map(|x| {
                cols_ret = cols;
                cols *= x;
                cols_ret
            })
            .collect();

        let source_tc: Vec<u16> = direct.left().torsion_coeffs_as_u16().collect();

        let mut rows: u16 = 1;
        let mut rows_ret: u16 = 1;
        let target_index_shift: Vec<u16> = direct
            .right()
            .torsion_coeffs_as_u16()
            .map(|x| {
                rows_ret = rows;
                rows *= x;
                rows_ret
            })
            .collect();

        let target_tc: Vec<u16> = direct.right().torsion_coeffs_as_u16().collect();

        let mut buffer = vec![false; (rows * cols).into()];

        for element in submodule.image() {
            let source_element: Vec<u16> = direct
                .left_projection
                .try_evaluate(element.clone())
                .expect("element from image")
                .into_values()
                .map(|x| x.into() % n)
                .zip(source_tc.iter())
                .map(|(x, tc)| if *tc == 1 { x } else { x % tc })
                .collect();

            let source_index: u16 = source_element
                .iter()
                .zip(source_index_shift.iter())
                .map(|(el, sh)| el * sh)
                .sum();

            let target_element: Vec<u16> = direct
                .right_projection
                .try_evaluate(element)
                .expect("element from image")
                .into_values()
                .map(|x| x.into() % n)
                .zip(target_tc.iter())
                .map(|(x, tc)| if *tc == 1 { x } else { x % tc })
                .collect();

            let target_index: u16 = target_element
                .iter()
                .zip(target_index_shift.iter())
                .map(|(el, sh)| el * sh)
                .sum();

            let index = usize::from(source_index + cols * target_index);

            *buffer
                .get_mut(index)
                .expect("index calculated to be within range") = true;
        }

        Self {
            source: direct.left(),
            target: direct.right(),
            matrix: Matrix::from_buffer(buffer, cols.into(), rows.into()),
        }
    }
}

impl<Period: Radix + IsGreater<U1> + Send + Sync>
    EnumerableMorphism<CanonModule<C<Period>, CIdeal<Period>>>
    for Relation<C<Period>, CIdeal<Period>>
{
    fn hom(source: Self::B, target: Self::B) -> impl Iterator<Item = Self> + Clone {
        let direct = DirectModule::sumproduct(&source, &target);

        direct
            .clone()
            .submodules_goursat()
            .into_iter()
            .map(move |submodule| Self::from((&direct, submodule)))
    }
}

impl<R: Ring, I: Ideal<Parent = R> + Ord> IsMap<CanonModule<R, I>> for Relation<R, I> {
    fn is_a_map(&self) -> bool {
        self.matrix
            .cols()
            .all(|collumn| collumn.filter(|entry| **entry).count() == 1)
    }
}

impl<R: Ring, I: Ideal<Parent = R> + Ord> IsMatching<CanonModule<R, I>> for Relation<R, I> {
    fn is_a_matching(&self) -> bool {
        self.matrix
            .cols()
            .all(|col| col.filter(|entry| **entry).count() <= 1)
            && self
                .matrix
                .rows()
                .all(|row| row.filter(|entry| **entry).count() <= 1)
    }
}

impl<R: Ring, I: Ideal<Parent = R> + Ord> IsWide<CanonModule<R, I>> for Relation<R, I> {
    fn is_wide(&self) -> bool {
        self.matrix
            .cols()
            .all(|col| col.filter(|entry| **entry).count() > 0)
            && self
                .matrix
                .rows()
                .all(|row| row.filter(|entry| **entry).count() > 0)
    }
}

impl<R: Ring, I: Ideal<Parent = R> + Ord> IsBij<CanonModule<R, I>> for Relation<R, I> {
    fn is_a_bijection(&self) -> bool {
        self.matrix
            .cols()
            .all(|col| col.filter(|entry| **entry).count() == 1)
            && self
                .matrix
                .rows()
                .all(|row| row.filter(|entry| **entry).count() == 1)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        category::{
            object::{Concrete, PartiallyEnumerable},
            Category,
        },
        util::category_of_relations::HelperData,
        Int,
    };
    // use bitvec::prelude::*;
    use std::sync::Arc;

    #[test]
    #[allow(clippy::default_numeric_fallback, reason = "i ain't refactoring this")]
    fn relation_composition_z5() {
        use typenum::{Unsigned, U5 as N};
        type R = C<N>;
        type I = CIdeal<N>;
        let category = Category::<CanonModule<R, I>, Relation<R, I>>::new(1);

        println!("{:?}", category);

        for object in category.clone().into_objects() {
            println!("{:?}", object);
        }

        let zn = category
            .clone()
            .into_objects()
            .into_iter()
            .find(|module| module.cardinality() == N::to_usize())
            .expect("there is a zn module");

        let hom_set_zn_zn = category.hom_set(&zn, &zn);

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

        let bottom: Relation<R, I> = hom_set_zn_zn
            .iter()
            .find(|relation| relation.matrix.buffer() == bottom_ok)
            .expect("there are exactly eight relations")
            .clone();
        let zero: Relation<R, I> = hom_set_zn_zn
            .iter()
            .find(|relation| relation.matrix.buffer() == zero_ok)
            .expect("there are exactly eight relations")
            .clone();
        let zero_dagger: Relation<R, I> = hom_set_zn_zn
            .iter()
            .find(|relation| relation.matrix.buffer() == zero_dagger_ok)
            .expect("there are exactly eight relations")
            .clone();
        let one: Relation<R, I> = hom_set_zn_zn
            .iter()
            .find(|relation| relation.matrix.buffer() == one_ok)
            .expect("there are exactly eight relations")
            .clone();
        let two: Relation<R, I> = hom_set_zn_zn
            .iter()
            .find(|relation| relation.matrix.buffer() == two_ok)
            .expect("there are exactly eight relations")
            .clone();
        let three: Relation<R, I> = hom_set_zn_zn
            .iter()
            .find(|relation| relation.matrix.buffer() == three_ok)
            .expect("there are exactly eight relations")
            .clone();
        let four: Relation<R, I> = hom_set_zn_zn
            .iter()
            .find(|relation| relation.matrix.buffer() == four_ok)
            .expect("there are exactly eight relations")
            .clone();
        let top: Relation<R, I> = hom_set_zn_zn
            .iter()
            .find(|relation| relation.matrix.buffer() == top_ok)
            .expect("there are exactly eight relations")
            .clone();

        //36 = 8 + 7 + 6 + 5 + 4 + 3 + 2 + 1

        //8
        assert_eq!(bottom.compose(&bottom), bottom);
        assert_eq!(bottom.compose(&zero_dagger), zero_dagger);
        assert_eq!(bottom.compose(&zero), bottom);
        assert_eq!(bottom.compose(&one), bottom);
        assert_eq!(bottom.compose(&two), bottom);
        assert_eq!(bottom.compose(&three), bottom);
        assert_eq!(bottom.compose(&four), bottom);
        assert_eq!(bottom.compose(&top), zero_dagger);

        //7
        assert_eq!(zero_dagger.compose(&zero_dagger), zero_dagger);
        assert_eq!(zero_dagger.compose(&zero), bottom);
        assert_eq!(zero_dagger.compose(&one), zero_dagger);
        assert_eq!(zero_dagger.compose(&two), zero_dagger);
        assert_eq!(zero_dagger.compose(&three), zero_dagger);
        assert_eq!(zero_dagger.compose(&four), zero_dagger);
        assert_eq!(zero_dagger.compose(&top), zero_dagger);

        //6
        assert_eq!(zero.compose(&zero), zero);
        assert_eq!(zero.compose(&one), zero);
        assert_eq!(zero.compose(&two), zero);
        assert_eq!(zero.compose(&three), zero);
        assert_eq!(zero.compose(&four), zero);
        assert_eq!(zero.compose(&top), top);

        //5
        assert_eq!(one.compose(&one), one);
        assert_eq!(one.compose(&two), two);
        assert_eq!(one.compose(&three), three);
        assert_eq!(one.compose(&four), four);
        assert_eq!(one.compose(&top), top);

        //4
        assert_eq!(two.compose(&two), four);
        assert_eq!(two.compose(&three), one);
        assert_eq!(two.compose(&four), three);
        assert_eq!(two.compose(&top), top);

        //3
        assert_eq!(three.compose(&three), four);
        assert_eq!(three.compose(&four), two);
        assert_eq!(three.compose(&top), top);

        //2
        assert_eq!(four.compose(&four), one);
        assert_eq!(four.compose(&top), top);

        //1
        assert_eq!(top.compose(&top), top);
    }

    #[test]
    fn category_step_by_step() {
        use typenum::{Unsigned, U3 as N};
        type R = C<N>;
        type I = CIdeal<N>;

        let zn_module: Arc<CanonModule<R, I>> = Arc::new(
            CanonModule::<R, I>::all_by_dimension(0..=1)
                .find(|module| module.cardinality() == N::to_usize())
                .unwrap(),
        );

        let direct = DirectModule::<R, I>::sumproduct(
            &Arc::clone(&zn_module),
            &Arc::new(zn_module.duplicate()),
        );

        let submodules = direct.clone().submodules_goursat();
        let helper_data = HelperData::<R>::new(&direct);

        let relations_zn_out: Vec<Relation<R, I>> = submodules
            .into_iter()
            .map(|submodule| {
                println!("new submodule: {:?}", submodule);
                for element in submodule.image() {
                    println!("element:{:?}", element)
                }
                Relation::<R, I>::from((&direct, submodule))
            })
            .collect();

        for relation in relations_zn_out.iter() {
            println!("{:?}", relation);
        }

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
        type R = C<N>;
        type I = CIdeal<N>;

        let zn_module: Arc<CanonModule<R, I>> = Arc::new(
            CanonModule::all_by_dimension(0..=2)
                .find(|zn_module| zn_module.cardinality() == 4 && zn_module.dimension() == 1)
                .unwrap(),
        );

        let direct = DirectModule::<R, I>::sumproduct(
            &Arc::clone(&zn_module),
            &Arc::new(zn_module.duplicate()),
        );

        let submodules = direct.clone().submodules_goursat();
        let helper_data = HelperData::<R>::new(&direct);

        let relations_on_zn: Vec<Relation<R, I>> = submodules
            .into_iter()
            .map(|submodule| Relation::<R, I>::from((&direct, submodule)))
            .collect();

        assert_eq!(relations_on_zn.len(), 15);
    }

    #[test]
    fn z3_category_from_function() {
        use typenum::{Unsigned, U3 as N};
        let n = N::to_usize();
        type R = C<N>;
        type I = CIdeal<N>;

        let category = Category::<CanonModule<R, I>, Relation<R, I>>::new(1);

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

        let relations_zn_out: Vec<Relation<R, I>> = hom_sets_fixed_source
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
        use typenum::{Unsigned, U7 as N};
        type R = C<N>;
        type I = CIdeal<N>;

        let category = Category::<CanonModule<R, I>, Relation<R, I>>::new(1);
        print!("{:?}", category);

        let hom_set_zn_zn: Vec<Relation<R, I>> = category
            .hom_sets
            .into_iter()
            .find(|(source, _)| source.cardinality() == N::to_usize().into())
            .expect("there is a hom_set with non-trivial source")
            .1
            .into_iter()
            .find(|(target, _)| target.cardinality() == N::to_usize().into())
            .expect("there is a hom_set with non-trivial target")
            .1;

        let mut hom_set_zn_zn_no_dupes = hom_set_zn_zn.clone();
        hom_set_zn_zn_no_dupes.dedup();

        assert_eq!(hom_set_zn_zn, hom_set_zn_zn_no_dupes);
    }

    #[test]
    fn trivial_to_c2_relations() {
        use typenum::U2 as N;
        // let n: Int = N::to_usize() as Int;
        type R = C<N>;
        type I = CIdeal<N>;

        let mut zn_modules = CanonModule::<R, I>::all_by_dimension(0..=1);

        let z1 = Arc::new(zn_modules.find(|module| module.cardinality() == 1).unwrap());
        let z2 = Arc::new(zn_modules.find(|module| module.cardinality() == 2).unwrap());

        let direct = DirectModule::<R, I>::sumproduct(&z1, &z2);

        assert_eq!(direct.submodules_goursat().len(), 2);
    }

    #[test]
    fn is_a_map() {
        use typenum::{Unsigned, U2 as N};
        type R = C<N>;
        type I = CIdeal<N>;
        let n = N::to_usize();

        let category = Category::<CanonModule<R, I>, Relation<R, I>>::new(1);
        let zn = CanonModule::<R, I>::from_iter([2]);

        let hom_set_znxzn = category.hom_set(&zn, &zn);

        assert_eq!(
            n,
            hom_set_znxzn
                .iter()
                .filter(|relation| relation.is_a_map())
                .count()
        );
    }

    #[test]
    fn identity_morphism() {
        use typenum::U2 as N;
        type R = C<N>;
        type I = CIdeal<N>;

        let morphisms = Category::<CanonModule<R, I>, Relation<R, I>>::new(1).into_morphisms();

        for morphism in morphisms {
            let id_source = Relation::<R, I>::identity(morphism.source());
            let id_target = Relation::<R, I>::identity(morphism.target());

            assert_eq!(morphism, id_source.compose(&morphism));
            assert_eq!(morphism, morphism.compose(&id_target));
        }
    }
}
