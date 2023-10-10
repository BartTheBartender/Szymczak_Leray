use crate::{
    category::{
        morphism::{
            Abelian, Concrete as ConcreteMorphism, Enumerable as EnumerableMorphism, Morphism,
            PreAbelian,
        },
        object::Concrete as ConcreteObject,
    },
    ralg::{
        matrix::Matrix,
        module::{
            canon::object::Object as CanonModule, quotient::Object as QuotObject, ModuleObject,
        },
        ring::{
            ideal::{Ideal, Principal as PrincipalIdeal},
            AdditivePartialGroup, AdditivePartialMonoid, Bezout as BezoutRing, Demesne,
            Factorial as FactorialRing, Ring,
        },
    },
};
use itertools::Itertools;
use std::{fmt, sync::Arc};

/* # Canon to Canon linear morphism */

#[derive(Clone)]
pub struct CanonToCanon<R: Ring, I: Ideal<Parent = R> + Ord> {
    source: Arc<CanonModule<R, I>>,
    target: Arc<CanonModule<R, I>>,
    matrix: Matrix<R>,
}

/* ## helper functions */

impl<R: Ring + Copy, I: Ideal<Parent = R> + Ord> CanonToCanon<R, I> {
    pub fn new(
        source: &Arc<CanonModule<R, I>>,
        target: &Arc<CanonModule<R, I>>,
        matrix: Matrix<R>,
    ) -> Self {
        Self {
            source: Arc::clone(source),
            target: Arc::clone(target),
            matrix,
        }
    }

    pub fn rows(&self) -> impl Iterator<Item = impl Iterator<Item = &R>> + '_ {
        self.matrix.rows()
    }

    pub fn cols(&self) -> impl Iterator<Item = impl Iterator<Item = &R>> + '_ {
        self.matrix.cols()
    }
}

/* ## debug and display */

impl<R: Ring + fmt::Debug, I: Ideal<Parent = R> + Ord + fmt::Debug> fmt::Debug
    for CanonToCanon<R, I>
where
    QuotObject<R, I>: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} : {:?} -> {:?}",
            self.matrix, self.source, self.target
        )
    }
}

impl<R: Ring + fmt::Display, I: Ideal<Parent = R> + Ord + fmt::Display> fmt::Display
    for CanonToCanon<R, I>
where
    QuotObject<R, I>: fmt::Display,
    CanonModule<R, I>: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} : {} -> {}", self.matrix, self.source, self.target)
    }
}

/* ## equality */

impl<R: Ring + Copy, I: Ideal<Parent = R> + Ord> PartialEq for CanonToCanon<R, I> {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source
            && self.target == other.target
            && self
                .matrix
                .clone()
                .try_add(other.matrix.clone().neg())
                .is_some_and(|matrix| {
                    matrix
                        .cols()
                        .all(|col| self.target.element_from_iterator(col.copied()).is_zero())
                })
    }
}

impl<R: Ring + Copy, I: Ideal<Parent = R> + Ord> Eq for CanonToCanon<R, I> {}

/* ## morphism */

impl<R: Ring + Copy, I: Ideal<Parent = R> + Ord> Morphism<CanonModule<R, I>>
    for CanonToCanon<R, I>
{
    type B = Arc<CanonModule<R, I>>;

    fn source(&self) -> Self::B {
        Arc::clone(&self.source)
    }

    fn target(&self) -> Self::B {
        Arc::clone(&self.target)
    }

    fn try_compose(&self, other: &Self) -> Option<Self> {
        (self.target == other.source).then_some(Self {
            source: Arc::clone(&self.source),
            target: Arc::clone(&other.target),
            matrix: self.matrix.compose(&other.matrix),
        })
    }

    fn compose(&self, other: &Self) -> Self {
        assert!(self.target == other.source, "middle object mismatch");
        Self {
            source: Arc::clone(&self.source),
            target: Arc::clone(&other.target),
            matrix: self.matrix.compose(&other.matrix),
        }
    }
}

/* ## enumerable morphism */

impl<R: BezoutRing + Copy, I: PrincipalIdeal<Parent = R> + Ord>
    EnumerableMorphism<CanonModule<R, I>> for CanonToCanon<R, I>
{
    fn hom(source: Self::B, target: Self::B) -> impl Iterator<Item = Self> + Clone {
        match (source.dimension(), target.dimension()) {
            (0, dim) => {
                vec![Self::new(&source, &target, Matrix::from_buffer([], 0, dim))].into_iter()
            }
            (dim, 0) => {
                vec![Self::new(&source, &target, Matrix::from_buffer([], dim, 0))].into_iter()
            }
            (sdim, _tdim) => (0..sdim)
                .map(|_| target.elements())
                .multi_cartesian_product()
                .filter(|cols| {
                    cols.iter()
                        .zip(source.things().clone())
                        .all(|(col, source_coeff)| {
                            col.things().zip(col.clone().into_things()).all(
                                |(target_coeff, value)| {
                                    value.is_zero()
                                        || target_coeff.clone().ideal.generator().is_divisor(
                                            source_coeff
                                                .clone()
                                                .ideal
                                                .generator()
                                                .mul(value.element),
                                        )
                                },
                            )
                        })
                })
                .map(|cols| {
                    Matrix::from_cols(
                        cols.into_iter()
                            .map(|col| col.into_values().collect::<Vec<_>>()),
                        sdim,
                    )
                })
                .map(|matrix| Self::new(&source, &target, matrix))
                .collect::<Vec<_>>()
                .into_iter(),
        }
    }
}

/* ## concrete morphism */

impl<R: Ring + Copy, I: Ideal<Parent = R> + Ord> ConcreteMorphism<CanonModule<R, I>>
    for CanonToCanon<R, I>
{
    fn try_evaluate(
        &self,
        element: <CanonModule<R, I> as ConcreteObject>::Element,
    ) -> Option<<CanonModule<R, I> as ConcreteObject>::Element> {
        self.source
            .is_element(&element)
            .then(|| match self.matrix.shape() {
                (0, _d) | (_d, 0) => self.target.zero(),
                (_s, _t) => self
                    .target
                    .element_from_matrix(Matrix::from(element).compose(&self.matrix)),
            })
    }
}

/* ## pre abelian structure */

impl<R: Ring + Copy, I: Ideal<Parent = R> + Ord> Demesne for CanonToCanon<R, I> {}

impl<R: Ring + Copy, I: Ideal<Parent = R> + Ord> AdditivePartialMonoid for CanonToCanon<R, I> {
    fn try_add(self, other: Self) -> Option<Self> {
        (self.source == other.source && self.target == other.target).then_some(Self {
            source: self.source,
            target: other.target,
            matrix: self.matrix.try_add(other.matrix)?,
        })
    }

    fn own_zero(&self) -> Self {
        Self {
            source: Arc::clone(&self.source),
            target: Arc::clone(&self.target),
            matrix: self.matrix.own_zero(),
        }
    }

    fn is_zero(&self) -> bool {
        self.matrix
            .cols()
            .all(|col| self.target.element_from_iterator(col.copied()).is_zero())
    }

    fn is_negable(&self) -> bool {
        true
    }

    fn try_neg(self) -> Option<Self> {
        Some(self.neg())
    }
}

impl<R: BezoutRing + FactorialRing + Into<u16>, I: PrincipalIdeal<Parent = R> + Ord>
    PreAbelian<CanonModule<R, I>> for CanonToCanon<R, I>
{
    #[allow(clippy::expect_used, reason = "uncought error elsewhere")]
    fn kernel(&self) -> Self {
        let (_u, s, v) = self.matrix.pseudo_smith();
        let (coeffs, columns): (Vec<_>, Vec<_>) = self
            .source
            .things()
            .zip(s.cols().zip(v.cols()))
            .filter_map(|(coeff, (smith_col, v_col))| {
                let x = smith_col
                    .copied()
                    .enumerate()
                    // there will be at most one nonzero element in the column
                    .find(|&(row, r)| {
                        !self
                            .target
                            .things()
                            .nth(row)
                            .expect("row was taken from columns")
                            .attach_element(r)
                            .is_zero()
                    })
                    .map_or_else(
                        || coeff.clone(),
                        |(row, r)| {
                            coeff
                                .clone()
                                .try_div(
                                    self.target
                                        .things()
                                        .nth(row)
                                        .expect("row was taken from columns")
                                        .clone()
                                        .mul(r),
                                )
                                .expect("smithing went wrong, first")
                        },
                    );
                (!x.ideal.is_full()).then_some((
                    x.clone(),
                    v_col
                        .into_iter()
                        .map(|y| {
                            y.mul({
                                coeff
                                    .ideal
                                    .clone()
                                    .generator()
                                    .try_divide(x.ideal.clone().generator())
                                    .next()
                                    .expect("smithing went wrong, second")
                            })
                        })
                        .collect(),
                ))
            })
            .sorted_by(|a, b| Ord::cmp(&a.0.ideal, &b.0.ideal))
            .unzip();

        let nof_cols: usize = columns.len();
        Self::new(
            &Arc::new(CanonModule::<R, I>::from_iter(coeffs)),
            &self.source(),
            Matrix::from_cols_custom(columns, nof_cols, self.source().dimension()),
        )
    }

    #[allow(clippy::expect_used, reason = "structural properties")]
    fn cokernel(&self) -> Self {
        let (u, s, _v) = self.matrix.pseudo_smith();
        let (coeffs, rows): (Vec<_>, Vec<_>) = self
            .target
            .things()
            .zip(s.rows().zip(u.rows()))
            .filter_map(|(coeff, (smith_row, u_row))| {
                let x = smith_row
                    .copied()
                    .find(|&r| !coeff.attach_element(r).is_zero())
                    .map_or_else(
                        || coeff.clone(),
                        |r| {
                            coeff
                                .clone()
                                .try_div(coeff.clone().mul(r))
                                .expect("division by subgroup of self should be possible")
                        },
                    );
                (!x.ideal.is_full()).then_some((x, u_row.copied().collect()))
            })
            .sorted_by(|a, b| Ord::cmp(&a.0.ideal, &b.0.ideal))
            .unzip();

        let nof_rows: usize = rows.len();
        Self::new(
            &self.target(),
            &Arc::new(CanonModule::<R, I>::from_iter(coeffs)),
            Matrix::from_rows_custom(rows, self.target().dimension(), nof_rows),
        )
    }
}

/* ## abelian strucutre */

impl<R: Ring + Copy, I: Ideal<Parent = R> + Ord> AdditivePartialGroup for CanonToCanon<R, I> {
    fn neg(self) -> Self {
        Self {
            source: self.source,
            target: self.target,
            matrix: self.matrix.neg(),
        }
    }

    fn neg_inplace(&mut self) {
        self.matrix.neg_inplace();
    }
}

impl<R: BezoutRing + FactorialRing + Into<u16>, I: PrincipalIdeal<Parent = R> + Ord>
    Abelian<CanonModule<R, I>> for CanonToCanon<R, I>
{
}

// - - -

#[cfg(test)]
mod test {
    #![allow(non_snake_case, reason = "module names look this way")]

    use super::*;
    use crate::ralg::cgroup::{ideal::CIdeal, C};
    use typenum::{U36, U7};

    type R = C<U36>;
    type I = CIdeal<U36>;

    #[test]
    fn equality() {
        let z3 = Arc::new(CanonModule::<R, I>::from_iter([3]));
        assert_eq!(
            CanonToCanon::new(&z3, &z3, Matrix::from_buffer([1].map(R::from), 1, 1),),
            CanonToCanon::new(&z3, &z3, Matrix::from_buffer([4].map(R::from), 1, 1),)
        );

        let z4sq = Arc::new(CanonModule::from_iter(
            [4, 4].map(|x| I::principal(R::from(x))),
        ));
        assert_eq!(
            CanonToCanon::new(
                &z4sq,
                &z4sq,
                Matrix::from_buffer([0, 1, 2, 3].map(R::from), 2, 2),
            ),
            CanonToCanon::new(
                &z4sq,
                &z4sq,
                Matrix::from_buffer([4, 9, 14, 19].map(R::from), 2, 2),
            )
        );
    }

    #[test]
    fn hom_Z1_Z2() {
        let z1 = Arc::new(CanonModule::<R, I>::from_iter([1]));
        let z2 = Arc::new(CanonModule::<R, I>::from_iter([2]));

        let mut hom_z1_z2 = CanonToCanon::hom(Arc::clone(&z1), Arc::clone(&z2));
        assert_eq!(
            hom_z1_z2.next(),
            Some(CanonToCanon::new(&z1, &z2, Matrix::from_buffer([], 0, 1),))
        );
        assert_eq!(hom_z1_z2.next(), None);
    }

    #[test]
    fn hom_Z2_Z1() {
        let z1 = Arc::new(CanonModule::<R, I>::from_iter([1]));
        let z2 = Arc::new(CanonModule::<R, I>::from_iter([2]));

        let mut hom_z2_z1 = CanonToCanon::hom(Arc::clone(&z2), Arc::clone(&z1));
        assert_eq!(
            hom_z2_z1.next(),
            Some(CanonToCanon::new(&z2, &z1, Matrix::from_buffer([], 1, 0),))
        );
        assert_eq!(hom_z2_z1.next(), None);
    }

    #[test]
    fn hom_Z2_Z3() {
        let z2 = Arc::new(CanonModule::<R, I>::from_iter([2]));
        let z3 = Arc::new(CanonModule::<R, I>::from_iter([3]));

        let mut hom_z2_z3 = CanonToCanon::hom(Arc::clone(&z2), Arc::clone(&z3));
        assert_eq!(
            hom_z2_z3.next(),
            Some(CanonToCanon::new(
                &z2,
                &z3,
                Matrix::from_buffer([0].map(R::from), 1, 1),
            ))
        );
        assert_eq!(hom_z2_z3.next(), None);
    }

    #[test]
    fn hom_Z2_Z4() {
        let z2 = Arc::new(CanonModule::<R, I>::from_iter([2]));
        let z4 = Arc::new(CanonModule::<R, I>::from_iter([4]));
        let mut hom_z2_z4 = CanonToCanon::hom(Arc::clone(&z2), Arc::clone(&z4));
        assert_eq!(
            hom_z2_z4.next(),
            Some(CanonToCanon::new(
                &z2,
                &z4,
                Matrix::from_buffer([2].map(R::from), 1, 1),
            ))
        );
        assert_eq!(
            hom_z2_z4.next(),
            Some(CanonToCanon::new(
                &z2,
                &z4,
                Matrix::from_buffer([0].map(R::from), 1, 1),
            ))
        );
        assert_eq!(hom_z2_z4.next(), None);
    }

    #[test]
    fn hom_Z4_Z2() {
        let z2 = Arc::new(CanonModule::<R, I>::from_iter([2]));
        let z4 = Arc::new(CanonModule::<R, I>::from_iter([4]));

        let mut hom_z4_z2 = CanonToCanon::hom(Arc::clone(&z4), Arc::clone(&z2));
        assert_eq!(
            hom_z4_z2.next(),
            Some(CanonToCanon::new(
                &z4,
                &z2,
                Matrix::from_buffer([1].map(R::from), 1, 1),
            ))
        );
        assert_eq!(
            hom_z4_z2.next(),
            Some(CanonToCanon::new(
                &z4,
                &z2,
                Matrix::from_buffer([0].map(R::from), 1, 1),
            ))
        );
        assert_eq!(hom_z4_z2.next(), None);
    }

    #[test]
    fn hom_Z3_Z3() {
        let z3 = Arc::new(CanonModule::<R, I>::from_iter([3]));
        let mut hom_z3_z3 = CanonToCanon::hom(Arc::clone(&z3), Arc::clone(&z3));
        assert_eq!(
            hom_z3_z3.next(),
            Some(CanonToCanon::new(
                &z3,
                &z3,
                Matrix::from_buffer([1].map(R::from), 1, 1),
            ))
        );
        assert_eq!(
            hom_z3_z3.next(),
            Some(CanonToCanon::new(
                &z3,
                &z3,
                Matrix::from_buffer([2].map(R::from), 1, 1),
            ))
        );
        assert_eq!(
            hom_z3_z3.next(),
            Some(CanonToCanon::new(
                &z3,
                &z3,
                Matrix::from_buffer([0].map(R::from), 1, 1),
            ))
        );
        assert_eq!(hom_z3_z3.next(), None);
    }

    #[test]
    fn evaluation() {
        let z1 = Arc::new(CanonModule::<R, I>::from_iter([1]));
        let z6 = Arc::new(CanonModule::<R, I>::from_iter([6]));

        let zero = CanonToCanon::new(&z1, &z1, Matrix::from_buffer([], 0, 0));
        let bot = CanonToCanon::new(&z1, &z6, Matrix::from_buffer([], 0, 2));
        let top = CanonToCanon::new(&z6, &z1, Matrix::from_buffer([], 2, 0));
        let map = CanonToCanon::new(
            &z6,
            &z6,
            Matrix::from_buffer([2, 0, 0, 1].map(R::from), 2, 2),
        );

        // zero
        assert_eq!(
            zero.try_evaluate(z1.element_from_iterator([0].map(R::from).into_iter())),
            Some(z1.element_from_iterator([0].map(R::from).into_iter()))
        );
        assert_eq!(
            zero.try_evaluate(z6.element_from_iterator([0, 0].map(R::from).into_iter())),
            None
        );

        // bot
        assert_eq!(
            bot.try_evaluate(z1.element_from_iterator([0].map(R::from).into_iter())),
            Some(z6.element_from_iterator([0, 0].map(R::from).into_iter()))
        );
        assert_eq!(
            bot.try_evaluate(z6.element_from_iterator([0, 0].map(R::from).into_iter())),
            None
        );

        // top
        assert_eq!(
            top.try_evaluate(z6.element_from_iterator([0, 0].map(R::from).into_iter())),
            Some(z1.element_from_iterator([0].map(R::from).into_iter()))
        );
        assert_eq!(
            top.try_evaluate(z6.element_from_iterator([0, 1].map(R::from).into_iter())),
            Some(z1.element_from_iterator([0].map(R::from).into_iter()))
        );
        assert_eq!(
            top.try_evaluate(z6.element_from_iterator([1, 0].map(R::from).into_iter())),
            Some(z1.element_from_iterator([0].map(R::from).into_iter()))
        );
        assert_eq!(
            top.try_evaluate(z1.element_from_iterator([0].map(R::from).into_iter())),
            None
        );

        // map
        assert_eq!(
            map.try_evaluate(z6.element_from_iterator([0, 0].map(R::from).into_iter())),
            Some(z6.element_from_iterator([0, 0].map(R::from).into_iter()))
        );
        assert_eq!(
            map.try_evaluate(z6.element_from_iterator([0, 1].map(R::from).into_iter())),
            Some(z6.element_from_iterator([0, 1].map(R::from).into_iter()))
        );
        assert_eq!(
            map.try_evaluate(z6.element_from_iterator([1, 0].map(R::from).into_iter())),
            Some(z6.element_from_iterator([2, 0].map(R::from).into_iter()))
        );
        assert_eq!(
            map.try_evaluate(z1.element_from_iterator([0].map(R::from).into_iter())),
            None,
        );
    }

    #[test]
    fn kernel_of_zero_morphism() {
        let z1 = Arc::new(CanonModule::<R, I>::from_iter([1]));
        let z6 = Arc::new(CanonModule::<R, I>::from_iter([6]));
        assert_eq!(
            CanonToCanon::new(&z6, &z1, Matrix::from_buffer([], 2, 0)).kernel(),
            CanonToCanon::new(
                &z6,
                &z6,
                Matrix::from_buffer([1, 0, 0, 1].map(R::from), 2, 2)
            )
        );
    }

    #[test]
    fn kernel_zero() {
        let z1 = Arc::new(CanonModule::<R, I>::from_iter([1]));
        let z6 = Arc::new(CanonModule::<R, I>::from_iter([6]));
        assert_eq!(
            CanonToCanon::new(
                &z6,
                &z6,
                Matrix::from_buffer([1, 0, 0, 2].map(R::from), 2, 2),
            )
            .kernel(),
            CanonToCanon::new(&z1, &z6, Matrix::from_buffer([], 0, 2))
        );
    }

    #[test]
    fn kernel_easy() {
        let z2 = Arc::new(CanonModule::<R, I>::from_iter([2]));
        let z6 = Arc::new(CanonModule::<R, I>::from_iter([6]));
        assert_eq!(
            CanonToCanon::new(
                &z6,
                &z6,
                Matrix::from_buffer([2, 0, 0, 1].map(R::from), 2, 2),
            )
            .kernel(),
            CanonToCanon::new(&z2, &z6, Matrix::from_buffer([1, 0].map(R::from), 1, 2))
        );
    }

    #[test]
    fn kernel_medium() {
        let z2 = Arc::new(CanonModule::<R, I>::from_iter([2]));
        let z2sq = Arc::new(CanonModule::<R, I>::from_iter([2, 2]));
        assert_eq!(
            CanonToCanon::new(
                &z2sq,
                &z2sq,
                Matrix::from_buffer([1, 1, 1, 1].map(R::from), 2, 2)
            )
            .kernel(),
            CanonToCanon::new(&z2, &z2sq, Matrix::from_buffer([1, 1].map(R::from), 1, 2))
        );
    }

    #[test]
    fn kernel_medium_high_field() {
        type R = C<U7>;
        type I = CIdeal<U7>;

        let z7 = Arc::new(CanonModule::<R, I>::from_iter([0]));
        let z7sq = Arc::new(CanonModule::<R, I>::from_iter([0, 0]));
        assert_eq!(
            CanonToCanon::new(&z7sq, &z7, Matrix::from_buffer([6, 4].map(R::from), 2, 1)).kernel(),
            CanonToCanon::new(&z7, &z7sq, Matrix::from_buffer([5, 3].map(R::from), 1, 2))
        );
    }

    #[test]
    fn kernel_asymetric() {
        let z2 = Arc::new(CanonModule::<R, I>::from_iter([2]));
        let z4 = Arc::new(CanonModule::<R, I>::from_iter([4]));
        let z4xz2 = Arc::new(CanonModule::<R, I>::from_iter([4, 2]));
        assert_eq!(
            CanonToCanon::new(&z4xz2, &z2, Matrix::from_buffer([1, 1].map(R::from), 2, 1)).kernel(),
            CanonToCanon::new(&z4, &z4xz2, Matrix::from_buffer([1, 1].map(R::from), 1, 2)),
            "this can produce a map from Z2xZ2 instead if we are not careful"
        );
    }

    #[test]
    fn kernel_hard() {
        let z43 = Arc::new(CanonModule::<R, I>::from_iter([4, 3]));
        let z942 = Arc::new(CanonModule::<R, I>::from_iter([9, 4, 2]));
        assert_eq!(
            CanonToCanon::new(
                &z942,
                &z43,
                Matrix::from_buffer([0, 0, 1, 2, 2, 0].map(R::from), 3, 2),
            )
            .kernel(),
            CanonToCanon::new(
                &z43,
                &z942,
                Matrix::from_buffer([0, 35, 0, 1, 3, 0].map(R::from), 2, 3),
            )
        );
    }

    #[test]
    fn cokernel_of_zero_morphism() {
        let z1 = Arc::new(CanonModule::<R, I>::from_iter([1]));
        let z6 = Arc::new(CanonModule::<R, I>::from_iter([6]));
        assert_eq!(
            CanonToCanon::new(&z1, &z6, Matrix::from_buffer([], 0, 2)).cokernel(),
            CanonToCanon::new(
                &z6,
                &z6,
                Matrix::from_buffer([1, 0, 0, 1].map(R::from), 2, 2)
            )
        );
    }

    #[test]
    fn cokernel_zero() {
        let z1 = Arc::new(CanonModule::<R, I>::from_iter([1]));
        let z6 = Arc::new(CanonModule::<R, I>::from_iter([6]));
        assert_eq!(
            CanonToCanon::new(
                &z6,
                &z6,
                Matrix::from_buffer([1, 0, 0, 2].map(R::from), 2, 2),
            )
            .cokernel(),
            CanonToCanon::new(&z6, &z1, Matrix::from_buffer([], 2, 0))
        );
    }

    #[test]
    fn cokernel_easy() {
        let z2 = Arc::new(CanonModule::<R, I>::from_iter([2]));
        let z6 = Arc::new(CanonModule::<R, I>::from_iter([6]));
        assert_eq!(
            CanonToCanon::new(
                &z6,
                &z6,
                Matrix::from_buffer([2, 0, 0, 1].map(R::from), 2, 2),
            )
            .cokernel(),
            CanonToCanon::new(&z6, &z2, Matrix::from_buffer([1, 0].map(R::from), 2, 1))
        );
    }

    #[test]
    fn cokernel_medium() {
        let z2 = Arc::new(CanonModule::<R, I>::from_iter([2]));
        let z2sq = Arc::new(CanonModule::<R, I>::from_iter([2, 2]));
        assert_eq!(
            CanonToCanon::new(
                &z2sq,
                &z2sq,
                Matrix::from_buffer([1, 1, 1, 1].map(R::from), 2, 2),
            )
            .cokernel(),
            CanonToCanon::new(&z2sq, &z2, Matrix::from_buffer([1, 1].map(R::from), 2, 1))
        );
    }

    #[test]
    fn cokernel_medium_other() {
        let z2 = Arc::new(CanonModule::<R, I>::from_iter([2]));
        let z4 = Arc::new(CanonModule::<R, I>::from_iter([4]));
        let z24 = Arc::new(CanonModule::<R, I>::from_iter([2, 4]));
        assert_eq!(
            CanonToCanon::new(&z4, &z24, Matrix::from_buffer([1, 1].map(R::from), 1, 2)).cokernel(),
            CanonToCanon::new(&z24, &z2, Matrix::from_buffer([35, 1].map(R::from), 2, 1))
        );
    }

    #[test]
    fn cokernel_hard() {
        let z2 = Arc::new(CanonModule::<R, I>::from_iter([2]));
        let z43 = Arc::new(CanonModule::<R, I>::from_iter([4, 3]));
        let z942 = Arc::new(CanonModule::<R, I>::from_iter([9, 4, 2]));
        assert_eq!(
            CanonToCanon::new(
                &z942,
                &z43,
                Matrix::from_buffer([0, 0, 1, 2, 2, 0].map(R::from), 3, 2),
            )
            .cokernel(),
            CanonToCanon::new(&z43, &z2, Matrix::from_buffer([0, 1].map(R::from), 2, 1))
        );
    }
}
