use nanoid::nanoid;
use std::{cmp, sync::Arc};

/* # element with UUID */

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Mark<T> {
    pub thing: T,
    index: Arc<str>,
}

/* ## helper functions */

impl<T> Mark<T> {
    pub fn new(thing: T, index: Arc<str>) -> Self {
        Self { thing, index }
    }

    pub fn replace<S>(self, new_thing: S) -> Mark<S> {
        Mark {
            thing: new_thing,
            index: self.index,
        }
    }

    pub fn map<S, F: FnOnce(T) -> S>(self, f: F) -> Mark<S> {
        Mark {
            thing: f(self.thing),
            index: self.index,
        }
    }
}

impl<T: Clone> Mark<T> {
    pub fn duplicate(&self) -> Self {
        Self::from(self.thing.clone())
    }
}
/* ## conversion */

impl<T> From<T> for Mark<T> {
    fn from(thing: T) -> Self {
        Self {
            thing,
            index: Arc::from(nanoid!()),
        }
    }
}

impl<T> Mark<Option<T>> {
    pub fn transmute(self) -> Option<Mark<T>> {
        let (maybe_thing, index) = (self.thing, self.index);
        maybe_thing.map(|thing| Mark { thing, index })
    }
}

/* ## send and sync */

unsafe impl<T: Send> Send for Mark<T> {}
unsafe impl<T: Sync> Sync for Mark<T> {}

/* ## order */

impl<T: PartialOrd> PartialOrd for Mark<T> {
    /// lexicographic order, reversed on thing
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match (
            self.thing.partial_cmp(&other.thing),
            self.index.partial_cmp(&other.index),
        ) {
            (None, _) => None,
            (Some(cmp::Ordering::Equal), ord) => ord,
            (Some(ord), _) => Some(ord.reverse()),
        }
    }
}

impl<T: Ord> Ord for Mark<T> {
    /// lexicographic order, reversed on thing
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match (self.thing.cmp(&other.thing), self.index.cmp(&other.index)) {
            (cmp::Ordering::Equal, ord) => ord,
            (ord, _) => ord.reverse(),
        }
    }
}