use crate::{
    rmodule::{canon::CanonModule, direct::DirectModule, ring::SuperRing},
    Int,
};
use std::{iter, marker::PhantomData};

pub struct HelperData<R: SuperRing> {
    pub capacity: Int,
    pub indices_normal: Vec<Int>,
    pub indices_transposed: Vec<Int>,
    pub torsion_coeffs_vec: Vec<Int>,
    super_ring: PhantomData<R>,
}

impl<R: SuperRing> HelperData<R> {
    pub fn new(direct: &DirectModule<R>) -> Self {
        let left = direct.left();
        let right = direct.right();

        HelperData {
            capacity: Self::capacity(&left, &right),
            indices_normal: Self::indices(&right, &left),
            indices_transposed: Self::indices(&left, &right),
            torsion_coeffs_vec: Self::torsion_coeffs_vec(&right, &left),
            super_ring: PhantomData::<R>,
        }
    }

    fn capacity(left: &CanonModule<R>, right: &CanonModule<R>) -> Int {
        iter::once(1)
            .chain(
                left.torsion_coeffs()
                    .map(|x| x.get())
                    .chain(right.torsion_coeffs().map(|x| x.get())),
            )
            .product()
    }

    fn indices(left: &CanonModule<R>, right: &CanonModule<R>) -> Vec<Int> {
        let mut one_left_right: Vec<Int> = iter::once(1)
            .chain(
                left.torsion_coeffs()
                    .map(|x| x.get())
                    .chain(right.torsion_coeffs().map(|x| x.get())),
            )
            .collect();
        one_left_right.pop();

        let mut prod: Int = 1;
        let output: Vec<Int> = one_left_right
            .into_iter()
            .map(|x| {
                prod *= x;
                prod
            })
            .collect();

        output
    }

    fn torsion_coeffs_vec(left: &CanonModule<R>, right: &CanonModule<R>) -> Vec<Int> {
        [
            left.torsion_coeffs().map(|x| x.get()).collect::<Vec<Int>>(),
            right
                .torsion_coeffs()
                .map(|x| x.get())
                .collect::<Vec<Int>>(),
        ]
        .concat()
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::{
        error::Error,
        rmodule::{
            canon::CanonModule,
            direct::DirectModule,
            ring::{Fin, Ring},
            torsion::CoeffTree,
        },
        util::category_of_relations::HelperData,
    };
    use std::sync::Arc;

    use typenum::U3 as N;
    type R = Fin<N>;

    #[test]
    fn capacities_trivially_equal() {
        let canon_modules: Vec<CanonModule<R>> = CoeffTree::<R, ()>::all_torsion_coeffs(3)
            .into_iter()
            .map(|torsion_coeffs| CanonModule::new(torsion_coeffs))
            .collect();

        let _ = canon_modules.iter().flat_map(|source| {
            canon_modules.iter().map(|target| {
                assert_eq!(
                    HelperData::capacity(source, target),
                    HelperData::capacity(target, source)
                )
            })
        });
    }

    #[test]
    fn capacity() {
        let zn: CanonModule<R> = CoeffTree::<R, ()>::all_torsion_coeffs(1)
            .into_iter()
            .map(|torsion_coeffs| CanonModule::new(torsion_coeffs))
            .next()
            .unwrap();

        assert_eq!(HelperData::capacity(&zn, &zn), 9);
    }

    #[test]
    fn indices() {
        let mut zn_modules = CoeffTree::<R, ()>::all_torsion_coeffs(2)
            .into_iter()
            .map(|torsion_coeffs| CanonModule::new(torsion_coeffs));

        let zn: CanonModule<R> = zn_modules.next().expect("there are exactly two modules");
        let znxzn: CanonModule<R> = zn_modules.next().expect("there are exactly two modules");

        assert!(!zn_modules.next().is_some());

        let indices: Vec<Int> = HelperData::indices(&zn, &znxzn);
        assert_eq!(indices, vec![1, 3, 9]);

        let indices: Vec<Int> = HelperData::indices(&znxzn, &zn);
        assert_eq!(indices, vec![1, 3, 9]);
    }

    #[test]
    fn torsion_coeffs_vec() {
        let mut zn_modules = CoeffTree::<R, ()>::all_torsion_coeffs(2)
            .into_iter()
            .map(|torsion_coeffs| CanonModule::new(torsion_coeffs));

        let zn: CanonModule<R> = zn_modules.next().expect("there are exactly two modules");
        let znxzn: CanonModule<R> = zn_modules.next().expect("there are exactly two modules");

        assert!(!zn_modules.next().is_some());

        let torsion_coeffs_vec: Vec<Int> = HelperData::torsion_coeffs_vec(&zn, &znxzn);
        assert_eq!(torsion_coeffs_vec, vec![3, 3, 3]);
    }
}
