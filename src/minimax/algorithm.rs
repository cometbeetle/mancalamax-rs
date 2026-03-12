//! Implementation of the minimax algorithm with alpha-beta pruning for Mancala.

use super::{MoveOrderFn, StateEvalFn};
use crate::game::{Mancala, Move, Player};
use std::cell::Cell;
use std::time::{Duration, Instant};

/// Helper enum to store the results of minimax searches.
enum InternalResult {
    Success(Move, f32),
    Evaluated(f32),
    Timeout,
}

/// Stores the value of a minimax search result.
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub best_move: Move,
    pub utility: f32,
    pub depth_searched: Option<usize>,
}

/// Stores the value of a minimax search result involving multiple moves.
///
/// Each [`Move`] in the [`best_moves`][Self::best_moves] field has a
/// corresponding utility value in the [`utilities`][Self::utilities] field
/// at the same index.
#[derive(Debug, Clone)]
pub struct MultiSearchResult {
    pub best_moves: Vec<Move>,
    pub utilities: Vec<f32>,
    pub depth_searched: Option<usize>,
}

/// Stores the necessary information for executing the minimax algorithm on a
/// Mancala board state in order to determine the most optimal move (i.e.,
/// the one that maximizes utility, or is calculated as best based on some heuristic).
#[derive(Debug, Clone)]
pub struct Minimax<T: Mancala> {
    pub(super) optimize_for: Player,
    pub(super) max_depth: Option<usize>,
    pub(super) max_time: Option<Duration>,
    pub(super) iterative_deepening: bool,
    pub(super) move_orderer: MoveOrderFn<T>,
    pub(super) evaluator: StateEvalFn<T>,
    pub(super) heuristic: StateEvalFn<T>,
    pub(super) start_time: Cell<Option<Instant>>,
}

impl<T: Mancala> Minimax<T> {
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

    /// Returns the set maximum search time.
    pub fn iterative_deepening(&self) -> bool {
        self.iterative_deepening
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

        if self.iterative_deepening {
            for limit in 1usize.. {
                depth_searched = Some(limit);
                if self.max_depth.is_some_and(|d| limit > d) || self.time_exceeded() {
                    break;
                }
                (best_move, utility) =
                    match self.max_value(state, f32::NEG_INFINITY, f32::INFINITY, 0, Some(limit)) {
                        InternalResult::Success(m, v) => (Some(m), v),
                        _ => break,
                    };
            }
        } else {
            (best_move, utility) =
                match self.max_value(state, f32::NEG_INFINITY, f32::INFINITY, 0, self.max_depth) {
                    InternalResult::Success(m, v) => (Some(m), v),
                    _ => (None, utility),
                }
        }

        self.start_time.set(None);

        match best_move {
            None => None,
            Some(m) => Some(SearchResult {
                best_move: m,
                utility,
                depth_searched,
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
                    Some(r) => Some(MultiSearchResult {
                        best_moves: r.iter().map(|(m, _)| m.clone()).collect(),
                        utilities: r.iter().map(|(_, v)| v.clone()).collect(),
                        depth_searched: Some(limit),
                    }),
                    None => break,
                };
            }
        } else {
            result = match self.max_value_all(state, 0, self.max_depth) {
                Some(r) => Some(MultiSearchResult {
                    best_moves: r.iter().map(|(m, _)| m.clone()).collect(),
                    utilities: r.iter().map(|(_, v)| v.clone()).collect(),
                    depth_searched: self.max_depth,
                }),
                None => None,
            }
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

    /// Maximize the utility / heuristic for a given state, and return a vector
    /// of (move, utility) pairs that do so.
    fn max_value_all(
        &self,
        state: &T,
        depth: usize,
        limit: Option<usize>,
    ) -> Option<Vec<(Move, f32)>> {
        debug_assert_ne!(
            self.start_time.get(),
            None,
            "Minimax search must be started with `search_utility_all()` before calling `max_value_all`"
        );

        // Stop if in a terminal state, or the artificial limit is exceeded.
        if state.is_over() || limit.is_some_and(|d| depth >= d) || self.time_exceeded() {
            return None;
        }

        let depth = depth + 1;
        let mut move_utils: Vec<(Move, f32)> = Vec::new();

        for m in self.order_moves(state) {
            let new_state = state.make_move(m).unwrap();
            let utility = {
                let result = if new_state.current_turn() == state.current_turn() {
                    self.max_value(&new_state, f32::NEG_INFINITY, f32::INFINITY, depth, limit)
                } else {
                    self.min_value(&new_state, f32::NEG_INFINITY, f32::INFINITY, depth, limit)
                };
                match result {
                    InternalResult::Success(_, v) => v,
                    InternalResult::Evaluated(v) => v,
                    InternalResult::Timeout => return None,
                }
            };

            move_utils.push((m, utility));
        }

        Some(move_utils)
    }

    /// Maximize the utility / heuristic for a given state, and return the
    /// (move, utility) pair that does so.
    fn max_value(
        &self,
        state: &T,
        alpha: f32,
        beta: f32,
        depth: usize,
        limit: Option<usize>,
    ) -> InternalResult {
        debug_assert_ne!(
            self.start_time.get(),
            None,
            "Minimax search must be started with `search_utility()` before calling `max_value`"
        );

        // If we are in a terminal state, evaluate utility.
        if state.is_over() {
            return InternalResult::Evaluated(self.evaluate(state));
        }

        // If we have reached the artificial limit, use the heuristic.
        if limit.is_some_and(|d| depth >= d) {
            return InternalResult::Evaluated(self.get_heuristic(state));
        }

        // If the time has expired, return nothing.
        if self.time_exceeded() {
            return InternalResult::Timeout;
        }

        let depth = depth + 1;
        let mut alpha = alpha;
        let mut v = f32::NEG_INFINITY;
        let mut best_move: Option<Move> = None;

        for m in self.order_moves(state) {
            let new_state = state.make_move(m).unwrap();
            let v2 = {
                let result = if new_state.current_turn() == state.current_turn() {
                    self.max_value(&new_state, alpha, beta, depth, limit)
                } else {
                    self.min_value(&new_state, alpha, beta, depth, limit)
                };
                match result {
                    InternalResult::Success(_, v) => v,
                    InternalResult::Evaluated(v) => v,
                    InternalResult::Timeout => return InternalResult::Timeout,
                }
            };

            if v2 > v {
                v = v2;
                best_move = Some(m);
                alpha = if alpha > v { alpha } else { v };
            }

            // Alpha > beta ==> prune
            if v >= beta {
                return match best_move {
                    None => InternalResult::Evaluated(v),
                    Some(m) => InternalResult::Success(m, v),
                };
            }
        }

        match best_move {
            None => InternalResult::Evaluated(v),
            Some(m) => InternalResult::Success(m, v),
        }
    }

    /// Minimize the utility / heuristic for a given state, and return the
    /// (move, utility) pair that does so.
    fn min_value(
        &self,
        state: &T,
        alpha: f32,
        beta: f32,
        depth: usize,
        limit: Option<usize>,
    ) -> InternalResult {
        debug_assert_ne!(
            self.start_time.get(),
            None,
            "Minimax search must be started with `search_utility()` before calling `min_value`"
        );

        // If we are in a terminal state, evaluate utility.
        if state.is_over() {
            return InternalResult::Evaluated(self.evaluate(state));
        }

        // If we have reached the artificial limit, use the heuristic.
        if limit.is_some_and(|d| depth >= d) {
            return InternalResult::Evaluated(self.get_heuristic(state));
        }

        // If the time has expired, return nothing.
        if self.time_exceeded() {
            return InternalResult::Timeout;
        }

        let depth = depth + 1;
        let mut beta = beta;
        let mut v = f32::INFINITY;
        let mut best_move: Option<Move> = None;

        for m in self.order_moves(state) {
            let new_state = state.make_move(m).unwrap();
            let v2 = {
                let result = if new_state.current_turn() == state.current_turn() {
                    self.min_value(&new_state, alpha, beta, depth, limit)
                } else {
                    self.max_value(&new_state, alpha, beta, depth, limit)
                };
                match result {
                    InternalResult::Success(_, v) => v,
                    InternalResult::Evaluated(v) => v,
                    InternalResult::Timeout => return InternalResult::Timeout,
                }
            };

            if v2 < v {
                v = v2;
                best_move = Some(m);
                beta = if beta < v { beta } else { v };
            }

            // Alpha > beta ==> prune
            if v <= alpha {
                return match best_move {
                    None => InternalResult::Evaluated(v),
                    Some(m) => InternalResult::Success(m, v),
                };
            }
        }

        match best_move {
            None => InternalResult::Evaluated(v),
            Some(m) => InternalResult::Success(m, v),
        }
    }
}
