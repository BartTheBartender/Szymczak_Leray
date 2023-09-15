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

/*
impl<T: Ord, V> MarkTree<T, V> {
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn get(&self, key: &Mark<T>) -> Option<&V> {
        self.buffer.get(key)
    }

    pub fn get_mut(&mut self, key: &Mark<T>) -> Option<&mut V> {
        self.buffer.get_mut(key)
    }

    pub fn replace(&mut self, key: &Mark<T>, value: V) -> Option<()> {
        let v = self.buffer.get_mut(key)?;
        *v = value;
        Some(())
    }

    pub fn contains_key(&self, key: &Mark<T>) -> bool {
        self.buffer.contains_key(key)
    }

    pub fn has_same_keys<W>(&self, other: &MarkTree<T, W>) -> bool {
        self.buffer
            .keys()
            .zip(other.buffer.keys())
            .all(|(s, o)| s == o)
    }

    /* # iters */

}
*/

/*
/* ## operations */

impl<T, V> MarkTree<T, V>
where
    T: Clone + Eq + Ord,
{
    pub fn map<W, F>(self, action: F) -> MarkTree<T, W>
    where
        F: Fn(T, V) -> W,
    {
        MarkTree {
            buffer: self
                .buffer
                .into_iter()
                .map(|(key, value)| (key.clone(), action(key.thing, value)))
                .collect(),
        }
    }

    pub fn map_ref<W, F>(&self, action: F) -> MarkTree<T, W>
    where
        F: Fn(&T, &V) -> W,
    {
        MarkTree {
            buffer: self
                .buffer
                .iter()
                .map(|(key, value)| (key.clone(), action(&key.thing, value)))
                .collect(),
        }
    }

    pub fn map_mut<F>(&mut self, action: F)
    where
        F: Fn(&mut V, T),
    {
        for (key, value) in &mut self.buffer {
            action(value, key.coeff.clone());
        }
    }


    /**
    # WARNING

    only performs action on keys present in both trees,
    so if the trees have different structures,
    this will invalidate the structural integrity of the trees
    and — therefore — lead to undefined behaviour
    */
    pub fn combine_ref<U, W, F>(&self, other: &MarkTree<T, U>, action: F) -> MarkTree<T, W>
    where
        F: Fn(&V, &U, T) -> W,
    {
        MarkTree {
            buffer: self
                .buffer
                .iter()
                .filter_map(|(self_key, self_value)| {
                    other.buffer.get(self_key).map(|other_value| {
                        (
                            self_key.clone(),
                            action(self_value, other_value, self_key.coeff.clone()),
                        )
                    })
                })
                .collect::<BTreeMap<_, _>>(),
        }
    }
}
*/

/*
impl<T, V> MarkTree<T, V>
where
    T: Clone + Eq + Ord,
    V: Copy,
{
    // move this to a From conversion in matrix module
    pub fn as_matrix(&self) -> Matrix<V> {
        Matrix::from_cols([self.values().copied().collect()], 1)
    }

}
*/
