use std::sync::atomic::{AtomicU64, Ordering};

/* # element with UUID */

static COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Mark<T> {
    pub thing: T,
    index: u64,
}

/* ## helper functions */

impl<T> Mark<T> {
    pub const fn new(thing: T, index: u64) -> Self {
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
            index: COUNTER.fetch_add(1, Ordering::SeqCst),
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
