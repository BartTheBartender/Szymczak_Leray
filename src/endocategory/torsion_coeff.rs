//imports from external sources
use std::collections::HashMap;

//imports from the crate
use crate::{Int, TorsionCoeff};

pub fn torsion_coeff(base: Int, max_dimension: Int) -> HashMap<Int, Vec<TorsionCoeff>> {
    let mut output: HashMap<Int, Vec<TorsionCoeff>> = Default::default();

    for dimension in 1..=max_dimension {
        output.insert(dimension, torsion_coeff_fixed_dim(base, dimension));
    }
    output
}

fn torsion_coeff_fixed_dim(base: Int, dimension: Int) -> Vec<TorsionCoeff> {
    let mut output = Vec::<TorsionCoeff>::new();
    let mut torsion_coeff = vec![0; dimension.into()];
    let tau = tau(base, dimension);

    torsion_coeff_fixed_dim_help(base, dimension, &mut output, torsion_coeff, &tau, 0);
    output
}

fn tau(base: Int, dimension: Int) -> Vec<Int> {
    let mut output = Vec::<Int>::new();

    for x in 1..=base {
        if base % x == 0 {
            output.push(x);
        }
    }

    output
}

fn torsion_coeff_fixed_dim_help(
    base: Int,
    dimension: Int,
    output: &mut Vec<TorsionCoeff>,
    torsion_coeff: TorsionCoeff,
    tau: &Vec<Int>,
    index: usize,
) {
    if index != dimension.into() {
        for x in tau {
            let mut torsion_coeff_ = torsion_coeff.clone();
            torsion_coeff_[index] = *x;
            torsion_coeff_fixed_dim_help(base, dimension, output, torsion_coeff_, tau, index + 1);
        }
    } else {
        output.push(torsion_coeff);
    }
}
