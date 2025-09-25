//! Components relating to the use of the minimax algorithm with Mancala
//! board states.

use crate::game::{Mancala, Move, Player};
use std::cell::Cell;
use std::collections::HashSet;
use std::time::{Duration, Instant};

/// Type alias for any function that evaluates a reference to a type
/// (usually some kind of Mancala game state) and a current player,
/// and produces a [`f32`] value indicating some level of utility.
/// Positive values indicate higher utility.
pub type StateEvalFn<T> = fn(&T, player: Player) -> f32;

/// Type alias for any function that evaluates a reference to a type
/// (usually some kind of Mancala game state) and produces a vector
/// of moves in a specific order. Every move in the vector should
/// be a valid move, given the supplied game state reference.
pub type MoveOrderFn<T> = fn(&T) -> Vec<Move>;

/// Stores the necessary information for executing the minimax algorithm on a
/// Mancala board state in order to determine the most optimal move (i.e.,
/// the one that maximizes utility, or is calculated as best based on some heuristic).
#[derive(Debug, Clone)]
pub struct Minimax<T: Mancala> {
    optimize_for: Player,
    max_depth: Option<usize>,
    max_time: Option<Duration>,
    iterative_deepening: bool,
    move_orderer: MoveOrderFn<T>,
    evaluator: StateEvalFn<T>,
    heuristic: StateEvalFn<T>,
    start_time: Cell<Option<Instant>>,
}

/// Helper for constructing [Minimax] instances based on certain specifications.
#[derive(Debug, Clone, Copy)]
pub struct MinimaxBuilder<T: Mancala> {
    optimize_for: Player,
    max_depth: Option<usize>,
    max_time: Option<Duration>,
    iterative_deepening: bool,
    move_orderer: MoveOrderFn<T>,
    evaluator: StateEvalFn<T>,
    heuristic: StateEvalFn<T>,
}

impl<T: Mancala> MinimaxBuilder<T> {
    /// Construct a new [MinimaxBuilder] instance using the default configuration.
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
    /// WARNING: If [`None`] is selected, and [`max_time`](Self::max_time) is also set to
    /// [`None`], the algorithm will not terminate unless all states have been searched,
    /// which may take time exponential in the number of remaining move combinations.
    pub fn max_depth(mut self, depth: Option<usize>) -> Self {
        self.max_depth = depth;
        self
    }

    /// Set the maximum time allowed to find a move.
    ///
    /// [`None`] means no time limit.
    ///
    /// WARNING: If [`None`] is selected, and [`max_depth`](Self::max_depth) is also set to
    /// [`None`], the algorithm will not terminate unless all states have been searched,
    /// which may take time exponential in the number of remaining move combinations.
    pub fn max_time(mut self, time: Duration) -> Self {
        self.max_time = Some(time);
        self
    }

    /// Set whether to use iterative deepening as a search strategy.
    pub fn iterative_deepening(mut self, iterative: bool) -> Self {
        self.iterative_deepening = iterative;
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

    /// Construct a [`Minimax`] instance based on the set configuration.
    pub fn build(&self) -> Minimax<T> {
        Minimax {
            optimize_for: self.optimize_for,
            max_depth: self.max_depth,
            max_time: self.max_time,
            iterative_deepening: self.iterative_deepening,
            move_orderer: self.move_orderer,
            evaluator: self.evaluator,
            heuristic: self.heuristic,
            start_time: None.into(),
        }
    }
}

impl<T: Mancala> Default for MinimaxBuilder<T> {
    /// The default [`Minimax`] configuration is the following:
    /// - `optimize_for`: [`Player::One`]
    /// - `max_depth`: `12`
    /// - `iterative_deepening`: [`false`]
    /// - `move_orderer`: Returns the valid moves in descending order by pit number.
    /// - `evaluator`: A function that returns the point differential between
    ///   the players (positive if the current player is winning).
    /// - `heuristic` Same as evaluator.
    fn default() -> Self {
        // Faster than sorting s.valid_moves() at each iteration.
        let move_orderer = |s: &T| {
            let mut moves = Vec::new();
            if s.swap_allowed() {
                moves.insert(0, Move::Swap);
            }
            for (i, pit) in s.board()[s.current_turn()]
              .as_ref()
              .iter()
              .enumerate()
            {
                if *pit > 0 {
                    moves.insert(0, Move::Pit(i + 1));
                }
            }
            moves
        };
        let evaluator = |s: &T, p: Player| match p {
            Player::One => (s.score(Player::One) - s.score(Player::Two)) as f32,
            Player::Two => (s.score(Player::Two) - s.score(Player::One)) as f32,
        };
        let heuristic = evaluator;
        Self {
            optimize_for: Player::One,
            max_depth: Some(12),
            max_time: None,
            iterative_deepening: false,
            move_orderer,
            evaluator,
            heuristic,
        }
    }
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

    /// Returns whether iterative deepening is being used.
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
    pub fn search(&self, state: &T) -> Option<Move> {
        self.start_time.set(Some(Instant::now()));

        // TODO: Implement iterative deepening.

        let (best_move, _) = self.max_value(state, f32::NEG_INFINITY, f32::INFINITY, 0);

        self.start_time.set(None);
        best_move
    }

    /// Determines whether the algorithm has been running longer than requested.
    ///
    /// Used internally inside `max_value` and `min_value`.
    fn time_exceeded(&self) -> bool {
        match (self.start_time(), self.max_time) {
            (Some(start), Some(max)) => Instant::now() - start >= max,
            _ => false,
        }
    }

    /// Maximize the utility / heuristic for a given state, and return the
    /// (move, utility) pair that does so.
    fn max_value(&self, state: &T, alpha: f32, beta: f32, depth: usize) -> (Option<Move>, f32) {
        assert_ne!(
            self.start_time.get(),
            None,
            "Minimax search must be started with `search()` before calling `min_value`"
        );

        // If we are in a terminal state, evaluate utility.
        if state.is_over() {
            return (None, self.evaluate(state));
        }

        // If we have reached the artificial limit, use the heuristic.
        if self.max_depth.is_some_and(|d| depth >= d) || self.time_exceeded() {
            return (None, self.get_heuristic(state));
        }

        let depth = depth + 1;
        let mut alpha = alpha;
        let mut v = f32::NEG_INFINITY;
        let mut best_move: Option<Move> = None;

        for m in self.order_moves(state) {
            let new_state = state.make_move(m).unwrap();
            let (_, v2) = if new_state.current_turn() == state.current_turn() {
                self.max_value(&new_state, alpha, beta, depth)
            } else {
                self.min_value(&new_state, alpha, beta, depth)
            };

            if v2 > v {
                v = v2;
                best_move = Some(m);
                alpha = if alpha > v { alpha } else { v };
            }

            // Alpha > beta ==> prune
            if v >= beta {
                return (best_move, v);
            }
        }

        (best_move, v)
    }

    /// Minimize the utility / heuristic for a given state, and return the
    /// (move, utility) pair that does so.
    fn min_value(&self, state: &T, alpha: f32, beta: f32, depth: usize) -> (Option<Move>, f32) {
        assert_ne!(
            self.start_time(),
            None,
            "Minimax search must be started with `search()` before calling `min_value`"
        );

        // If we are in a terminal state, evaluate utility.
        if state.is_over() {
            return (None, self.evaluate(state));
        }

        // If we have reached the artificial limit, use the heuristic.
        if self.max_depth.is_some_and(|d| depth >= d) || self.time_exceeded() {
            return (None, self.get_heuristic(state));
        }

        let depth = depth + 1;
        let mut beta = beta;
        let mut v = f32::INFINITY;
        let mut best_move: Option<Move> = None;

        for m in self.order_moves(state) {
            let new_state = state.make_move(m).unwrap();
            let (_, v2) = if new_state.current_turn() == state.current_turn() {
                self.min_value(&new_state, alpha, beta, depth)
            } else {
                self.max_value(&new_state, alpha, beta, depth)
            };

            if v2 < v {
                v = v2;
                best_move = Some(m);
                beta = if beta < v { beta } else { v };
            }

            // Alpha > beta ==> prune
            if v <= alpha {
                return (best_move, v);
            }
        }

        (best_move, v)
    }
}
