//! Implementation of the minimax algorithm with alpha-beta pruning for Mancala.

use super::builder::ParMinimaxBuilder;
use super::table::{TTEntry, TTable};
use super::zobrist::{MancalaZobrist, ZobristData};
use super::{InternalResult, MoveOrderFn, MultiSearchResult, SearchResult, StateEvalFn};
use crate::game::{Move, Player};
use rustc_hash::FxHashMap;
use std::sync::{RwLock, mpsc};
use std::thread;
use std::time::{Duration, Instant};

// TODO: Interestingly, only better with iterative deepening with these settings.

// TODO: Clean up, and just compare separate TTs vs. shared TT vs. fully sequential.
// TODO: Then do VTune analysis of cache, how long waiting for stuff, etc. Make detailed.
// TODO: Mention cases where it happens to be faster under these restricted circumstances.

// TODO: Note that if shared table implementation were better, it might be possible
//       to get some speedup. BUT -- NOW IT SEEMS LIKE MY SYSTEM IS ABOUT AS GOOD AS DASHMAP!

/// Stores the necessary information for executing a parallelized variant of the minimax
/// algorithm on a Mancala board state in order to determine the most optimal move (i.e.,
/// the one that maximizes utility, or is calculated as best based on some heuristic).
#[derive(Debug)]
pub struct ParMinimax<T: MancalaZobrist> {
    pub(super) optimize_for: Player,
    pub(super) max_depth: Option<usize>,
    pub(super) max_time: Option<Duration>,
    pub(super) iterative_deepening: bool,
    pub(super) use_t_table: bool,
    pub(super) move_orderer: MoveOrderFn<T>,
    pub(super) evaluator: StateEvalFn<T>,
    pub(super) heuristic: StateEvalFn<T>,
    pub(super) start_time: RwLock<Option<Instant>>,
    pub(super) t_table: TTable,
    pub(super) z_data: RwLock<ZobristData>,
    pub(super) shared_t_table: bool,
}

impl<T: MancalaZobrist> From<ParMinimaxBuilder<T>> for ParMinimax<T> {
    /// Alias for [`ParMinimaxBuilder::build`].
    fn from(value: ParMinimaxBuilder<T>) -> Self {
        value.build()
    }
}

impl<T: MancalaZobrist> From<&ParMinimaxBuilder<T>> for ParMinimax<T> {
    /// Alias for [`ParMinimaxBuilder::build`].
    fn from(value: &ParMinimaxBuilder<T>) -> Self {
        value.build()
    }
}

impl<T: MancalaZobrist> ParMinimax<T> {
    /// Returns the player for which minimax will optimize the outcome.
    #[inline]
    pub fn optimize_for(&self) -> Player {
        self.optimize_for
    }

    /// Returns the set maximum search depth.
    #[inline]
    pub fn max_depth(&self) -> Option<usize> {
        self.max_depth
    }

    /// Returns the set maximum search time.
    #[inline]
    pub fn max_time(&self) -> Option<Duration> {
        self.max_time
    }

    /// Returns whether iterative deepening will be used during search.
    #[inline]
    pub fn iterative_deepening(&self) -> bool {
        self.iterative_deepening
    }

    /// Returns whether a transposition table will be used during search.
    #[inline]
    pub fn use_t_table(&self) -> bool {
        self.use_t_table
    }

    /// Returns the start time (if currently running) of the algorithm.
    #[inline]
    pub fn start_time(&self) -> Option<Instant> {
        *self.start_time.read().unwrap()
    }

    /// Returns a reference to the current Zobrist data.
    #[inline]
    pub fn z_data(&self) -> &RwLock<ZobristData> {
        &self.z_data
    }

    /// Returns the number of shared transposition table buckets.
    #[inline]
    pub fn t_table_buckets(&self) -> usize {
        self.t_table.n_buckets()
    }

    /// Returns whether a shared transposition table will be used (as
    /// opposed to separate, per-thread tables).
    #[inline]
    pub fn shared_t_table(&self) -> bool {
        self.shared_t_table
    }

    /// Calls the move ordering function on a given state.
    #[inline]
    pub fn order_moves(&self, state: &T) -> Vec<Move> {
        (self.move_orderer)(state)
    }

    /// Calls the evaluation function on a given state.
    #[inline]
    pub fn evaluate(&self, state: &T) -> f32 {
        (self.evaluator)(state, self.optimize_for)
    }

    /// Calls the heuristic function on a given state.
    #[inline]
    pub fn get_heuristic(&self, state: &T) -> f32 {
        (self.heuristic)(state, self.optimize_for)
    }

    /// Search for the optimal move using the parallelized minimax algorithm with
    /// alpha-beta pruning, based on the set configuration parameters.
    ///
    /// If no move was found successfully, returns [`None`].
    pub fn search_utility(&self, state: &T) -> Option<SearchResult> {
        *self.start_time.write().unwrap() = Some(Instant::now());
        let mut found_move: Option<Move> = None;
        let mut utility = f32::NEG_INFINITY;
        let mut depth_searched: Option<usize> = self.max_depth;
        let mut fully_searched = false;

        // Ensure the current Zobrist values are valid.
        if !self.z_data.read().unwrap().is_valid_for(state) {
            let mut z_data = self.z_data.write().unwrap();
            *z_data = ZobristData::for_states_like(state, 0x49CB86856BB06133);
        }

        thread::scope(|s| {
            if self.iterative_deepening {
                for limit in 1usize.. {
                    if fully_searched
                        || self.max_depth.is_some_and(|d| limit > d)
                        || self.time_exceeded()
                    {
                        break;
                    }
                    (found_move, utility, fully_searched) = match self.max_value(
                        state,
                        f32::NEG_INFINITY,
                        f32::INFINITY,
                        0,
                        Some(limit),
                        s,
                        None,
                    ) {
                        InternalResult::Node {
                            found_move: m,
                            utility: v,
                            fully_searched: f,
                            ..
                        } if m.is_some() => {
                            depth_searched = Some(limit);
                            (m, v, f)
                        }
                        _ => break,
                    };
                }
            } else {
                (found_move, utility, fully_searched) = match self.max_value(
                    state,
                    f32::NEG_INFINITY,
                    f32::INFINITY,
                    0,
                    self.max_depth,
                    s,
                    None,
                ) {
                    InternalResult::Node {
                        found_move: m,
                        utility: v,
                        fully_searched: f,
                        ..
                    } => (m, v, f),
                    _ => (found_move, utility, fully_searched),
                }
            }
        });

        *self.start_time.write().unwrap() = None;

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

    /// Search for all possible moves and their utilities using the parallelized minimax
    /// algorithm with alpha-beta pruning, based on the set configuration parameters.
    ///
    /// Note that, to find the utilities for every valid move, alpha-beta pruning is
    /// disabled for the first call to the utility maximizer. This decreases performance
    /// by a significant amount.
    ///
    /// If no moves could be successfully evaluated, returns [`None`].
    pub fn search_utility_all(&self, state: &T) -> Option<MultiSearchResult> {
        *self.start_time.write().unwrap() = Some(Instant::now());
        let mut result: Option<MultiSearchResult> = None;

        // Ensure the current Zobrist values are valid.
        if !self.z_data.read().unwrap().is_valid_for(state) {
            let mut z_data = self.z_data.write().unwrap();
            *z_data = ZobristData::for_states_like(state, 0x49CB86856BB06133);
        }

        thread::scope(|s| {
            if self.iterative_deepening {
                for limit in 1usize.. {
                    if result.as_ref().is_some_and(|r| r.fully_searched)
                        || self.max_depth.is_some_and(|d| limit > d)
                        || self.time_exceeded()
                    {
                        break;
                    }
                    result = match self.max_value_all(state, 0, Some(limit), s) {
                        Some(r) => Some(r),
                        None => break,
                    };
                }
            } else {
                result = self.max_value_all(state, 0, self.max_depth, s);
            };

            *self.start_time.write().unwrap() = None;
            result
        })
    }

    /// Search for the optimal move using the parallelized minimax algorithm with
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
        match (*self.start_time.read().unwrap(), self.max_time) {
            (Some(start), Some(max)) => Instant::now() - start >= max,
            _ => false,
        }
    }

    /// Maximize the utility / heuristic for a given state, and return
    /// the utilities for each checked move.
    fn max_value_all<'scope, 'env>(
        &'env self,
        state: &T,
        depth: usize,
        limit: Option<usize>,
        scope: &'scope thread::Scope<'scope, 'env>,
    ) -> Option<MultiSearchResult>
    where
        T: 'env,
        'env: 'scope,
    {
        debug_assert!(
            self.start_time.read().unwrap().is_some(),
            "Minimax search must be started with `search_utility_all()` before calling `max_value_all`"
        );

        // Stop if in a terminal state, or the artificial limit is exceeded.
        if state.is_over() || limit.is_some_and(|d| depth >= d) || self.time_exceeded() {
            return None;
        }

        let mut move_util_term: Vec<(Move, f32, bool)> = Vec::new();
        let mut handles = Vec::new();

        for m in self.order_moves_with_tt(state, None) {
            let new_state = state
                .make_move_zobrist(&self.z_data.read().unwrap(), m)
                .unwrap();

            let parent_turn = state.current_turn();
            let run_search = move || {
                let mut table = match self.shared_t_table {
                    false => Some(FxHashMap::default()),
                    true => None,
                };
                let t = table.as_mut();
                if new_state.current_turn() == parent_turn {
                    self.max_value(
                        &new_state,
                        f32::NEG_INFINITY,
                        f32::INFINITY,
                        depth + 1,
                        limit,
                        scope,
                        t,
                    )
                } else {
                    self.min_value(
                        &new_state,
                        f32::NEG_INFINITY,
                        f32::INFINITY,
                        depth + 1,
                        limit,
                        scope,
                        t,
                    )
                }
            };

            handles.push((m, scope.spawn(run_search)));
        }

        for (m, h) in handles {
            let (utility, terminal) = match h.join().unwrap() {
                InternalResult::Node {
                    utility: v,
                    fully_searched: f,
                    ..
                } => (v, f),
                InternalResult::Timeout => return None,
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
    fn max_value<'scope, 'env>(
        &'env self,
        state: &T,
        mut alpha: f32,
        mut beta: f32,
        depth: usize,
        limit: Option<usize>,
        scope: &'scope thread::Scope<'scope, 'env>,
        mut assigned_table: Option<&mut FxHashMap<u64, TTEntry>>,
    ) -> InternalResult
    where
        T: 'env,
        'env: 'scope,
    {
        debug_assert!(
            self.start_time.read().unwrap().is_some(),
            "Minimax search must be started with `search_utility()` before calling `max_value`"
        );

        // Run the common starting procedure.
        let (early_result, alpha_orig, beta_orig, remaining) = self.max_min_preamble(
            state,
            &mut alpha,
            &mut beta,
            depth,
            limit,
            assigned_table.as_deref(),
        );
        if let Some(r) = early_result {
            return r;
        }

        let mut v = f32::NEG_INFINITY;
        let mut found_move: Option<Move> = None;
        let mut fully_searched = true;

        let (tx, rx) = mpsc::channel();
        let mut spawned = 0usize;

        for (i, m) in self
            .order_moves_with_tt(state, assigned_table.as_deref())
            .iter()
            .copied()
            .enumerate()
        {
            let new_state = state
                .make_move_zobrist(&self.z_data.read().unwrap(), m)
                .unwrap();

            // Use Young Brothers Wait Concept (YBWC) to only run the search in parallel
            // after first narrowing the bounds (i.e., after running a search on the first
            // move returned by the move orderer). We only perform root move splitting.
            if depth == 0 && i > 0 {
                let parent_turn = state.current_turn();
                let tx = tx.clone();
                let run_search = move |alpha, beta| {
                    let mut table = match self.shared_t_table {
                        false if self.use_t_table => Some(FxHashMap::default()),
                        _ => None,
                    };
                    let t = table.as_mut();
                    let result = if new_state.current_turn() == parent_turn {
                        self.max_value(&new_state, alpha, beta, depth + 1, limit, scope, t)
                    } else {
                        self.min_value(&new_state, alpha, beta, depth + 1, limit, scope, t)
                    };
                    let _ = tx.send((m, result));
                };
                scope.spawn(move || run_search(alpha, beta));
                spawned += 1;
                continue;
            }

            let (v2, local_terminal) = {
                let result = if new_state.current_turn() == state.current_turn() {
                    self.max_value(
                        &new_state,
                        alpha,
                        beta,
                        depth + 1,
                        limit,
                        scope,
                        assigned_table.as_deref_mut(),
                    )
                } else {
                    self.min_value(
                        &new_state,
                        alpha,
                        beta,
                        depth + 1,
                        limit,
                        scope,
                        assigned_table.as_deref_mut(),
                    )
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
                break;
            }
        }

        drop(tx);

        // Evaluate the results from the spawned threads. Note that we do not
        // prune, since alpha will never be greater than beta at the root.
        for _ in 0..spawned {
            let (m, result) = rx.recv().unwrap();
            let (v2, local_terminal) = match result {
                InternalResult::Node {
                    utility: v,
                    fully_searched: f,
                    ..
                } => (v, f),
                InternalResult::Timeout => continue,
            };

            if v2 > v {
                v = v2;
                found_move = Some(m);
                alpha = alpha.max(v);
            }

            fully_searched &= local_terminal;
        }

        // Store results into the transposition table, if necessary.
        self.tt_store(
            state,
            v,
            remaining,
            found_move,
            fully_searched,
            alpha_orig,
            beta_orig,
            assigned_table,
        );

        InternalResult::Node {
            found_move,
            utility: v,
            fully_searched,
        }
    }

    /// Minimize the utility / heuristic for a given state, and return the
    /// move and associated utility that do so.
    fn min_value<'scope, 'env>(
        &'env self,
        state: &T,
        mut alpha: f32,
        mut beta: f32,
        depth: usize,
        limit: Option<usize>,
        scope: &'scope thread::Scope<'scope, 'env>,
        mut assigned_table: Option<&mut FxHashMap<u64, TTEntry>>,
    ) -> InternalResult {
        debug_assert!(
            self.start_time.read().unwrap().is_some(),
            "Minimax search must be started with `search_utility()` before calling `min_value`"
        );

        // Run the common starting procedure.
        let (early_result, alpha_orig, beta_orig, remaining) = self.max_min_preamble(
            state,
            &mut alpha,
            &mut beta,
            depth,
            limit,
            assigned_table.as_deref(),
        );
        if let Some(r) = early_result {
            return r;
        }

        let mut v = f32::INFINITY;
        let mut found_move: Option<Move> = None;
        let mut fully_searched = true;

        for m in self.order_moves_with_tt(state, assigned_table.as_deref()) {
            let new_state = state
                .make_move_zobrist(&self.z_data.read().unwrap(), m)
                .unwrap();

            let (v2, local_terminal) = {
                let result = if new_state.current_turn() == state.current_turn() {
                    self.min_value(
                        &new_state,
                        alpha,
                        beta,
                        depth + 1,
                        limit,
                        scope,
                        assigned_table.as_deref_mut(),
                    )
                } else {
                    self.max_value(
                        &new_state,
                        alpha,
                        beta,
                        depth + 1,
                        limit,
                        scope,
                        assigned_table.as_deref_mut(),
                    )
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
                break;
            }
        }

        // Store results into the transposition table, if necessary.
        self.tt_store(
            state,
            v,
            remaining,
            found_move,
            fully_searched,
            alpha_orig,
            beta_orig,
            assigned_table,
        );

        InternalResult::Node {
            found_move,
            utility: v,
            fully_searched,
        }
    }

    /// Helper function that performs the following actions at the beginning
    /// of either [`max_value`] or [`min_value`]:
    /// - Check if the state is a terminal state.
    /// - Check if a valid result is in the transposition table.
    /// - Modify the search bounds based on the transposition table, if necessary.
    /// - Check if the depth limit has been reached.
    /// - Check if the time limit has been exceeded.
    fn max_min_preamble(
        &self,
        state: &T,
        alpha: &mut f32,
        beta: &mut f32,
        depth: usize,
        limit: Option<usize>,
        assigned_table: Option<&FxHashMap<u64, TTEntry>>,
    ) -> (Option<InternalResult>, f32, f32, usize) {
        // Keep track of the original values for alpha, beta, and the remaining depth.
        let alpha_orig = *alpha;
        let beta_orig = *beta;
        let remaining = limit.map(|l| l.saturating_sub(depth)).unwrap_or(usize::MAX);

        // If we are in a terminal state, evaluate utility.
        if state.is_over() {
            let r = InternalResult::Node {
                found_move: None,
                utility: self.evaluate(state),
                fully_searched: true,
            };
            return (Some(r), alpha_orig, beta_orig, remaining);
        }

        // Check transposition table, and narrow bounds if necessary.
        if self.use_t_table {
            if let Some(r) = self.tt_probe(state, remaining, alpha, beta, assigned_table) {
                return (Some(r), alpha_orig, beta_orig, remaining);
            }
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
        assigned_table: Option<&FxHashMap<u64, TTEntry>>,
    ) -> Option<InternalResult> {
        if !self.use_t_table {
            return None;
        }

        match assigned_table {
            Some(table) => table
                .get(&state.zobrist_hash())
                .and_then(|e| e.probe(remaining, alpha, beta)),
            None => self
                .t_table
                .get(state.zobrist_hash())
                .and_then(|e| e.probe(remaining, alpha, beta)),
        }
    }

    /// Helper function to store an evaluated state in the transposition table.
    fn tt_store(
        &self,
        state: &T,
        value: f32,
        remaining: usize,
        found_move: Option<Move>,
        fully_searched: bool,
        alpha_orig: f32,
        beta_orig: f32,
        assigned_table: Option<&mut FxHashMap<u64, TTEntry>>,
    ) {
        if !self.use_t_table {
            return;
        }

        let key = state.zobrist_hash();
        let entry = TTEntry::new(
            value,
            remaining,
            found_move,
            fully_searched,
            alpha_orig,
            beta_orig,
        );

        match assigned_table {
            Some(table) => {
                if table.get(&key).is_none_or(|old| {
                    (entry.remaining >= old.remaining)
                        || (entry.fully_searched && !old.fully_searched)
                }) {
                    table.insert(key, entry);
                }
            }
            None => {
                if self.t_table.get(key).is_none_or(|old| {
                    (entry.remaining >= old.remaining)
                        || (entry.fully_searched && !old.fully_searched)
                }) {
                    self.t_table.insert(key, entry);
                }
            }
        }
    }

    /// Helper function to get the move stored for the current state's
    /// transposition table entry, if one exists.
    fn get_tt_move(
        &self,
        state: &T,
        assigned_table: Option<&FxHashMap<u64, TTEntry>>,
    ) -> Option<Move> {
        match assigned_table {
            Some(table) => table.get(&state.zobrist_hash()).and_then(|e| e.found_move),
            None => self
                .t_table
                .get(state.zobrist_hash())
                .and_then(|e| e.found_move),
        }
    }

    /// Helper function to order the moves during minimax search, ensuring
    /// that the transposition entry is tried first, if it exists.
    fn order_moves_with_tt(
        &self,
        state: &T,
        assigned_table: Option<&FxHashMap<u64, TTEntry>>,
    ) -> Vec<Move> {
        let mut moves = self.order_moves(state);

        if !self.use_t_table {
            return moves;
        }

        if let Some(tt_move) = self.get_tt_move(state, assigned_table) {
            if let Some(pos) = moves.iter().position(|m| *m == tt_move) {
                let m = moves.remove(pos);
                moves.insert(0, m);
            }
        }
        moves
    }
}
