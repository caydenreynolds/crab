use std::collections::{BTreeMap, HashMap, HashSet};
use std::hash::Hash;

pub trait ListFunctional<T> {
    fn fpush(self, value: T) -> Self;
}

impl<T> ListFunctional<T> for Vec<T> {
    fn fpush(self, value: T) -> Self {
        let mut vec = self;
        vec.push(value);
        vec
    }
}

pub trait ListReplace<T> {
    fn replace(&mut self, i: usize, value: T);
}

impl<T> ListReplace<T> for Vec<T> {
    fn replace(&mut self, i: usize, value: T) {
        self.insert(i, value);
        self.remove(i + 1);
    }
}

pub trait SetFunctional<T: Eq + Hash> {
    fn finsert(self, key: T) -> Self;
}
impl<T: Eq + Hash> SetFunctional<T> for HashSet<T> {
    fn finsert(self, value: T) -> Self {
        let mut map = self;
        map.insert(value);
        map
    }
}

pub trait MapFunctional<K: Eq + Hash, V> {
    fn finsert(self, key: K, value: V) -> Self;
}

impl<K: Eq + Hash, V> MapFunctional<K, V> for HashMap<K, V> {
    fn finsert(self, key: K, value: V) -> Self {
        let mut map = self;
        map.insert(key, value);
        map
    }
}

impl<K: Eq + Hash + Ord, V> MapFunctional<K, V> for BTreeMap<K, V> {
    fn finsert(self, key: K, value: V) -> Self {
        let mut map = self;
        map.insert(key, value);
        map
    }
}
