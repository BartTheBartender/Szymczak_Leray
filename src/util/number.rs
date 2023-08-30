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
