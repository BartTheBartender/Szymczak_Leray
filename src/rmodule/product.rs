use crate::{
    category::morphism::{AbelianMorphism, Compose, Morphism},
    util::number::versor,
    zmodule::{canon::CanonZModule, map::CanonToCanon, Zahl},
};
use std::sync::Arc;

pub struct BiProductZModule {
    left_inclusion: CanonToCanon,
    right_inclusion: CanonToCanon,
    left_projection: CanonToCanon,
    right_projection: CanonToCanon,
}

impl BiProductZModule {
    // pub fn new(
    //     left_inclusion: CanonToCanon,
    //     right_inclusion: CanonToCanon,
    //     left_projection: CanonToCanon,
    //     right_projection: CanonToCanon,
    // ) -> Self {
    //     Self {
    //         left_inclusion,
    //         right_inclusion,
    //         left_projection,
    //         right_projection,
    //     }
    // }

    pub fn left(&self) -> Arc<CanonZModule> {
        // should be the same as left_projection.target()
        Arc::clone(&self.left_inclusion.source())
    }

    pub fn right(&self) -> Arc<CanonZModule> {
        // should be the same as right_projection.target()
        Arc::clone(&self.right_inclusion.source())
    }

    pub fn submodules_goursat(&self) -> Vec<CanonToCanon> {
        self.left()
            .submodules()
            .iter()
            .zip(self.right().submodules())
            .flat_map(|(left_sub, right_sub)| {
                right_sub
                    .source()
                    .submodules()
                    .into_iter()
                    .map(|right_sub_sub| {
                        let right_quot = right_sub_sub.cokernel();
                        CanonToCanon::epis(left_sub.source(), right_quot.target())
                            .into_iter()
                            .map(|projection| {
                                let equa = equailiser(
                                    right_quot.compose(&smol.left_projection),
                                    projection.compose(&smol.right_projection),
                                );
                                equa.compose(subs_into_self)
                            })
                    })
            })
            .collect()
    }
}

/*
impl From<CanonZModule> for BiProductZModule {
    fn from(canon: CanonZModule) -> Self {
        let dim = canon.dimension();
        let left_coeff = canon
            .torsion_coeff
            .iter()
            .step_by(2)
            .copied()
            .collect::<Vec<Zahl>>();
        let right_coeff = canon
            .torsion_coeff
            .iter()
            .skip(1)
            .step_by(2)
            .copied()
            .collect::<Vec<Zahl>>();
        let canon_arc = Arc::new(canon);
        let left_dim = left_coeff.len();
        let right_dim = right_coeff.len();
        Self {
            left_inclusion: CanonToCanon::new_unchecked(
                Arc::new(CanonZModule::new(left_coeff)),
                Arc::clone(&canon_arc),
                (0..left_dim).map(|index| versor(2 * index, dim)).collect(),
            ),
            right_inclusion: CanonToCanon::new_unchecked(
                Arc::new(CanonZModule::new(right_coeff)),
                Arc::clone(&canon_arc),
                (0..right_dim)
                    .map(|index| versor(2 * index + 1, dim))
                    .collect(),
            ),
        }
    }
}
*/
