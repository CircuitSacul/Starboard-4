//! Wrappers around DashMap and DashSet that are async-safe.

use std::hash::Hash;

use dashmap::{mapref::one::Ref, DashMap, DashSet};

pub struct AsyncDashMap<K, V> {
    map: DashMap<K, V>,
}

impl<K, V> From<DashMap<K, V>> for AsyncDashMap<K, V> {
    fn from(map: DashMap<K, V>) -> Self {
        Self { map }
    }
}

impl<K, V> AsyncDashMap<K, V>
where
    K: Eq + Hash,
{
    pub fn insert(&self, key: K, value: V) -> Option<V> {
        self.map.insert(key, value)
    }

    pub fn remove(&self, key: &K) -> Option<(K, V)> {
        self.map.remove(key)
    }

    pub fn alter(&self, key: &K, f: impl FnOnce(&K, V) -> V) {
        self.map.alter(key, f);
    }

    pub fn with<R>(&self, key: &K, f: impl FnOnce(&K, &Option<Ref<K, V>>) -> R) -> R {
        f(key, &self.map.get(key))
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

pub struct AsyncDashSet<K> {
    set: DashSet<K>,
}

impl<K> From<DashSet<K>> for AsyncDashSet<K> {
    fn from(set: DashSet<K>) -> Self {
        Self { set }
    }
}

impl<K> AsyncDashSet<K>
where
    K: Eq + Hash,
{
    pub fn insert(&self, key: K) -> bool {
        self.set.insert(key)
    }

    pub fn remove(&self, key: &K) -> Option<K> {
        self.set.remove(key)
    }

    pub fn contains(&self, key: &K) -> bool {
        self.set.contains(key)
    }
}
