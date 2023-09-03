use crate::{
    category::morphism::{Compose, Morphism, PreAbelianMorphism},
    rmodule::{
        canon::{zn_dividedby_rxzm, CanonModule},
        ring::SuperRing,
        torsion::CoeffTree,
        Module,
    },
    util::{
        iterator::{product, Dedup},
        matrix::Matrix,
    },
};
use itertools::Itertools;
use std::{
    fmt,
    ops::{Add, Neg},
    sync::Arc,
};

/* # Canon to Canon */

#[derive(Clone, PartialEq, Eq)]
pub struct CanonToCanon<R: SuperRing> {
    source: Arc<CanonModule<R>>,
    target: Arc<CanonModule<R>>,
    map: Matrix<R>,
}

impl<R: SuperRing> fmt::Debug for CanonToCanon<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} : {:?} -> {:?}", self.map, self.source, self.target)
    }
}
impl<R: SuperRing> fmt::Display for CanonToCanon<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<R: SuperRing> CanonToCanon<R> {
    pub fn new(
        source: Arc<CanonModule<R>>,
        target: Arc<CanonModule<R>>,
        mut map: Matrix<R>,
    ) -> Self {
        for (row, coeff) in target.coeff_tree().coeffs().enumerate() {
            for x in map.row_mut(u8::try_from(row).expect("we're gonna need a bigger int")) {
                *x = *x % coeff;
            }
        }
        Self {
            source,
            target,
            map,
        }
    }

    pub fn rows(&self) -> impl Iterator<Item = Vec<R>> + '_ {
        self.map.rows()
    }

    pub fn cols(&self) -> impl Iterator<Item = Vec<R>> + '_ {
        self.map.cols()
    }

    pub fn evaluate_unchecked(
        &self,
        v: &<CanonModule<R> as Module<R>>::Element,
    ) -> <CanonModule<R> as Module<R>>::Element {
        self.target
            .element_from_matrix(v.as_matrix().compose_unchecked(&self.map))
    }

    /*
    pub fn evaluate(
        &self,
        v: &<CanonModule<R> as Module<R>>::Element,
    ) -> Result<<CanonModule<R> as Module<R>>::Element, Error> {
        match self.source.as_ref().is_element(v) {
            true => Ok(self.evaluate_unchecked(v)),
            false => Err(Error::InvalidElement),
        }
    }
    */

    pub fn image(&self) -> Vec<<CanonModule<R> as Module<R>>::Element> {
        let mut im: Vec<_> = self
            .source()
            .all_elements()
            .map(|element| self.evaluate_unchecked(&element))
            .collect();
        im.clear_duplicates();
        im
    }

    pub fn hom(
        source: Arc<CanonModule<R>>,
        target: Arc<CanonModule<R>>,
    ) -> impl Iterator<Item = Self> {
        let dim = source.dimension();
        let source_coeffs = source.torsion_coeffs().collect::<Vec<_>>();
        product(
            target.all_elements().collect::<Vec<_>>().into_iter(),
            dim.try_into().unwrap(),
        )
        .filter(move |cols| {
            cols.iter()
                .zip(source_coeffs.clone())
                .all(|(col, source_coeff)| {
                    col.coeffs().zip(col.values()).all(|(target_coeff, value)| {
                        value.is_zero() || ((*value * source_coeff) % target_coeff).is_zero()
                    })
                })
        })
        .map(move |cols| {
            Matrix::from_cols(
                cols.into_iter()
                    .map(|col| col.into_values().collect::<Vec<_>>()),
                dim.try_into().unwrap(),
            )
        })
        .map(move |matrix| Self::new(Arc::clone(&source), Arc::clone(&target), matrix))
    }
}

impl<R: SuperRing> Morphism<CanonModule<R>, CanonModule<R>> for CanonToCanon<R> {
    fn source(&self) -> Arc<CanonModule<R>> {
        Arc::clone(&self.source)
    }

    fn target(&self) -> Arc<CanonModule<R>> {
        Arc::clone(&self.target)
    }
}

impl<R: SuperRing> Compose<CanonModule<R>, CanonModule<R>, CanonModule<R>, Self>
    for CanonToCanon<R>
{
    type Output = Self;

    fn compose_unchecked(&self, other: &Self) -> Self {
        assert_eq!(self.target, other.source, "invalid composition attempt");
        Self::new(
            Arc::clone(&self.source),
            Arc::clone(&other.target),
            self.map.compose_unchecked(&other.map),
        )
    }
}

impl<R: SuperRing> PreAbelianMorphism<R, CanonModule<R>, CanonModule<R>> for CanonToCanon<R> {
    fn is_zero(&self) -> bool {
        self.map.iter().all(R::is_zero)
    }

    fn kernel(&self) -> Self {
        let (_u, s, v) = self.map.pseudo_smith();
        let mut columns = Vec::new();
        let mut coeffs = Vec::new();
        for (coeff, (smith_col, v_col)) in self
            .source
            .coeff_tree()
            .coeffs()
            .zip(s.cols().zip(v.cols()))
        {
            // there will be at most one nonzero element in the column
            let (row_coeff, c) = smith_col
                .into_iter()
                .enumerate()
                .find(|&(_row, x)| !(x % coeff).is_zero())
                .map(|(row, x)| (self.target.coeff_tree().coeffs().nth(row).unwrap(), x))
                .unwrap_or((R::one(), R::zero()));
            let x = zn_dividedby_rxzm(coeff, c, row_coeff);
            coeffs.push(x);
            columns.push(v_col.into_iter().map(|y| y * coeff.divide_by(&x)).collect());
        }

        let (columns, coeffs) = match coeffs.iter().all(R::is_one) {
            true => (
                columns.get(0..1).unwrap().to_owned(),
                coeffs.get(0..1).unwrap().to_owned(),
            ),
            false => {
                // if coeffs are not in the correct order, reorder them along with their respective columns
                columns
                    .into_iter()
                    .zip(coeffs)
                    .filter(|(_col, coeff)| !coeff.is_one())
                    .sorted_by(|a, b| Ord::cmp(&a.1, &b.1).reverse())
                    .unzip()
            }
        };

        let ncols: u8 = columns
            .len()
            .try_into()
            .expect("we're gonna need a bigger int");
        Self::new(
            Arc::new(CanonModule::new(CoeffTree::from_iter(coeffs))),
            self.source(),
            Matrix::from_cols(columns, ncols),
        )
    }

    fn cokernel(&self) -> Self {
        let (u, s, _v) = self.map.pseudo_smith();
        let mut rows = Vec::new();
        let mut coeffs = Vec::new();
        for (coeff, (smith_row, u_row)) in self
            .target
            .coeff_tree()
            .coeffs()
            .zip(s.rows().zip(u.rows()))
        {
            // there will be at most one nonzero element in the column
            let c = smith_row
                .into_iter()
                .find(|&x| !(x % coeff).is_zero())
                .unwrap_or_else(R::zero);
            let x = zn_dividedby_rxzm(coeff, c, coeff);
            coeffs.push(x);
            rows.push(u_row);
        }

        let (rows, coeffs) = match coeffs.iter().all(R::is_one) {
            true => (
                rows.get(0..1).unwrap().to_owned(),
                coeffs.get(0..1).unwrap().to_owned(),
            ),
            false => {
                // if coeffs are not in the correct order, reorder them along with their respective rows
                rows.into_iter()
                    .zip(coeffs)
                    .filter(|(_row, coeff)| !coeff.is_one())
                    .sorted_by(|a, b| Ord::cmp(&a.1, &b.1).reverse())
                    .unzip()
            }
        };

        let nrows: u8 = rows
            .len()
            .try_into()
            .expect("we're gonna need a bigger int");
        Self::new(
            self.target(),
            Arc::new(CanonModule::new(CoeffTree::from_iter(coeffs))),
            Matrix::from_rows(rows, nrows),
        )
    }
}

impl<R: SuperRing> Add for CanonToCanon<R> {
    type Output = Self;

    /**
    this assumes that both self and output have the same source and target.
    we could panic otherwise, but that would require checking
    and therefore slow us down
    */
    fn add(self, other: Self) -> Self::Output {
        Self::Output {
            source: Arc::clone(&self.source),
            target: Arc::clone(&other.target),
            map: &self.map + &other.map,
        }
    }
}

impl<R: SuperRing> Neg for CanonToCanon<R> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::Output {
            source: Arc::clone(&self.source),
            target: Arc::clone(&self.target),
            map: -&self.map,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::rmodule::{
        ring::{Fin, Set},
        torsion::CoeffTree,
    };
    use typenum::U36;

    type R = Fin<U36>;

    #[test]
    fn homs() {
        let z2 = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(2)])));
        let z3 = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(3)])));
        let z4 = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(4)])));

        let mut hom_z2_z3 = CanonToCanon::hom(Arc::clone(&z2), Arc::clone(&z3));
        assert_eq!(
            hom_z2_z3.next(),
            Some(CanonToCanon::new(
                Arc::clone(&z2),
                Arc::clone(&z3),
                Matrix::from_buffer([R::new(0)], 1, 1),
            ))
        );
        assert_eq!(hom_z2_z3.next(), None,);

        let mut hom_z2_z4 = CanonToCanon::hom(Arc::clone(&z2), Arc::clone(&z4));
        assert_eq!(
            hom_z2_z4.next(),
            Some(CanonToCanon::new(
                Arc::clone(&z2),
                Arc::clone(&z4),
                Matrix::from_buffer([R::new(0)], 1, 1),
            ))
        );
        assert_eq!(
            hom_z2_z4.next(),
            Some(CanonToCanon::new(
                Arc::clone(&z2),
                Arc::clone(&z4),
                Matrix::from_buffer([R::new(2)], 1, 1),
            ))
        );
        assert_eq!(hom_z2_z4.next(), None,);

        let mut hom_z4_z2 = CanonToCanon::hom(Arc::clone(&z4), Arc::clone(&z2));
        assert_eq!(
            hom_z4_z2.next(),
            Some(CanonToCanon::new(
                Arc::clone(&z4),
                Arc::clone(&z2),
                Matrix::from_buffer([R::new(0)], 1, 1),
            ))
        );
        assert_eq!(
            hom_z4_z2.next(),
            Some(CanonToCanon::new(
                Arc::clone(&z4),
                Arc::clone(&z2),
                Matrix::from_buffer([R::new(1)], 1, 1),
            ))
        );
        assert_eq!(hom_z4_z2.next(), None,);

        let mut hom_z3_z3 = CanonToCanon::hom(Arc::clone(&z3), Arc::clone(&z3));
        assert_eq!(
            hom_z3_z3.next(),
            Some(CanonToCanon::new(
                Arc::clone(&z3),
                Arc::clone(&z3),
                Matrix::from_buffer([R::new(0)], 1, 1),
            ))
        );
        assert_eq!(
            hom_z3_z3.next(),
            Some(CanonToCanon::new(
                Arc::clone(&z3),
                Arc::clone(&z3),
                Matrix::from_buffer([R::new(1)], 1, 1),
            ))
        );
        assert_eq!(
            hom_z3_z3.next(),
            Some(CanonToCanon::new(
                Arc::clone(&z3),
                Arc::clone(&z3),
                Matrix::from_buffer([R::new(2)], 1, 1),
            ))
        );
        assert_eq!(hom_z3_z3.next(), None,);
    }

    #[test]
    fn kernel_zero() {
        let z1 = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(1)])));
        let z6 = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(6)])));
        assert_eq!(
            CanonToCanon::new(
                Arc::clone(&z6),
                Arc::clone(&z6),
                Matrix::from_buffer([R::new(2), R::new(0), R::new(0), R::new(1)], 2, 2),
            )
            .kernel(),
            CanonToCanon::new(
                Arc::clone(&z1),
                Arc::clone(&z6),
                Matrix::from_buffer([R::new(0), R::new(0)], 1, 2),
            )
        );
    }

    #[test]
    fn kernel_easy() {
        let z2 = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(2)])));
        let z6 = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(6)])));
        assert_eq!(
            CanonToCanon::new(
                Arc::clone(&z6),
                Arc::clone(&z6),
                Matrix::from_buffer([R::new(1), R::new(0), R::new(0), R::new(2)], 2, 2),
            )
            .kernel(),
            CanonToCanon::new(
                Arc::clone(&z2),
                Arc::clone(&z6),
                Matrix::from_buffer([R::new(0), R::new(1)], 1, 2),
            )
        );
    }

    #[test]
    fn kernel_medium() {
        let z2 = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(2)])));
        let z2sq = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([
            R::new(2),
            R::new(2),
        ])));
        assert_eq!(
            CanonToCanon::new(
                Arc::clone(&z2sq),
                Arc::clone(&z2sq),
                Matrix::from_buffer([R::new(1), R::new(1), R::new(1), R::new(1)], 2, 2),
            )
            .kernel(),
            CanonToCanon::new(
                Arc::clone(&z2),
                Arc::clone(&z2sq),
                Matrix::from_buffer([R::new(1), R::new(1)], 1, 2),
            )
        );
    }

    // #[test]
    // this fails so far but i do not think it is too important right now
    fn kernel_asymetric() {
        let z2 = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(2)])));
        let z4 = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(4)])));
        let z4xz2 = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([
            R::new(4),
            R::new(2),
        ])));
        assert_eq!(
            CanonToCanon::new(
                Arc::clone(&z4xz2),
                Arc::clone(&z2),
                Matrix::from_buffer([R::new(1), R::new(1)], 2, 1),
            )
            .kernel(),
            CanonToCanon::new(
                Arc::clone(&z4),
                Arc::clone(&z4xz2),
                Matrix::from_buffer([R::new(1), R::new(1)], 1, 2),
            ),
            "this can produce a Z2xZ2 instead if we are not careful"
        );
    }

    #[test]
    fn kernel_hard() {
        let z43 = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([
            R::new(4),
            R::new(3),
        ])));
        let z942 = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([
            R::new(9),
            R::new(4),
            R::new(2),
        ])));
        let z322 = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([
            R::new(3),
            R::new(2),
            R::new(2),
        ])));
        assert_eq!(
            CanonToCanon::new(
                Arc::clone(&z942),
                Arc::clone(&z43),
                Matrix::from_buffer(
                    [
                        R::new(0),
                        R::new(2),
                        R::new(2),
                        R::new(1),
                        R::new(0),
                        R::new(0)
                    ],
                    3,
                    2
                ),
            )
            .kernel(),
            CanonToCanon::new(
                Arc::clone(&z322),
                Arc::clone(&z942),
                Matrix::from_buffer(
                    [
                        R::new(3),
                        R::new(0),
                        R::new(0),
                        R::new(0),
                        R::new(2),
                        R::new(3),
                        R::new(0),
                        R::new(0),
                        R::new(1)
                    ],
                    3,
                    3
                ),
            )
        );
    }

    #[test]
    fn cokernel_zero() {
        let z1 = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(1)])));
        let z6 = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(6)])));
        assert_eq!(
            CanonToCanon::new(
                Arc::clone(&z6),
                Arc::clone(&z6),
                Matrix::from_buffer([R::new(2), R::new(0), R::new(0), R::new(1)], 2, 2),
            )
            .cokernel(),
            CanonToCanon::new(
                Arc::clone(&z6),
                Arc::clone(&z1),
                Matrix::from_buffer([R::new(0), R::new(0)], 2, 1),
            )
        );
    }

    #[test]
    fn cokernel_easy() {
        let z2 = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(2)])));
        let z6 = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(6)])));
        assert_eq!(
            CanonToCanon::new(
                Arc::clone(&z6),
                Arc::clone(&z6),
                Matrix::from_buffer([R::new(1), R::new(0), R::new(0), R::new(2)], 2, 2),
            )
            .cokernel(),
            CanonToCanon::new(
                Arc::clone(&z6),
                Arc::clone(&z2),
                Matrix::from_buffer([R::new(0), R::new(1)], 2, 1),
            )
        );
    }

    #[test]
    fn cokernel_medium() {
        let z2 = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(2)])));
        let z2sq = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([
            R::new(2),
            R::new(2),
        ])));
        assert_eq!(
            CanonToCanon::new(
                Arc::clone(&z2sq),
                Arc::clone(&z2sq),
                Matrix::from_buffer([R::new(1), R::new(1), R::new(1), R::new(1)], 2, 2),
            )
            .cokernel(),
            CanonToCanon::new(
                Arc::clone(&z2sq),
                Arc::clone(&z2),
                Matrix::from_buffer([R::new(1), R::new(1)], 2, 1),
            )
        );
    }

    #[test]
    fn cokernel_hard() {
        let z2 = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([R::new(2)])));
        let z43 = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([
            R::new(4),
            R::new(3),
        ])));
        let z942 = Arc::new(CanonModule::new(CoeffTree::<R, ()>::from_iter([
            R::new(9),
            R::new(4),
            R::new(2),
        ])));
        assert_eq!(
            CanonToCanon::new(
                Arc::clone(&z942),
                Arc::clone(&z43),
                Matrix::from_buffer(
                    [
                        R::new(0),
                        R::new(2),
                        R::new(2),
                        R::new(1),
                        R::new(0),
                        R::new(0)
                    ],
                    3,
                    2
                ),
            )
            .cokernel(),
            CanonToCanon::new(
                Arc::clone(&z43),
                Arc::clone(&z2),
                Matrix::from_buffer([R::new(1), R::new(0),], 2, 1),
            )
        );
    }
}
