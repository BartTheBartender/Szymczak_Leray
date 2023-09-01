use crate::{
    rmodule::{canon::CanonModule, direct::DirectModule, ring::SuperRing},
    Int,
};
use std::{iter, marker::PhantomData};

#[derive(Debug)]
pub struct HelperData<R: SuperRing> {
    pub indices: Vec<Int>,
    pub torsion_coeffs_vec: Vec<Int>,
    pub rows: Int,
    pub cols: Int,
    pub capacity: Int,

    super_ring: PhantomData<R>,
}

impl<R: SuperRing> HelperData<R> {
    pub fn new(direct: &DirectModule<R>) -> Self {
        let source = direct.left();
        let target = direct.right();

        let rows = Self::edge_len(&target);
        let cols = Self::edge_len(&source);

        HelperData {
            indices: Self::indices(&source, &target),
            torsion_coeffs_vec: Self::torsion_coeffs_vec(&source, &target),
            rows,
            cols,
            capacity: rows * cols,
            super_ring: PhantomData::<R>,
        }
    }

    fn edge_len(object: &CanonModule<R>) -> Int {
        object.torsion_coeffs().map(|x| x.get()).product()
    }

    fn indices(source: &CanonModule<R>, target: &CanonModule<R>) -> Vec<Int> {
        let mut one_source_target: Vec<Int> = iter::once(1)
            .chain(
                source
                    .torsion_coeffs()
                    .map(|x| x.get())
                    .chain(target.torsion_coeffs().map(|x| x.get())),
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

    fn torsion_coeffs_vec(source: &CanonModule<R>, target: &CanonModule<R>) -> Vec<Int> {
        [
            source
                .torsion_coeffs()
                .map(|x| x.get())
                .collect::<Vec<Int>>(),
            target
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

    #[test]
    fn edge_len() {
        use typenum::U7 as N;
        let n: u16 = 7;

        type R = Fin<N>;
        let zn_canon: CanonModule<R> = CoeffTree::<R, ()>::all_torsion_coeffs(1)
            .into_iter()
            .map(|torsion_coeffs| CanonModule::new(torsion_coeffs))
            .filter(|module| module.cardinality() == n.into())
            .next()
            .expect("there is a zn_module here");

        assert_eq!(HelperData::edge_len(&zn_canon), n.into());

        let znxzn_canon: CanonModule<R> = CoeffTree::<R, ()>::all_torsion_coeffs(2)
            .into_iter()
            .map(|torsion_coeffs| CanonModule::new(torsion_coeffs))
            .filter(|module| module.cardinality() == (n * n).into())
            .next()
            .expect("there is a zn_module here");
        assert_eq!(HelperData::edge_len(&znxzn_canon), (n * n).into());
    }

    #[test]
    fn indices() {
        use typenum::U5 as N;
        let n: u16 = 5;

        type R = Fin<N>;
        let zn_canon: CanonModule<R> = CoeffTree::<R, ()>::all_torsion_coeffs(1)
            .into_iter()
            .map(|torsion_coeffs| CanonModule::new(torsion_coeffs))
            .filter(|module| module.cardinality() == n.into())
            .next()
            .expect("there is a zn_module here");

        assert_eq!(HelperData::indices(&zn_canon, &zn_canon), vec![1, 5]);

        let znxzn_canon: CanonModule<R> = CoeffTree::<R, ()>::all_torsion_coeffs(2)
            .into_iter()
            .map(|torsion_coeffs| CanonModule::new(torsion_coeffs))
            .filter(|module| module.cardinality() == (n * n).into())
            .next()
            .expect("there is a zn_module here");
        assert_eq!(
            HelperData::indices(&znxzn_canon, &znxzn_canon),
            vec![1, 5, 25, 125]
        );
    }

    #[test]
    fn indices_different_modules() {
        use typenum::U15 as N;
        let n: u16 = 5;
        let m: u16 = 3;

        type R = Fin<N>;
        let zn_canon: CanonModule<R> = CoeffTree::<R, ()>::all_torsion_coeffs(1)
            .into_iter()
            .map(|torsion_coeffs| CanonModule::new(torsion_coeffs))
            .filter(|module| module.cardinality() == n.into())
            .next()
            .expect("there is a zn_module here");

        let zm_canon: CanonModule<R> = CoeffTree::<R, ()>::all_torsion_coeffs(1)
            .into_iter()
            .map(|torsion_coeffs| CanonModule::new(torsion_coeffs))
            .filter(|module| module.cardinality() == m.into())
            .next()
            .expect("there is a zm_module here");

        assert_eq!(HelperData::indices(&zn_canon, &zm_canon), vec![1, 5]);
        assert_eq!(HelperData::indices(&zm_canon, &zn_canon), vec![1, 3]);

        let znxzn_canon: CanonModule<R> = CoeffTree::<R, ()>::all_torsion_coeffs(2)
            .into_iter()
            .map(|torsion_coeffs| CanonModule::new(torsion_coeffs))
            .filter(|module| module.cardinality() == (n * n).into())
            .next()
            .expect("there is a znxzn_module here");

        let zmxzm_canon: CanonModule<R> = CoeffTree::<R, ()>::all_torsion_coeffs(2)
            .into_iter()
            .map(|torsion_coeffs| CanonModule::new(torsion_coeffs))
            .filter(|module| module.cardinality() == (m * m).into())
            .next()
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
        let n: u16 = 5;
        let m: u16 = 3;

        type R = Fin<N>;
        let zn_canon: CanonModule<R> = CoeffTree::<R, ()>::all_torsion_coeffs(1)
            .into_iter()
            .map(|torsion_coeffs| CanonModule::new(torsion_coeffs))
            .filter(|module| module.cardinality() == n.into())
            .next()
            .expect("there is a zn_module here");

        let zm_canon: CanonModule<R> = CoeffTree::<R, ()>::all_torsion_coeffs(1)
            .into_iter()
            .map(|torsion_coeffs| CanonModule::new(torsion_coeffs))
            .filter(|module| module.cardinality() == m.into())
            .next()
            .expect("there is a zm_module here");

        assert_eq!(
            HelperData::torsion_coeffs_vec(&zn_canon, &zm_canon),
            vec![5, 3]
        );
        assert_eq!(
            HelperData::torsion_coeffs_vec(&zm_canon, &zn_canon),
            vec![3, 5]
        );

        let znxzn_canon: CanonModule<R> = CoeffTree::<R, ()>::all_torsion_coeffs(2)
            .into_iter()
            .map(|torsion_coeffs| CanonModule::new(torsion_coeffs))
            .filter(|module| module.cardinality() == (n * n).into())
            .next()
            .expect("there is a znxzn_module here");

        let zmxzm_canon: CanonModule<R> = CoeffTree::<R, ()>::all_torsion_coeffs(2)
            .into_iter()
            .map(|torsion_coeffs| CanonModule::new(torsion_coeffs))
            .filter(|module| module.cardinality() == (m * m).into())
            .next()
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
