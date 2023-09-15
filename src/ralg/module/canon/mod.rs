use crate::ralg::module::canon::mark::Mark;
use std::collections::BTreeSet;

pub mod element;
mod mark;
pub mod object;

/* # coefficient tree */

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct MarkTree<T: Ord> {
    buffer: BTreeSet<Mark<T>>,
}

unsafe impl<T: Ord + Send> Send for MarkTree<T> {}
unsafe impl<T: Ord + Sync> Sync for MarkTree<T> {}

impl<T: Ord> Default for MarkTree<T> {
    fn default() -> Self {
        Self {
            buffer: BTreeSet::new(),
        }
    }
}

/* ## basic interface */

impl<T: Ord> MarkTree<T> {
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn contains(&self, mark: &Mark<T>) -> bool {
        self.buffer.contains(mark)
    }

    pub fn remove(&mut self, mark: &Mark<T>) -> bool {
        self.buffer.remove(mark)
    }

    /**
    checks whether the things in marks are the same
    disregarding the uuids
    */
    pub fn is_equivalent(&self, other: &Self) -> bool {
        self.buffer
            .iter()
            .zip(other.buffer.iter())
            .all(|(self_mark, other_mark)| self_mark.thing == other_mark.thing)
    }

    /* # iterators */

    pub fn iter(&self) -> impl Iterator<Item = &Mark<T>> + Clone {
        self.buffer.iter()
    }

    pub fn into_iter(self) -> impl Iterator<Item = Mark<T>> {
        self.buffer.into_iter()
    }

    pub fn things(&self) -> impl Iterator<Item = &T> + Clone {
        self.buffer.iter().map(|mark| &mark.thing)
    }

    pub fn into_things(self) -> impl Iterator<Item = T> {
        self.buffer.into_iter().map(|mark| mark.thing)
    }
}

impl<T: Ord + Clone> MarkTree<T> {
    /**
    returns a tree isomorphic to self,
    but with *different* coefficient uuids
    */
    pub fn duplicate(&self) -> Self {
        Self {
            buffer: self.buffer.iter().map(Mark::duplicate).collect(),
        }
    }
}