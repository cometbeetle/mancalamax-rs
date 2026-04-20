//! Components used for the implementation of a concurrent transposition table.

use super::InternalResult;
use crate::game::Move;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{LockResult, RwLock, RwLockReadGuard, RwLockWriteGuard, TryLockError};

/// Helper struct for storing data in the transposition table.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct TTEntry {
    pub(super) utility: f32,
    pub(super) bound: ValueBound,
    pub(super) found_move: Option<Move>,
    pub(super) fully_searched: bool,
    pub(super) remaining: usize,
}

impl TTEntry {
    pub(super) fn new(
        value: f32,
        remaining: usize,
        found_move: Option<Move>,
        fully_searched: bool,
        alpha_orig: f32,
        beta_orig: f32,
    ) -> Self {
        let bound = if value <= alpha_orig {
            ValueBound::Upper
        } else if value >= beta_orig {
            ValueBound::Lower
        } else {
            ValueBound::Exact
        };

        Self {
            utility: value,
            remaining,
            bound,
            found_move,
            fully_searched,
        }
    }

    /// Helper method for getting a valid result from a transposition table entry,
    /// if one is present for the given search parameters. Also serves to narrow
    /// the search bounds, if necessary.
    pub(super) fn probe(
        &self,
        remaining: usize,
        alpha: &mut f32,
        beta: &mut f32,
    ) -> Option<InternalResult> {
        if self.remaining < remaining {
            return None;
        }

        match self.bound {
            ValueBound::Exact => {
                return Some(self.to_internal());
            }
            ValueBound::Lower => {
                *alpha = alpha.max(self.utility);
            }
            ValueBound::Upper => {
                *beta = beta.min(self.utility);
            }
        }

        if *alpha >= *beta {
            Some(self.to_internal())
        } else {
            None
        }
    }

    /// Helper method to convert from TTEntry to InternalResult.
    fn to_internal(&self) -> InternalResult {
        InternalResult::Node {
            found_move: self.found_move,
            utility: self.utility,
            fully_searched: self.fully_searched,
        }
    }
}

/// Helper enum to store transposition table entry bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ValueBound {
    Exact,
    Lower,
    Upper,
}

/// Implementation of a concurrent, auto-expanding transposition table
/// with per-bucket locks.
#[derive(Debug)]
pub(super) struct TTable {
    container: PriorityLock<TableContainer>,
    expanding: AtomicBool,
}

impl TTable {
    pub(super) fn new(init_size: usize) -> Self {
        assert!(init_size > 0, "TTable init_size must be greater than 0");

        let mut buckets = Vec::with_capacity(init_size);
        for _ in 0..init_size {
            buckets.push(RwLock::new(Vec::new()));
        }

        let container = PriorityLock::new(TableContainer {
            buckets,
            size: AtomicUsize::new(0),
        });

        Self {
            container,
            expanding: AtomicBool::new(false),
        }
    }

    pub(super) fn insert(&self, hash: u64, entry: TTEntry) {
        let temp_guard = self.container.read().unwrap();

        // If we need to expand the table, do so, and then continue.
        let size = temp_guard.size.load(Ordering::Relaxed) as f32;
        let n_buckets = temp_guard.buckets.len() as f32;
        let container = if size / n_buckets > 0.75 {
            if self
                .expanding
                .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
                .is_err()
            {
                temp_guard
            } else {
                drop(temp_guard);
                self.expand();
                self.container.read().unwrap()
            }
        } else {
            temp_guard
        };

        let idx = self.bucket_idx(&container, hash);
        let mut bucket = container.buckets[idx].write().unwrap();
        for (k, e) in bucket.iter_mut() {
            if *k == hash {
                *e = entry;
                return;
            }
        }
        bucket.push((hash, entry));
        container.size.fetch_add(1, Ordering::Relaxed);
    }

    pub(super) fn get(&self, hash: u64) -> Option<TTEntry> {
        let container = self.container.read().unwrap();
        let idx = self.bucket_idx(&container, hash);
        let bucket = container.buckets[idx].read().unwrap();
        for (k, e) in bucket.iter() {
            if *k == hash {
                return Some(*e);
            }
        }
        None
    }

    pub(super) fn remove(&mut self, hash: u64) -> Option<TTEntry> {
        let container = self.container.read().unwrap();
        let idx = self.bucket_idx(&container, hash);
        let mut bucket = container.buckets[idx].write().unwrap();
        let pos = bucket.iter().position(|(k, _)| *k == hash)?;
        container.size.fetch_sub(1, Ordering::Relaxed);
        Some(bucket.swap_remove(pos).1)
    }

    pub(super) fn n_buckets(&self) -> usize {
        self.container.read().unwrap().buckets.len()
    }

    fn bucket_idx(&self, guard: &RwLockReadGuard<TableContainer>, key: u64) -> usize {
        key as usize % guard.buckets.len()
    }

    fn expand(&self) {
        let mut container = self.container.write().unwrap();
        let n_buckets = container.buckets.len() * 2;
        let mut buckets = Vec::with_capacity(n_buckets);
        for _ in 0..n_buckets {
            buckets.push(RwLock::new(Vec::new()));
        }
        let new_container = TableContainer {
            buckets,
            size: AtomicUsize::new(0),
        };
        for bucket in container.buckets.drain(..) {
            let mut bucket = bucket.write().unwrap();
            for (k, e) in bucket.drain(..) {
                let idx = k as usize % n_buckets;
                let mut bucket = new_container.buckets[idx].write().unwrap();
                bucket.push((k, e));
            }
        }
        new_container
            .size
            .store(container.size.load(Ordering::Relaxed), Ordering::Relaxed);
        *container = new_container;
        self.expanding.store(false, Ordering::Relaxed);
    }
}

#[derive(Debug)]
struct TableContainer {
    buckets: Vec<RwLock<Vec<(u64, TTEntry)>>>,
    size: AtomicUsize,
}

#[derive(Debug)]
struct PriorityLock<T> {
    lock: RwLock<T>,
    writers_waiting: AtomicUsize,
}

/// Reader-writer lock that gives writers priority. Readers can only
/// acquire the lock if no writers currently hold the lock, and if no
/// writers are currently waiting for the lock.
impl<T> PriorityLock<T> {
    fn new(t: T) -> Self {
        Self {
            lock: RwLock::new(t),
            writers_waiting: AtomicUsize::new(0),
        }
    }

    fn read(&'_ self) -> LockResult<RwLockReadGuard<'_, T>> {
        loop {
            while self.writers_waiting.load(Ordering::Acquire) > 0 {
                std::hint::spin_loop();
            }

            match self.lock.try_read() {
                Ok(guard) => {
                    if self.writers_waiting.load(Ordering::Acquire) == 0 {
                        return Ok(guard);
                    }
                }
                Err(TryLockError::Poisoned(e)) => return Err(e),
                _ => (),
            }

            std::hint::spin_loop();
        }
    }

    fn write(&'_ self) -> LockResult<RwLockWriteGuard<'_, T>> {
        self.writers_waiting.fetch_add(1, Ordering::Release);
        let result = self.lock.write();
        self.writers_waiting.fetch_sub(1, Ordering::Release);
        result
    }
}
