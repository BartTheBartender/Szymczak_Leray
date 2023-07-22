use crate::{
    category::{
        morphism::{Compose, Morphism},
        Endocategory,
    },
    error::Error,
    zmodule::canon::CanonZModule,
    Int, TorsionCoeff,
};

use bitvec::vec::BitVec;
use std::{
    collections::HashSet,
    fmt::{self, Display},
    sync::Arc,
};

pub struct Relation {
    pub source: Arc<CanonZModule>,
    pub target: Arc<CanonZModule>,
    pub matrix_normal: BitVec,
    pub matrix_transposed: BitVec,
}

impl Relation {
    pub fn krakowian_product_unchecked(
        //so far no couterexample against and two for xD
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

impl PartialEq for Relation {
    fn eq(&self, other: &Self) -> bool {
        self.matrix_normal == other.matrix_normal
    }
}

impl Display for Relation {
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

impl Morphism<CanonZModule, CanonZModule> for Relation {
    fn source(&self) -> Arc<CanonZModule> {
        Arc::clone(&self.source)
    }

    fn target(&self) -> Arc<CanonZModule> {
        Arc::clone(&self.target)
    }
}

//other * self
impl Compose<CanonZModule, CanonZModule, CanonZModule, Relation> for Relation {
    type Output = Relation;

    fn compose_unchecked(&self, other: &Relation) -> Self::Output {
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

        let output_normal = Relation::krakowian_product_unchecked(
            other.matrix_transposed.as_ref(),
            self.matrix_normal.as_ref(),
            column_size,
        );

        let output_transposed = Relation::krakowian_product_unchecked(
            self.matrix_normal.as_ref(),
            other.matrix_transposed.as_ref(),
            column_size,
        );

        Relation {
            source: self.source(),
            target: other.target(),
            matrix_normal: output_normal,
            matrix_transposed: output_transposed,
        }
    }
}

impl Endocategory<CanonZModule, CanonZModule, Relation> {
    fn hom_set(source: &CanonZModule, target: &CanonZModule) -> HashSet<Relation> {
        todo!() //to jest funkcja o którą prosiłeś. w szczególności nie musi być d  dokładnie taka, chodzi mi o ideę (nie wiem np jak tutaj uwzględniać zanurzenia modułów w siebiw w tej syngnaturze
    }
}
