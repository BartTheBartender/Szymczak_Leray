use crate::{
    ralg::{
        module::{canon::object::Object as CanonModule, direct::Object as DirectModule},
        ring::{ideal::Principal as PrincipalIdeal, Ring},
    },
    Int,
};
use std::{iter, marker::PhantomData};

#[derive(Debug)]
pub struct HelperData<R: Ring> {
    pub indices: Vec<Int>,
    pub torsion_coeffs_vec: Vec<Int>,
    pub rows: Int,
    pub cols: Int,
    pub capacity: Int,

    super_ring: PhantomData<R>,
}

impl<R: Ring + Copy + Into<u16>> HelperData<R> {
    pub fn new<I: PrincipalIdeal<Parent = R> + Ord>(direct: &DirectModule<R, I>) -> Self {
        let source = direct.left();
        let target = direct.right();

        let rows = Self::edge_len(&target);
        let cols = Self::edge_len(&source);

        Self {
            indices: Self::indices(&source, &target),
            torsion_coeffs_vec: Self::torsion_coeffs_vec(&source, &target),
            rows,
            cols,
            capacity: rows * cols,
            super_ring: PhantomData::<R>,
        }
    }

    fn edge_len<I: PrincipalIdeal<Parent = R> + Ord>(object: &CanonModule<R, I>) -> Int {
        object.torsion_coeffs_as_u16().product()
    }

    fn indices<I: PrincipalIdeal<Parent = R> + Ord>(
        source: &CanonModule<R, I>,
        target: &CanonModule<R, I>,
    ) -> Vec<Int> {
        let mut one_source_target: Vec<Int> = iter::once(1)
            .chain(
                source
                    .torsion_coeffs_as_u16()
                    .chain(target.torsion_coeffs_as_u16()),
            )
            .collect();
        one_source_target.pop();

        let mut prod: Int = 1;
        let output: Vec<Int> = one_source_target
            .into_iter()
            .map(|x| {
                prod *= x;
                prod
            })
            .collect();

        output
    }

    fn torsion_coeffs_vec<I: PrincipalIdeal<Parent = R> + Ord>(
        source: &CanonModule<R, I>,
        target: &CanonModule<R, I>,
    ) -> Vec<Int> {
        [
            source.torsion_coeffs_as_u16().collect::<Vec<Int>>(),
            target.torsion_coeffs_as_u16().collect::<Vec<Int>>(),
        ]
        .concat()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        category::object::{Concrete, PartiallyEnumerable},
        ralg::cgroup::{ideal::CIdeal, C},
    };

    #[test]
    fn edge_len() {
        use typenum::U7 as N;
        type R = C<N>;
        type I = CIdeal<N>;
        let n: u16 = 7;

        let zn_canon: CanonModule<R, I> = CanonModule::all_by_dimension(0..=1)
            .find(|module| module.cardinality() == n.into())
            .expect("there is a zn_module here");

        assert_eq!(HelperData::edge_len(&zn_canon), n);

        let znxzn_canon: CanonModule<R, I> = CanonModule::all_by_dimension(0..=2)
            .find(|module| module.cardinality() == (n * n).into())
            .expect("there is a zn_module here");
        assert_eq!(HelperData::edge_len(&znxzn_canon), n * n);
    }

    #[test]
    fn indices() {
        use typenum::U5 as N;
        type R = C<N>;
        type I = CIdeal<N>;
        let n: u16 = 5;

        let zn_canon: CanonModule<R, I> = CanonModule::all_by_dimension(0..=1)
            .find(|module| module.cardinality() == n.into())
            .expect("there is a zn_module here");

        assert_eq!(HelperData::indices(&zn_canon, &zn_canon), vec![1, 5]);

        let znxzn_canon: CanonModule<R, I> = CanonModule::all_by_dimension(0..=2)
            .find(|module| module.cardinality() == (n * n).into())
            .expect("there is a zn_module here");
        assert_eq!(
            HelperData::indices(&znxzn_canon, &znxzn_canon),
            vec![1, 5, 25, 125]
        );
    }

    #[test]
    fn indices_different_modules() {
        use typenum::U15 as N;
        type R = C<N>;
        type I = CIdeal<N>;
        let n: u16 = 5;
        let m: u16 = 3;

        let zn_canon: CanonModule<R, I> = CanonModule::all_by_dimension(0..=1)
            .find(|module| module.cardinality() == n.into())
            .expect("there is a zn_module here");

        let zm_canon: CanonModule<R, I> = CanonModule::all_by_dimension(0..=1)
            .find(|module| module.cardinality() == m.into())
            .expect("there is a zm_module here");

        assert_eq!(HelperData::indices(&zn_canon, &zm_canon), vec![1, 5]);
        assert_eq!(HelperData::indices(&zm_canon, &zn_canon), vec![1, 3]);

        let znxzn_canon: CanonModule<R, I> = CanonModule::all_by_dimension(0..=2)
            .find(|module| module.cardinality() == (n * n).into())
            .expect("there is a znxzn_module here");

        let zmxzm_canon: CanonModule<R, I> = CanonModule::all_by_dimension(0..=2)
            .find(|module| module.cardinality() == (m * m).into())
            .expect("there is a zmxzm_module here");

        assert_eq!(
            HelperData::indices(&znxzn_canon, &zmxzm_canon),
            vec![1, 5, 25, 75]
        );
        assert_eq!(
            HelperData::indices(&zmxzm_canon, &znxzn_canon),
            vec![1, 3, 9, 45]
        );
    }

    #[test]
    fn torsion_coeffs_vec() {
        use typenum::U15 as N;
        type R = C<N>;
        type I = CIdeal<N>;
        let n: u16 = 5;
        let m: u16 = 3;

        let zn_canon: CanonModule<R, I> = CanonModule::all_by_dimension(0..=1)
            .find(|module| module.cardinality() == n.into())
            .expect("there is a zn_module here");

        let zm_canon: CanonModule<R, I> = CanonModule::all_by_dimension(0..=1)
            .find(|module| module.cardinality() == m.into())
            .expect("there is a zm_module here");

        assert_eq!(
            HelperData::torsion_coeffs_vec(&zn_canon, &zm_canon),
            vec![5, 3]
        );
        assert_eq!(
            HelperData::torsion_coeffs_vec(&zm_canon, &zn_canon),
            vec![3, 5]
        );

        let znxzn_canon: CanonModule<R, I> = CanonModule::all_by_dimension(0..=2)
            .find(|module| module.cardinality() == (n * n).into())
            .expect("there is a znxzn_module here");

        let zmxzm_canon: CanonModule<R, I> = CanonModule::all_by_dimension(0..=2)
            .find(|module| module.cardinality() == (m * m).into())
            .expect("there is a zmxzm_module here");

        assert_eq!(
            HelperData::torsion_coeffs_vec(&znxzn_canon, &zmxzm_canon),
            vec![5, 5, 3, 3]
        );
        assert_eq!(
            HelperData::torsion_coeffs_vec(&zmxzm_canon, &znxzn_canon),
            vec![3, 3, 5, 5]
        );
    }
}
