use std::hash::Hash;

use dashmap::DashSet;

use crate::utils::async_dash::AsyncDashSet;

pub struct DashSetLock<T>
where
    T: Eq + Hash + Clone,
{
    set: AsyncDashSet<T>,
}

impl<T> Default for DashSetLock<T>
where
    T: Eq + Hash + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> DashSetLock<T>
where
    T: Eq + Hash + Clone,
{
    pub fn new() -> Self {
        Self {
            set: DashSet::new().into(),
        }
    }

    pub fn lock(&self, key: T) -> Option<DashSetLockGuard<T>> {
        if self.set.insert(key.clone()) {
            Some(DashSetLockGuard::new(self, key))
        } else {
            None
        }
    }
}

pub struct DashSetLockGuard<'a, T>
where
    T: Eq + Hash + Clone,
{
    lock: &'a DashSetLock<T>,
    key: T,
}

impl<'a, T> DashSetLockGuard<'a, T>
where
    T: Eq + Hash + Clone,
{
    fn new(lock: &'a DashSetLock<T>, key: T) -> Self {
        Self { lock, key }
    }

    fn release(&self) {
        self.lock.set.remove(&self.key);
    }
}

impl<T> Drop for DashSetLockGuard<'_, T>
where
    T: Eq + Hash + Clone,
{
    fn drop(&mut self) {
        self.release();
    }
}
