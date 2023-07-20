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

    pub fn all_elements_equal<T>(iterator: &[T]) -> bool
    where
        T: PartialEq,
    {
        iterator.windows(2).all(|w| w[0] == w[1])
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
}
