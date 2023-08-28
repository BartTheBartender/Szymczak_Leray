pub mod number {
    use crate::rmodule::ring::Zahl;
    use gcd::Gcd;

    // to mogłoby zadziałać dla pozosałych liczbowych też, jakby się uprzeć, ale mi się nie chce
    pub fn divisors(number: Zahl) -> Vec<Zahl> {
        // jedynka i sama liczba zawsze będą dzielnikami, więc nie ma sensu sprawdzać
        match number {
            0 => Vec::new(),
            non_zero_num => {
                let mut divs = vec![1];
                divs.extend((2..non_zero_num).filter(|x| non_zero_num % x == 0));
                if non_zero_num > 1 {
                    divs.push(non_zero_num);
                }
                divs
            }
        }
    }

    pub fn are_coprime(x: Zahl, y: Zahl) -> bool {
        x.gcd(y) == 1
    }

    pub fn versor(index: usize, length: usize) -> Vec<Zahl> {
        let mut vers = vec![0; length];
        if let Some(element) = vers.get_mut(index) {
            *element = 1;
        }
        vers
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn divisors_of_small_ints() {
            assert_eq!(divisors(6), vec![1, 2, 3, 6]);
            assert_eq!(divisors(7), vec![1, 7]);
            assert_eq!(divisors(8), vec![1, 2, 4, 8]);
            assert_eq!(divisors(9), vec![1, 3, 9]);
        }

        #[test]
        fn coprimality() {
            assert!(are_coprime(1, 3));
            assert!(are_coprime(2, 3));
            assert!(!are_coprime(2, 4));
        }

        #[test]
        fn versors() {
            assert_eq!(versor(0, 3), vec![1, 0, 0]);
            assert_eq!(versor(2, 3), vec![0, 0, 1]);
            assert_eq!(versor(4, 4), vec![0, 0, 0, 0]);
            assert_eq!(versor(1, 5), vec![0, 1, 0, 0, 0]);
        }
    }
}

pub mod iterator {
    use crate::rmodule::ring::Zahl;
    use itertools::*;

    pub fn product<T, I: Iterator<Item = T> + Clone>(
        iterator: I,
        n: Zahl,
    ) -> impl Iterator<Item = Vec<T>>
    where
        T: Clone,
    {
        (0..n).map(|_| iterator.clone()).multi_cartesian_product()
    }

    pub trait Dedup<T: PartialEq + Clone> {
        fn clear_duplicates(&mut self);
    }

    impl<T: PartialEq + Clone> Dedup<T> for Vec<T> {
        fn clear_duplicates(&mut self) {
            let mut already_seen = Vec::new();
            self.retain(|item| match already_seen.contains(item) {
                true => false,
                _ => {
                    already_seen.push(item.clone());
                    true
                }
            })
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn products() {
            assert_eq!(
                product(vec![0, 1].into_iter(), 3).collect::<Vec<_>>(),
                vec![
                    vec![0, 0, 0],
                    vec![0, 0, 1],
                    vec![0, 1, 0],
                    vec![0, 1, 1],
                    vec![1, 0, 0],
                    vec![1, 0, 1],
                    vec![1, 1, 0],
                    vec![1, 1, 1]
                ]
            );
        }

        #[test]
        fn deduplicate() {
            let mut og = vec![0, 1, 1, 2, 1, 3, 2, 1, 0, 2, 1, 4, 1];
            og.clear_duplicates();
            assert_eq!(og, vec![0, 1, 2, 3, 4])
        }
    }
}

pub mod category_of_relations {
    use crate::rmodule::{canon::CanonModule, direct::DirectModule, ring::SuperRing};
    use std::iter;

    pub fn helper_indices<R: SuperRing>(
        left: &CanonModule<R>,
        right: &CanonModule<R>,
    ) -> Vec<usize> {
        /*
        let one_target_source_tc = iter::once(1)
            .chain(
                direct
                    .left()
                    .torsion_coeffs()
                    .map(|x| x.into())
                    .chain(direct.right().torsion_coeffs().map(|x| x.into()))
                    .collect::<Vec<_>>()
                    .into_iter(),
            )
            .collect::<Vec<_>>();

        let one_source_target_tc = iter::once(1)
            .chain(
                direct
                    .left()
                    .torsion_coeffs()
                    .map(|x| x.into())
                    .chain(direct.right().torsion_coeffs().map(|x| x.into()))
                    .collect::<Vec<_>>()
                    .into_iter(),
            )
            .collect::<Vec<_>>();
        */

        let mut one_left_right: Vec<usize> = iter::once(1)
            .chain(
                left.torsion_coeffs()
                    .map(|x| x.into())
                    .chain(right.torsion_coeffs().map(|x| x.into())),
            )
            .collect();
        one_left_right.pop();

        let mut prod: usize = 1;
        let output: Vec<usize> = one_left_right
            .into_iter()
            .map(|x| {
                prod *= x;
                prod
            })
            .collect();

        output
    }

    pub fn helper_capacity<R: SuperRing>(left: &CanonModule<R>, right: &CanonModule<R>) -> usize {
        iter::once(1)
            .chain(
                left.torsion_coeffs()
                    .map(|x| x.into())
                    .chain(right.torsion_coeffs().map(|x| x.into())),
            )
            .product()
    }

    pub fn calculate_helper_indices_and_capacity<R: SuperRing>(
        direct: &DirectModule<R>,
    ) -> (Vec<usize>, Vec<usize>, usize) {
        (
            helper_indices(direct.right().as_ref(), direct.left().as_ref()),
            helper_indices(direct.left().as_ref(), direct.right().as_ref()),
            helper_capacity(direct.left().as_ref(), direct.right().as_ref()),
        )
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
            util::category_of_relations::*,
        };
        use std::sync::Arc;

        #[test]
        fn helper_capacity() {
            use typenum::U2 as N;
            type R = Fin<N>;

            let one_dim_modules: Vec<CanonModule<R>> = CoeffTree::<R, ()>::all_torsion_coeffs(1)
                .into_iter()
                .map(|torsion_coeffs| CanonModule::new(torsion_coeffs))
                .collect();

            assert_eq!(one_dim_modules.len(), 2);
        }
    }
}
