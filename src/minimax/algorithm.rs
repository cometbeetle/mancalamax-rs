//! Implementation of the minimax algorithm with alpha-beta pruning for Mancala.

use super::{MoveOrderFn, StateEvalFn};
use crate::game::{Mancala, Move, Player};
use rustc_hash::FxHashMap;
use std::cell::{Cell, RefCell};
use std::hash::Hash;
use std::time::{Duration, Instant};

/// Trait used to specify the wrapper hashing type required by
/// the minimax transposition table implementation.
pub trait TTHash<T: Mancala> {
    type HashWrapper: for<'a> From<&'a T> + Clone + Send + Sync + Hash + PartialEq + Eq;
}

/// Stores the value of a minimax search result.
///
/// If the [`exact`][Self::exact] field is [`true`], then the heuristic
/// was never used in finding the current result (i.e., the search evaluated all
/// possible terminal states).
#[derive(Debug, Clone)]
pub struct SearchResult {
    best_move: Move,
    utility: f32,
    depth_searched: Option<usize>,
    exact: bool,
}

impl SearchResult {
    pub fn best_move(&self) -> Move {
        self.best_move
    }
    pub fn utility(&self) -> f32 {
        self.utility
    }
    pub fn depth_searched(&self) -> Option<usize> {
        self.depth_searched
    }
    pub fn exact(&self) -> bool {
        self.exact
    }
}

/// Stores the value of a minimax search result involving multiple moves.
///
/// If the [`exact`][Self::exact] field is [`true`], then the heuristic
/// was never used in finding the current result (i.e., the search evaluated all
/// possible terminal states).
///
/// Each [`Move`] in the [`best_moves`][Self::best_moves] field has a
/// corresponding utility value in the [`utilities`][Self::utilities] field
/// at the same index.
#[derive(Debug, Clone)]
pub struct MultiSearchResult {
    best_moves: Vec<Move>,
    utilities: Vec<f32>,
    depth_searched: Option<usize>,
    exact: bool,
}

impl MultiSearchResult {
    pub fn best_moves(&self) -> &Vec<Move> {
        &self.best_moves
    }
    pub fn utilities(&self) -> &Vec<f32> {
        &self.utilities
    }
    pub fn depth_searched(&self) -> Option<usize> {
        self.depth_searched
    }
    pub fn exact(&self) -> bool {
        self.exact
    }
}

/// Stores the necessary information for executing the minimax algorithm on a
/// Mancala board state in order to determine the most optimal move (i.e.,
/// the one that maximizes utility, or is calculated as best based on some heuristic).
#[derive(Debug, Clone)]
pub struct Minimax<T: Mancala + TTHash<T>> {
    pub(super) optimize_for: Player,
    pub(super) max_depth: Option<usize>,
    pub(super) max_time: Option<Duration>,
    pub(super) iterative_deepening: bool,
    pub(super) use_t_table: bool,
    pub(super) move_orderer: MoveOrderFn<T>,
    pub(super) evaluator: StateEvalFn<T>,
    pub(super) heuristic: StateEvalFn<T>,
    pub(super) start_time: Cell<Option<Instant>>,
    pub(super) t_table: RefCell<FxHashMap<T::HashWrapper, TTEntry>>,
}

impl<T: Mancala + TTHash<T>> Minimax<T> {
    /// Returns the player for which minimax will optimize the outcome.
    pub fn optimize_for(&self) -> Player {
        self.optimize_for
    }

    /// Returns the set maximum search depth.
    pub fn max_depth(&self) -> Option<usize> {
        self.max_depth
    }

    /// Returns the set maximum search time.
    pub fn max_time(&self) -> Option<Duration> {
        self.max_time
    }

    /// Returns whether iterative deepening will be used during search.
    pub fn iterative_deepening(&self) -> bool {
        self.iterative_deepening
    }

    /// Returns whether a transposition table will be used during search.
    pub fn use_t_table(&self) -> bool {
        self.use_t_table
    }

    /// Returns the start time (if currently running) of the algorithm.
    pub fn start_time(&self) -> Option<Instant> {
        self.start_time.get()
    }

    /// Calls the move ordering function on a given state.
    pub fn order_moves(&self, state: &T) -> Vec<Move> {
        (self.move_orderer)(state)
    }

    /// Calls the evaluation function on a given state.
    pub fn evaluate(&self, state: &T) -> f32 {
        (self.evaluator)(state, self.optimize_for)
    }

    /// Calls the heuristic function on a given state.
    pub fn get_heuristic(&self, state: &T) -> f32 {
        (self.heuristic)(state, self.optimize_for)
    }

    /// Search for the optimal move using the minimax algorithm and
    /// alpha-beta pruning, based on the set configuration parameters.
    ///
    /// If no move was found successfully, returns [`None`].
    pub fn search_utility(&self, state: &T) -> Option<SearchResult> {
        self.start_time.set(Some(Instant::now()));
        let mut best_move: Option<Move> = None;
        let mut utility = f32::NEG_INFINITY;
        let mut depth_searched: Option<usize> = self.max_depth;
        let mut fully_searched = false;

        if self.iterative_deepening {
            for limit in 1usize.. {
                if fully_searched
                    || self.max_depth.is_some_and(|d| limit > d)
                    || self.time_exceeded()
                {
                    break;
                }
                (best_move, utility, fully_searched) =
                    match self.max_value(state, f32::NEG_INFINITY, f32::INFINITY, 0, Some(limit)) {
                        InternalResult::Success {
                            best_move: m,
                            utility: v,
                            exact: c,
                        } => {
                            depth_searched = Some(limit);
                            (Some(m), v, c)
                        }
                        _ => break,
                    };
            }
        } else {
            (best_move, utility, fully_searched) =
                match self.max_value(state, f32::NEG_INFINITY, f32::INFINITY, 0, self.max_depth) {
                    InternalResult::Success {
                        best_move: m,
                        utility: v,
                        exact: c,
                    } => (Some(m), v, c),
                    _ => (None, utility, fully_searched),
                }
        }

        self.start_time.set(None);

        match best_move {
            None => None,
            Some(m) => Some(SearchResult {
                best_move: m,
                utility,
                depth_searched,
                exact: fully_searched,
            }),
        }
    }

    /// Search for all possible moves and their utilities using the minimax algorithm
    /// and alpha-beta pruning, based on the set configuration parameters.
    ///
    /// Note that, to find the utilities for every valid move, alpha-beta pruning is
    /// disabled for the first call to the utility maximizer. This decreases performance
    /// by a significant amount.
    ///
    /// If no moves could be successfully evaluated, returns [`None`].
    pub fn search_utility_all(&self, state: &T) -> Option<MultiSearchResult> {
        self.start_time.set(Some(Instant::now()));
        let mut result: Option<MultiSearchResult> = None;

        if self.iterative_deepening {
            for limit in 1usize.. {
                if self.max_depth.is_some_and(|d| limit > d) || self.time_exceeded() {
                    break;
                }
                result = match self.max_value_all(state, 0, Some(limit)) {
                    Some(r) => Some(r),
                    None => break,
                };
            }
        } else {
            result = self.max_value_all(state, 0, self.max_depth);
        };

        self.start_time.set(None);
        result
    }

    /// Search for the optimal move using the minimax algorithm and
    /// alpha-beta pruning, based on the set configuration parameters.
    ///
    /// To also return the evaluated utility of the optimal move, call
    /// [`search_utility`][Self::search_utility] instead.
    ///
    /// If no move was found successfully, returns [`None`].
    pub fn search(&self, state: &T) -> Option<Move> {
        self.search_utility(state).map(|r| r.best_move)
    }

    /// Determines whether the algorithm has been running longer than requested.
    ///
    /// Used internally inside [`max_value`] and [`min_value`].
    fn time_exceeded(&self) -> bool {
        match (self.start_time.get(), self.max_time) {
            (Some(start), Some(max)) => Instant::now() - start >= max,
            _ => false,
        }
    }

    /// Maximize the utility / heuristic for a given state, and return
    /// the utilities for each checked move.
    fn max_value_all(
        &self,
        state: &T,
        depth: usize,
        limit: Option<usize>,
    ) -> Option<MultiSearchResult> {
        debug_assert!(
            self.start_time.get().is_some(),
            "Minimax search must be started with `search_utility_all()` before calling `max_value_all`"
        );

        // Stop if in a terminal state, or the artificial limit is exceeded.
        if state.is_over() || limit.is_some_and(|d| depth >= d) || self.time_exceeded() {
            return None;
        }

        let depth = depth + 1;
        let mut move_util_exact: Vec<(Move, f32, bool)> = Vec::new();

        for m in self.order_moves_with_tt(state) {
            let new_state = state.make_move(m).unwrap();
            let (utility, exact) = {
                let result = if new_state.current_turn() == state.current_turn() {
                    self.max_value(&new_state, f32::NEG_INFINITY, f32::INFINITY, depth, limit)
                } else {
                    self.min_value(&new_state, f32::NEG_INFINITY, f32::INFINITY, depth, limit)
                };
                match result {
                    InternalResult::Success {
                        utility: v,
                        exact: c,
                        ..
                    } => (v, c),
                    InternalResult::Evaluated {
                        utility: v,
                        exact: c,
                    } => (v, c),
                    InternalResult::Timeout => return None,
                }
            };

            move_util_exact.push((m, utility, exact));
        }

        Some(MultiSearchResult {
            best_moves: move_util_exact.iter().map(|(m, _, _)| m.clone()).collect(),
            utilities: move_util_exact.iter().map(|(_, v, _)| *v).collect(),
            depth_searched: limit,
            exact: move_util_exact.iter().all(|(_, _, c)| *c),
        })
    }

    /// Maximize the utility / heuristic for a given state, and return the
    /// move and associated utility that do so.
    fn max_value(
        &self,
        state: &T,
        mut alpha: f32,
        mut beta: f32,
        depth: usize,
        limit: Option<usize>,
    ) -> InternalResult {
        debug_assert!(
            self.start_time.get().is_some(),
            "Minimax search must be started with `search_utility()` before calling `max_value`"
        );

        // Check transposition table, and narrow bounds if necessary.
        let alpha_orig = alpha;
        let beta_orig = beta;
        let remaining = limit.map(|l| l.saturating_sub(depth)).unwrap_or(usize::MAX);
        if self.use_t_table
            && let Some(r) = self.lookup_restrict(state, remaining, &mut alpha, &mut beta)
        {
            return r;
        }

        // If we are in a terminal state, evaluate utility.
        if state.is_over() {
            return InternalResult::Evaluated {
                utility: self.evaluate(state),
                exact: true,
            };
        }

        // If we have reached the artificial limit, use the heuristic.
        if limit.is_some_and(|d| depth >= d) {
            return InternalResult::Evaluated {
                utility: self.get_heuristic(state),
                exact: false,
            };
        }

        // If the time has expired, return nothing.
        if self.time_exceeded() {
            return InternalResult::Timeout;
        }

        let depth = depth + 1;
        let mut v = f32::NEG_INFINITY;
        let mut best_move: Option<Move> = None;
        let mut exact = true;

        for m in self.order_moves_with_tt(state) {
            let new_state = state.make_move(m).unwrap();
            let (v2, local_exact) = {
                let result = if new_state.current_turn() == state.current_turn() {
                    self.max_value(&new_state, alpha, beta, depth, limit)
                } else {
                    self.min_value(&new_state, alpha, beta, depth, limit)
                };
                match result {
                    InternalResult::Success {
                        utility: v,
                        exact: c,
                        ..
                    } => (v, c),
                    InternalResult::Evaluated {
                        utility: v,
                        exact: c,
                        ..
                    } => (v, c),
                    InternalResult::Timeout => return InternalResult::Timeout,
                }
            };

            if v2 > v {
                v = v2;
                best_move = Some(m);
                alpha = alpha.max(v);
            }

            exact &= local_exact;

            // Alpha > beta: prune.
            if v >= beta {
                return match best_move {
                    None => InternalResult::Evaluated { utility: v, exact },
                    Some(m) => InternalResult::Success {
                        best_move: m,
                        utility: v,
                        exact,
                    },
                };
            }
        }

        // Store results into the transition table, and return the internal result.
        self.store_return(state, alpha_orig, beta_orig, v, remaining, best_move, exact)
    }

    /// Minimize the utility / heuristic for a given state, and return the
    /// move and associated utility that do so.
    fn min_value(
        &self,
        state: &T,
        mut alpha: f32,
        mut beta: f32,
        depth: usize,
        limit: Option<usize>,
    ) -> InternalResult {
        debug_assert!(
            self.start_time.get().is_some(),
            "Minimax search must be started with `search_utility()` before calling `min_value`"
        );

        // Check transposition table, and narrow bounds if necessary.
        let alpha_orig = alpha;
        let beta_orig = beta;
        let remaining = limit.map(|l| l.saturating_sub(depth)).unwrap_or(usize::MAX);
        if self.use_t_table
            && let Some(r) = self.lookup_restrict(state, remaining, &mut alpha, &mut beta)
        {
            return r;
        }

        // If we are in a terminal state, evaluate utility.
        if state.is_over() {
            return InternalResult::Evaluated {
                utility: self.evaluate(state),
                exact: true,
            };
        }

        // If we have reached the artificial limit, use the heuristic.
        if limit.is_some_and(|d| depth >= d) {
            return InternalResult::Evaluated {
                utility: self.get_heuristic(state),
                exact: false,
            };
        }

        // If the time has expired, return nothing.
        if self.time_exceeded() {
            return InternalResult::Timeout;
        }

        let depth = depth + 1;
        let mut v = f32::INFINITY;
        let mut best_move: Option<Move> = None;
        let mut exact = true;

        for m in self.order_moves_with_tt(state) {
            let new_state = state.make_move(m).unwrap();
            let (v2, local_exact) = {
                let result = if new_state.current_turn() == state.current_turn() {
                    self.min_value(&new_state, alpha, beta, depth, limit)
                } else {
                    self.max_value(&new_state, alpha, beta, depth, limit)
                };
                match result {
                    InternalResult::Success {
                        utility: v,
                        exact: c,
                        ..
                    } => (v, c),
                    InternalResult::Evaluated {
                        utility: v,
                        exact: c,
                    } => (v, c),
                    InternalResult::Timeout => return InternalResult::Timeout,
                }
            };

            if v2 < v {
                v = v2;
                best_move = Some(m);
                beta = beta.min(v);
            }

            exact &= local_exact;

            // Alpha > beta: prune.
            if v <= alpha {
                return match best_move {
                    None => InternalResult::Evaluated { utility: v, exact },
                    Some(m) => InternalResult::Success {
                        best_move: m,
                        utility: v,
                        exact,
                    },
                };
            }
        }

        // Store results into the transition table, and return the internal result.
        self.store_return(state, alpha_orig, beta_orig, v, remaining, best_move, exact)
    }

    /// Helper function to perform a lookup in the transposition table.
    fn lookup_restrict(
        &self,
        state: &T,
        remaining: usize,
        alpha: &mut f32,
        beta: &mut f32,
    ) -> Option<InternalResult> {
        if let Some(entry) = self.t_table.borrow().get(&state.into()) {
            if entry.remaining >= remaining {
                match entry.bound {
                    Bound::Between => return Some(entry.to_internal()),
                    Bound::Lower => *alpha = alpha.max(entry.value),
                    Bound::Upper => *beta = beta.min(entry.value),
                }
                if alpha >= beta {
                    return Some(entry.to_internal());
                }
            }
        }
        None
    }

    /// Helper function to store a state in the transposition table, and return the
    /// corresponding internal result.
    fn store_return(
        &self,
        state: &T,
        alpha_orig: f32,
        beta_orig: f32,
        v: f32,
        remaining: usize,
        best_move: Option<Move>,
        exact: bool,
    ) -> InternalResult {
        let bound = if v <= alpha_orig {
            Bound::Upper
        } else if v >= beta_orig {
            Bound::Lower
        } else {
            Bound::Between
        };

        let entry = TTEntry {
            value: v,
            remaining,
            bound,
            best_move,
            exact,
        };

        if self.use_t_table {
            self.t_table.borrow_mut().insert(state.into(), entry);
        }

        entry.to_internal()
    }

    fn tt_best_move(&self, state: &T) -> Option<Move> {
        self.t_table
            .borrow()
            .get(&state.into())
            .and_then(|e| e.best_move)
    }

    fn order_moves_with_tt(&self, state: &T) -> Vec<Move> {
        let mut moves = self.order_moves(state);
        if let Some(tt_move) = self.tt_best_move(state) {
            if let Some(pos) = moves.iter().position(|m| *m == tt_move) {
                let m = moves.remove(pos);
                moves.insert(0, m);
            }
        }
        moves
    }
}

/// Helper enum to store the results of minimax searches.
enum InternalResult {
    Success {
        best_move: Move,
        utility: f32,
        exact: bool,
    },
    Evaluated {
        utility: f32,
        exact: bool,
    },
    Timeout,
}

#[derive(Debug, Clone, Copy)]
enum Bound {
    Between,
    Lower,
    Upper,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct TTEntry {
    value: f32,
    remaining: usize,
    bound: Bound,
    best_move: Option<Move>,
    exact: bool,
}

impl TTEntry {
    /// Helper function to convert TTEntry to an InternalResult.
    fn to_internal(&self) -> InternalResult {
        match self.best_move {
            Some(m) => InternalResult::Success {
                best_move: m,
                utility: self.value,
                exact: self.exact,
            },
            None => InternalResult::Evaluated {
                utility: self.value,
                exact: self.exact,
            },
        }
    }
}
