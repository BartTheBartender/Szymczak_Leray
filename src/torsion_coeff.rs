#![allow(dead_code)]
#![allow(unused_variables)]
use crate::{Int, BASE, MAX_DIMENSION};

const fn tau_len() -> usize {
    let mut k = 0;
    let mut sigma_len = 0;
    while k != BASE {
        k += 1;
        if BASE % k == 0 {
            sigma_len += 1;
        }
    }
    sigma_len
}

const TAU_LEN: usize = tau_len();

const fn tau_array() -> [usize; TAU_LEN] {
    let mut tau: [usize; TAU_LEN] = [0; TAU_LEN];
    let mut k = 0;
    let mut index = 0;

    while index != TAU_LEN {
        k += 1;
        if BASE % k == 0 {
            tau[index] = k;
            index += 1;
        }
    }

    tau
}

pub const TAU: [usize; TAU_LEN] = tau_array();
const TORSION_COEFF_LEN: usize = TAU_LEN.pow(MAX_DIMENSION as u32);

const fn torsion_coeff() -> [[Int; MAX_DIMENSION]; TORSION_COEFF_LEN] {
    let mut output: [[Int; MAX_DIMENSION]; TORSION_COEFF_LEN] =
        [[0; MAX_DIMENSION]; TORSION_COEFF_LEN];
    let mut curr_indices: [usize; MAX_DIMENSION] = [0; MAX_DIMENSION];

    let mut index = 0;
    let mut first_iteration = true;

    while index < TORSION_COEFF_LEN {
        if first_iteration {
            first_iteration = false;
        } else {
            let mut i = 0;

            while i < MAX_DIMENSION {
                //generowanie

                curr_indices[i] += 1;

                if curr_indices[i] == TAU_LEN {
                    curr_indices[i] = 0;
                    i += 1;
                } else {
                    break;
                }
            }
        }

        let mut i = 0;

        while i < MAX_DIMENSION {
            //przepisywanie

            output[index][i] = TAU[curr_indices[i]] as Int;
            i += 1;
        }

        index += 1;
    }

    output
}

pub const TORSION_COEFF: [[Int; MAX_DIMENSION]; TORSION_COEFF_LEN] = torsion_coeff();
