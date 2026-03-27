//! Builder utilities for constructing [`Minimax`] instances.

use super::Minimax;
use super::{MancalaZobrist, MoveOrderFn, StateEvalFn};
use crate::game::{Move, Player};
use rustc_hash::FxHashMap;
use std::time::Duration;

/// Helper for constructing [`Minimax`] instances based on certain specifications.
#[derive(Debug, Clone, Copy)]
pub struct MinimaxBuilder<T: MancalaZobrist> {
    optimize_for: Player,
    max_depth: Option<usize>,
    max_time: Option<Duration>,
    iterative_deepening: bool,
    use_t_table: bool,
    move_orderer: MoveOrderFn<T>,
    evaluator: StateEvalFn<T>,
    heuristic: StateEvalFn<T>,
    t_table_capacity: usize,
}

impl<T: MancalaZobrist> Default for MinimaxBuilder<T> {
    /// The default [`Minimax`] configuration is the following:
    /// - `optimize_for`: [`Player::One`]
    /// - `max_depth`: `18`
    /// - `max_time`: [`None`]
    /// - `iterative_deepening`: [`true`]
    /// - `use_t_table`: [`true`]
    /// - `move_orderer`: A function that returns the valid moves in descending order by pit number.
    /// - `evaluator`: A function that returns the point differential between
    ///   the players (positive if the current player is winning).
    /// - `heuristic`: Same as evaluator.
    /// - `t_table_capacity`: `0`
    fn default() -> Self {
        // Faster than sorting s.valid_moves() at each iteration.
        let move_orderer = |s: &T| {
            let mut moves = Vec::new();
            if s.swap_allowed() {
                moves.insert(0, Move::Swap);
            }
            for (i, pit) in s.board()[s.current_turn()].as_ref().iter().enumerate() {
                if *pit > 0 {
                    moves.insert(0, Move::Pit(i + 1));
                }
            }
            moves
        };
        let evaluator = |s: &T, p: Player| match p {
            Player::One => (s.score(Player::One) as isize - s.score(Player::Two) as isize) as f32,
            Player::Two => (s.score(Player::Two) as isize - s.score(Player::One) as isize) as f32,
        };
        let heuristic = evaluator;
        Self {
            optimize_for: Player::One,
            max_depth: Some(18),
            max_time: None,
            iterative_deepening: true,
            use_t_table: true,
            move_orderer,
            evaluator,
            heuristic,
            t_table_capacity: 0,
        }
    }
}

impl<T: MancalaZobrist> From<Minimax<T>> for MinimaxBuilder<T> {
    fn from(value: Minimax<T>) -> Self {
        from_common(&value)
    }
}

impl<T: MancalaZobrist> From<&Minimax<T>> for MinimaxBuilder<T> {
    fn from(value: &Minimax<T>) -> Self {
        from_common(value)
    }
}

impl<T: MancalaZobrist> MinimaxBuilder<T> {
    /// Construct a new [`MinimaxBuilder`] instance using the default configuration.
    ///
    /// See [`MinimaxBuilder::default`] for details.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the player for which minimax should optimize the outcome.
    pub fn optimize_for(mut self, p: Player) -> Self {
        self.optimize_for = p;
        self
    }

    /// Set the maximum search depth.
    ///
    /// [`None`] means no search depth limit.
    ///
    /// <div class="warning">
    ///
    /// If [`None`] is selected, and [`max_time`][`Self::max_time`] is also set to
    /// [`None`], the algorithm will not terminate unless all states have been searched,
    /// which may take an intractable amount of time.
    ///
    /// </div>
    pub fn max_depth(mut self, d: Option<usize>) -> Self {
        self.max_depth = d;
        self
    }

    /// Set the maximum time allowed to find a move.
    ///
    /// [`None`] means no time limit.
    ///
    /// <div class="warning">
    ///
    /// If [`None`] is selected, and [`max_depth`][Self::max_depth] is also set to
    /// [`None`], the algorithm will not terminate unless all states have been searched,
    /// which may take an intractable amount of time.
    ///
    /// </div>
    pub fn max_time(mut self, t: Option<Duration>) -> Self {
        self.max_time = t;
        self
    }

    /// Set whether iterative deepening should be used.
    ///
    /// Enables returning the deepest search result from minimax if
    /// time expires before the depth limit is reached.
    pub fn iterative_deepening(mut self, enabled: bool) -> Self {
        self.iterative_deepening = enabled;
        self
    }

    /// Set whether to use a transposition table during search.
    pub fn use_t_table(mut self, enabled: bool) -> Self {
        self.use_t_table = enabled;
        self
    }

    /// Set the move ordering function.
    ///
    /// This function is used for each state checked by minimax, and
    /// should be designed to supply moves in an optimal order (i.e., one
    /// which avoids excessive future computation).
    pub fn move_orderer(mut self, o: MoveOrderFn<T>) -> Self {
        self.move_orderer = o;
        self
    }

    /// Set the evaluator function.
    ///
    /// This function is used to evaluate states only when it is a terminal state
    /// (i.e., when the game is over).
    pub fn evaluator(mut self, e: StateEvalFn<T>) -> Self {
        self.evaluator = e;
        self
    }

    /// Set the heuristic function.
    ///
    /// This function is used to evaluate states only when the artificial limit
    /// (i.e., the time / depth limit) has been reached, and may be different
    /// from the evaluator function.
    pub fn heuristic(mut self, h: StateEvalFn<T>) -> Self {
        self.heuristic = h;
        self
    }

    /// Set the initial transposition table capacity.
    pub fn t_table_capacity(mut self, c: usize) -> Self {
        self.t_table_capacity = c;
        self
    }

    /// Construct a [`Minimax`] instance based on the set configuration.
    pub fn build(&self) -> Minimax<T> {
        // Initially size the transposition table.
        let mut t_table = FxHashMap::default();
        if self.use_t_table {
            t_table.reserve(self.t_table_capacity);
        }

        Minimax {
            optimize_for: self.optimize_for,
            max_depth: self.max_depth,
            max_time: self.max_time,
            iterative_deepening: self.iterative_deepening,
            use_t_table: self.use_t_table,
            move_orderer: self.move_orderer,
            evaluator: self.evaluator,
            heuristic: self.heuristic,
            start_time: None.into(),
            t_table: t_table.into(),
            z_data: Default::default(),
        }
    }
}

/// Helper function for implementing the [`From`] trait.
fn from_common<T: MancalaZobrist>(value: &Minimax<T>) -> MinimaxBuilder<T> {
    MinimaxBuilder {
        optimize_for: value.optimize_for,
        max_depth: value.max_depth,
        max_time: value.max_time,
        iterative_deepening: value.iterative_deepening,
        use_t_table: value.use_t_table,
        move_orderer: value.move_orderer,
        evaluator: value.evaluator,
        heuristic: value.heuristic,
        t_table_capacity: value.t_table.borrow().capacity(),
    }
}
