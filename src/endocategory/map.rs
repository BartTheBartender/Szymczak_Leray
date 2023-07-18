//imports from external sources
use rayon::prelude::*;
use std::collections::HashMap;
use std::io::Error;

//imports from the crate
use crate::endocategory::morphism::*;
use crate::endocategory::{self, *};
use crate::{Int, TorsionCoeff};

#[derive(Eq, PartialEq, Debug)]

pub struct Map {
    source: TorsionCoeff,
    target: TorsionCoeff,
    pub entries: Vec<Int>,
}

impl Map {
    pub fn hom_set(base: Int, source: &TorsionCoeff, target: &TorsionCoeff) -> Vec<Self> {
        let entries_len = source.len() * target.len();
        let mut entries = vec![0 as Int; entries_len];

        let mut output = Vec::<Self>::new();

        Self::hom_set_help(base, entries_len, &mut output, entries, &source, &target, 0);

        output
    }

    fn hom_set_help(
        base: Int,
        entries_len: usize,
        output: &mut Vec<Self>,
        entries: Vec<Int>,
        source: &TorsionCoeff,
        target: &TorsionCoeff,
        index: usize,
    ) {
        if index != entries_len {
            (0..base).into_iter().for_each(|x| {
                let mut entries_ = entries.clone();
                entries_[index] = x;
                Self::hom_set_help(
                    base,
                    entries_len,
                    output,
                    entries_,
                    source,
                    target,
                    index + 1,
                );
            })
        } else {
            let map = Map {
                source: source.to_vec(),
                target: target.to_vec(),
                entries,
            };
            output.push(map);
        }
    }
}

impl Morphism for Map {
    fn compose_left(&self, other: &Self) -> Result<Self, Error> {
        todo!()
    }

    fn apply_left(&mut self, other: &Self) -> Result<&mut Self, Error> {
        todo!()
    }
}

impl Endocategory<Map> {
    pub fn new(base: Int, max_dimension: Int) -> Self {
        let all_torsion_coeff: HashMap<_, _> = torsion_coeff::torsion_coeff(base, max_dimension);
        let all_torsion_coeff: Vec<_> = all_torsion_coeff.values().collect();

        all_torsion_coeff.par_iter().for_each(|&sources| {
            all_torsion_coeff.par_iter().for_each(|&targets| {
                sources.par_iter().for_each(|source| {
                    targets
                        .par_iter()
                        .for_each(|target| {
                            let hom_set = Map::hom_set(base, source, target);
                        })
                        .collect();
                })
            })
        });
        todo!()
    }
}
