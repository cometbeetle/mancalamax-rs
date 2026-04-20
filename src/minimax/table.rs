//! Components used for the implementation of a concurrent transposition table.

use super::InternalResult;
use crate::game::Move;
use rustc_hash::FxHashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
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

/// Implementation of a fixed-size, concurrent, sharded transposition table
/// with per-bucket locks.
#[derive(Debug)]
pub(crate) struct TTable {
    buckets: Vec<PriorityLock<FxHashMap<u64, TTEntry>>>,
    size: AtomicUsize,
}

impl TTable {
    pub(super) fn new(n_buckets: usize) -> Self {
        assert!(n_buckets > 0, "n_buckets must be greater than 0");
        assert!(
            n_buckets.is_power_of_two(),
            "n_buckets must be a power of two"
        );

        let mut buckets = Vec::with_capacity(n_buckets);
        for _ in 0..n_buckets {
            buckets.push(PriorityLock::new(FxHashMap::default()));
        }

        Self {
            buckets,
            size: AtomicUsize::new(0),
        }
    }

    pub(super) fn insert(&self, hash: u64, entry: TTEntry) {
        let idx = self.bucket_idx(hash);
        let mut bucket = self.buckets[idx].write().unwrap();
        self.size.fetch_add(1, Ordering::Relaxed);
        bucket.insert(hash, entry);
    }

    pub(super) fn get(&self, hash: u64) -> Option<TTEntry> {
        let idx = self.bucket_idx(hash);
        let bucket = self.buckets[idx].read().unwrap();
        bucket.get(&hash).copied()
    }

    pub(super) fn remove(&mut self, hash: u64) -> Option<TTEntry> {
        let idx = self.bucket_idx(hash);
        let mut bucket = self.buckets[idx].write().unwrap();
        self.size.fetch_sub(1, Ordering::Relaxed);
        bucket.remove(&hash)
    }

    #[inline]
    pub(super) fn n_buckets(&self) -> usize {
        self.buckets.len()
    }

    #[inline]
    fn bucket_idx(&self, key: u64) -> usize {
        key as usize & (self.n_buckets() - 1)
    }
}

#[derive(Debug)]
struct PriorityLock<T> {
    lock: RwLock<T>,
    writers_waiting: AtomicUsize,
}

/// Reader-writer lock that gives writers priority. Readers can only
/// acquire the lock if no writers currently hold the lock, and if no
/// writers are currently waiting for the lock.
///
/// This helps prevent cases where writers are starved, which would prevent
/// updates to the transposition table from succeeding in a timely manner.
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
