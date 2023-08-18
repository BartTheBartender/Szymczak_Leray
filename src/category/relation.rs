#[allow(unused_imports)] // DELETE LATER
use crate::{
    category::morphism::{Compose, EndoMorphism, Morphism},
    rmodule::{
        canon::CanonModule, direct::DirectModule, map::CanonToCanon, ring::SuperRing, Module,
    },
    Int,
};

use bitvec::prelude::*;
use std::{
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
    /*
       nie mam pojęcia jak działa ta funkcja,
       musisz ją naprawić samemu
    pub fn new_unchecked(
        elements: Vec<<CanonModule<R> as Module<R>>::Element>,
        helper_indices_normal: &Vec<R>,
        helper_indices_transposed: &Vec<R>,
        helper_capacity: &usize,
        source: Arc<CanonModule<R>>,
        target: Arc<CanonModule<R>>,
    ) -> Self {
        let mut matrix_normal = BitVec::with_capacity(*helper_capacity);
        let mut matrix_transposed = BitVec::with_capacity(*helper_capacity);
        for element in elements.iter() {
            matrix_normal.set(
                element
                    .iter()
                    .zip(helper_indices_normal.iter())
                    .map(|(&x, &y)| x * y)
                    .sum::<Int>() as usize,
                true,
            );

            matrix_transposed.set(
                element
                    .iter()
                    .zip(helper_indices_transposed.iter())
                    .map(|(&x, &y)| x * y)
                    .sum::<Int>() as usize,
                true,
            );
        }

        Relation {
            source,
            target,
            matrix_normal,
            matrix_transposed,
        }
    }
    */

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

/* dlaczego implementujesz to sam?
impl<R: SuperRing> PartialEq for Relation<R> {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source
            && self.target == other.target
            && self.matrix_normal == other.matrix_normal
            && self.matrix_transposed == other.matrix_transposed //to be removed in the future
    }
}
*/

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

impl<R: SuperRing> TryFrom<(&DirectModule<R>, &CanonToCanon<R>)> for Relation<R> {
    type Error = &'static str;
    /**
    the morphism should be mono in order for this conversion to work
    although the implementation neglects to check this

    the morphism should be a submodule of the given module
    */
    fn try_from(pair: (&DirectModule<R>, &CanonToCanon<R>)) -> Result<Self, Self::Error> {
        let (module, submodule) = pair;
        // allocate a matrix for every element of the module
        // for every element of the submodule, apply the inclusion and set the corresponding matrix element to one
        todo!()
    }
}

impl<R: SuperRing + std::hash::Hash> EndoMorphism<CanonModule<R>> for Relation<R> {}

/*
impl Endocategory<CanonModule, CanonModule, Relation> {
    fn hom_set(source: &CanonModule, target: &CanonModule) -> HashSet<Relation> {
        todo!() //to jest funkcja o którą prosiłeś. w szczególności nie musi być d  dokładnie taka, chodzi mi o ideę (nie wiem np jak tutaj uwzględniać zanurzenia modułów w siebiw w tej syngnaturze
        }}

*/

/*
#[cfg(test)]
mod test {

    use super::*;
    use crate::error::Error;

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
}
*/
