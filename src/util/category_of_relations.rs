use crate::{
    rmodule::{canon::CanonModule, direct::DirectModule, ring::SuperRing},
    Int,
};
use std::{iter, marker::PhantomData};

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

    use typenum::U3 as N;
    type R = Fin<N>;
}
