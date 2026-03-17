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
/// If the [`fully_searched`][Self::fully_searched] field is [`true`], then the heuristic
/// was never used in finding the current result (i.e., the search evaluated all
/// possible terminal states).
#[derive(Debug, Clone, Copy)]
pub struct SearchResult {
    found_move: Move,
    utility: f32,
    depth_searched: Option<usize>,
    fully_searched: bool,
}

impl SearchResult {
    pub fn found_move(&self) -> Move {
        self.found_move
    }
    pub fn utility(&self) -> f32 {
        self.utility
    }
    pub fn depth_searched(&self) -> Option<usize> {
        self.depth_searched
    }
    pub fn fully_searched(&self) -> bool {
        self.fully_searched
    }
}

/// Stores the value of a minimax search result involving multiple moves.
///
/// If the [`fully_searched`][Self::fully_searched] field is [`true`], then the heuristic
/// was never used in finding the current result (i.e., the search evaluated all
/// possible terminal states).
///
/// Each [`Move`] in the [`found_moves`][Self::found_moves] field has a
/// corresponding utility value in the [`utilities`][Self::utilities] field
/// at the same index.
#[derive(Debug, Clone)]
pub struct MultiSearchResult {
    found_moves: Vec<Move>,
    utilities: Vec<f32>,
    depth_searched: Option<usize>,
    fully_searched: bool,
}

impl MultiSearchResult {
    pub fn found_moves(&self) -> &Vec<Move> {
        &self.found_moves
    }
    pub fn utilities(&self) -> &Vec<f32> {
        &self.utilities
    }
    pub fn depth_searched(&self) -> Option<usize> {
        self.depth_searched
    }
    pub fn fully_searched(&self) -> bool {
        self.fully_searched
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
        let mut found_move: Option<Move> = None;
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
                (found_move, utility, fully_searched) =
                    match self.max_value(state, f32::NEG_INFINITY, f32::INFINITY, 0, Some(limit)) {
                        InternalResult::Node {
                            found_move: m,
                            utility: v,
                            fully_searched: f,
                        } if m.is_some() => {
                            depth_searched = Some(limit);
                            (m, v, f)
                        }
                        _ => break,
                    };
            }
        } else {
            (found_move, utility, fully_searched) =
                match self.max_value(state, f32::NEG_INFINITY, f32::INFINITY, 0, self.max_depth) {
                    InternalResult::Node {
                        found_move: m,
                        utility: v,
                        fully_searched: f,
                    } => (m, v, f),
                    _ => (found_move, utility, fully_searched),
                }
        }

        self.start_time.set(None);

        match found_move {
            None => None,
            Some(m) => Some(SearchResult {
                found_move: m,
                utility,
                depth_searched,
                fully_searched,
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
                if result.as_ref().is_some_and(|r| r.fully_searched)
                    || self.max_depth.is_some_and(|d| limit > d)
                    || self.time_exceeded()
                {
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
        self.search_utility(state).map(|r| r.found_move)
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
        let mut move_util_term: Vec<(Move, f32, bool)> = Vec::new();

        for m in self.order_moves_with_tt(state) {
            let new_state = state.make_move(m).unwrap();
            let (utility, terminal) = {
                let result = if new_state.current_turn() == state.current_turn() {
                    self.max_value(&new_state, f32::NEG_INFINITY, f32::INFINITY, depth, limit)
                } else {
                    self.min_value(&new_state, f32::NEG_INFINITY, f32::INFINITY, depth, limit)
                };
                match result {
                    InternalResult::Node {
                        utility: v,
                        fully_searched: f,
                        ..
                    } => (v, f),
                    InternalResult::Timeout => return None,
                }
            };

            move_util_term.push((m, utility, terminal));
        }

        Some(MultiSearchResult {
            found_moves: move_util_term.iter().map(|(m, _, _)| m.clone()).collect(),
            utilities: move_util_term.iter().map(|(_, v, _)| *v).collect(),
            depth_searched: limit,
            fully_searched: move_util_term.iter().all(|(_, _, t)| *t),
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

        // Run the common starting procedure.
        let (early_result, alpha_orig, beta_orig, remaining) =
            self.max_min_preamble(state, &mut alpha, &mut beta, depth, limit);
        if let Some(r) = early_result {
            return r;
        }

        let mut v = f32::NEG_INFINITY;
        let mut found_move: Option<Move> = None;
        let mut fully_searched = true;

        for m in self.order_moves_with_tt(state) {
            let new_state = state.make_move(m).unwrap();
            let (v2, local_terminal) = {
                let result = if new_state.current_turn() == state.current_turn() {
                    self.max_value(&new_state, alpha, beta, depth + 1, limit)
                } else {
                    self.min_value(&new_state, alpha, beta, depth + 1, limit)
                };
                match result {
                    InternalResult::Node {
                        utility: v,
                        fully_searched: f,
                        ..
                    } => (v, f),
                    InternalResult::Timeout => return InternalResult::Timeout,
                }
            };

            if v2 > v {
                v = v2;
                found_move = Some(m);
                alpha = alpha.max(v);
            }

            fully_searched &= local_terminal;

            // Alpha > beta: prune.
            if v >= beta {
                return InternalResult::Node {
                    found_move,
                    utility: v,
                    fully_searched,
                };
            }
        }

        // Store results into the transition table, if necessary.
        if self.use_t_table {
            self.tt_store(
                state,
                alpha_orig,
                beta_orig,
                v,
                remaining,
                found_move,
                fully_searched,
            )
        }

        InternalResult::Node {
            found_move,
            utility: v,
            fully_searched,
        }
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

        // Run the common starting procedure.
        let (early_result, alpha_orig, beta_orig, remaining) =
            self.max_min_preamble(state, &mut alpha, &mut beta, depth, limit);
        if let Some(r) = early_result {
            return r;
        }

        let mut v = f32::INFINITY;
        let mut found_move: Option<Move> = None;
        let mut fully_searched = true;

        for m in self.order_moves_with_tt(state) {
            let new_state = state.make_move(m).unwrap();
            let (v2, local_terminal) = {
                let result = if new_state.current_turn() == state.current_turn() {
                    self.min_value(&new_state, alpha, beta, depth + 1, limit)
                } else {
                    self.max_value(&new_state, alpha, beta, depth + 1, limit)
                };
                match result {
                    InternalResult::Node {
                        utility: v,
                        fully_searched: f,
                        ..
                    } => (v, f),
                    InternalResult::Timeout => return InternalResult::Timeout,
                }
            };

            if v2 < v {
                v = v2;
                found_move = Some(m);
                beta = beta.min(v);
            }

            fully_searched &= local_terminal;

            // Alpha > beta: prune.
            if v <= alpha {
                return InternalResult::Node {
                    found_move,
                    utility: v,
                    fully_searched,
                };
            }
        }

        // Store results into the transition table, if necessary.
        if self.use_t_table {
            self.tt_store(
                state,
                alpha_orig,
                beta_orig,
                v,
                remaining,
                found_move,
                fully_searched,
            )
        }

        InternalResult::Node {
            found_move,
            utility: v,
            fully_searched,
        }
    }

    /// Helper function that performs the following actions at the beginning
    /// of either [`max_value`] or [`min_value`]:
    /// - Check if a valid result is in the transposition table.
    /// - Modify the search bounds based on the transposition table, if necessary.
    /// - Check if the state is a terminal state.
    /// - Check if the depth limit has been reached.
    /// - Check if the time limit has been exceeded.
    fn max_min_preamble(
        &self,
        state: &T,
        alpha: &mut f32,
        beta: &mut f32,
        depth: usize,
        limit: Option<usize>,
    ) -> (Option<InternalResult>, f32, f32, usize) {
        // Keep track of the original values for alpha, beta, and the remaining depth.
        let alpha_orig = *alpha;
        let beta_orig = *beta;
        let remaining = limit.map(|l| l.saturating_sub(depth)).unwrap_or(usize::MAX);

        // Check transposition table, and narrow bounds if necessary.
        if let Some(r) = self.tt_probe(state, remaining, alpha, beta) {
            return (Some(r), alpha_orig, beta_orig, remaining);
        }

        // If we are in a terminal state, evaluate utility.
        if state.is_over() {
            let r = InternalResult::Node {
                found_move: None,
                utility: self.evaluate(state),
                fully_searched: true,
            };
            return (Some(r), alpha_orig, beta_orig, remaining);
        }

        // If we have reached the artificial depth limit, use the heuristic.
        if limit.is_some_and(|d| depth >= d) {
            let r = InternalResult::Node {
                found_move: None,
                utility: self.get_heuristic(state),
                fully_searched: false,
            };
            return (Some(r), alpha_orig, beta_orig, remaining);
        }

        // If the time has expired, return nothing by indicating a timeout.
        if self.time_exceeded() {
            return (
                Some(InternalResult::Timeout),
                alpha_orig,
                beta_orig,
                remaining,
            );
        }

        // Continue the search if no termination conditions are met.
        (None, alpha_orig, beta_orig, remaining)
    }

    /// Helper function to probe the transposition table for a valid result.
    fn tt_probe(
        &self,
        state: &T,
        remaining: usize,
        alpha: &mut f32,
        beta: &mut f32,
    ) -> Option<InternalResult> {
        if !self.use_t_table {
            return None;
        }

        self.t_table
            .borrow()
            .get(&state.into())
            .and_then(|e| e.probe(remaining, alpha, beta))
    }

    /// Helper function to store a state in the transposition table.
    fn tt_store(
        &self,
        state: &T,
        alpha_orig: f32,
        beta_orig: f32,
        v: f32,
        remaining: usize,
        found_move: Option<Move>,
        terminal: bool,
    ) {
        let entry = TTEntry::new(v, remaining, found_move, terminal, alpha_orig, beta_orig);

        if self.use_t_table {
            let key = state.into();
            let mut table = self.t_table.borrow_mut();

            if table
                .get(&key)
                .is_none_or(|old| entry.remaining >= old.remaining)
            {
                table.insert(key, entry);
            }
        }
    }

    /// Helper function to get the move stored for the current state's
    /// transposition table entry, if one exists.
    fn get_tt_move(&self, state: &T) -> Option<Move> {
        self.t_table
            .borrow()
            .get(&state.into())
            .and_then(|e| e.found_move)
    }

    /// Helper function to order the moves during minimax search, ensuring
    /// that the transposition entry is tried first, if it exists.
    fn order_moves_with_tt(&self, state: &T) -> Vec<Move> {
        let mut moves = self.order_moves(state);
        if let Some(tt_move) = self.get_tt_move(state) {
            if let Some(pos) = moves.iter().position(|m| *m == tt_move) {
                let m = moves.remove(pos);
                moves.insert(0, m);
            }
        }
        moves
    }
}

/// Helper enum to store the internal results of minimax searches.
enum InternalResult {
    Node {
        found_move: Option<Move>,
        utility: f32,
        fully_searched: bool,
    },
    Timeout,
}

/// Helper enum to store transposition table entry bounds.
#[derive(Debug, Clone, Copy)]
enum ValueBound {
    Exact,
    Lower,
    Upper,
}

/// Helper struct for storing data in the transposition table.
#[derive(Debug, Clone, Copy)]
pub(crate) struct TTEntry {
    utility: f32,
    bound: ValueBound,
    found_move: Option<Move>,
    fully_searched: bool,
    remaining: usize,
}

impl TTEntry {
    fn new(
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

    /// Helper method to convert TTEntry to an InternalResult.
    fn to_internal(&self) -> InternalResult {
        InternalResult::Node {
            found_move: self.found_move,
            utility: self.utility,
            fully_searched: self.fully_searched,
        }
    }

    /// Helper method for getting a valid result from the transposition table,
    /// if one is present for the given search parameters. Also serves to
    /// narrow the search bounds, if necessary.
    fn probe(&self, remaining: usize, alpha: &mut f32, beta: &mut f32) -> Option<InternalResult> {
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
}
