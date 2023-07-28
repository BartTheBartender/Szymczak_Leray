pub mod number {
    use crate::zmodule::canon::Zahl;
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
    use crate::zmodule::canon::Zahl;
    use itertools::*;

    pub fn product<T>(iterator: Vec<T>, n: Zahl) -> Vec<Vec<T>>
    where
        T: Clone,
    {
        (0..n)
            .map(|_| iterator.clone().into_iter())
            .multi_cartesian_product()
            .collect()
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
                product(vec![0, 1], 3),
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
    use crate::{zmodule::canon::CanonZModule, Int};

    pub fn calculate_helper_indices(
        source: &CanonZModule,
        target: &CanonZModule,
    ) -> (Vec<Int>, Vec<Int>, usize) {
        let source_and_target_tc = [source.torsion_coeff(), target.torsion_coeff()].concat();
        let target_and_source_tc = [target.torsion_coeff(), source.torsion_coeff()].concat();

        let mut helper_indices_normal: Vec<Int> = target_and_source_tc
            .into_iter()
            .scan(1, |acc, num| {
                *acc *= num;
                Some(*acc)
            })
            .collect();
        let mut helper_indices_transposed: Vec<Int> = source_and_target_tc
            .into_iter()
            .scan(1, |acc, num| {
                *acc *= num;
                Some(*acc)
            })
            .collect();

        let helper_capacity = helper_indices_normal.pop().unwrap() as usize;
        let helper_capacity_ = helper_indices_transposed.pop().unwrap() as usize;
        assert_eq!(helper_capacity, helper_capacity_); //to be removed in the future

        helper_indices_normal.insert(0, 1);
        helper_indices_transposed.insert(0, 1);

        (
            helper_indices_normal,
            helper_indices_transposed,
            helper_capacity,
        )
    }
}
