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
