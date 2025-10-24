//! Builder utilities for constructing [`Minimax`] instances.

use super::Minimax;
use super::{MoveOrderFn, StateEvalFn};
use crate::game::{Mancala, Move, Player};
use std::time::Duration;

/// Helper for constructing [`Minimax`] instances based on certain specifications.
#[derive(Debug, Clone, Copy)]
pub struct MinimaxBuilder<T: Mancala> {
    optimize_for: Player,
    max_depth: Option<usize>,
    max_time: Option<Duration>,
    move_orderer: MoveOrderFn<T>,
    evaluator: StateEvalFn<T>,
    heuristic: StateEvalFn<T>,
}

impl<T: Mancala> MinimaxBuilder<T> {
    /// Construct a new [`MinimaxBuilder`] instance using the default configuration.
    ///
    /// See [`MinimaxBuilder::default`] for details.
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
    /// If None is selected, and max_time is also set to
    /// None, the algorithm will not terminate unless all states have been searched,
    /// which may take an intractable amount of time.
    /// </div>
    pub fn max_depth(mut self, depth: Option<usize>) -> Self {
        self.max_depth = depth;
        self
    }

    /// Set the maximum time allowed to find a move.
    ///
    /// [`None`] means no time limit.
    ///
    /// <div class="warning">
    /// If None is selected, and max_depth is also set to
    /// None, the algorithm will not terminate unless all states have been searched,
    /// which may take an intractable amount of time.
    /// </div>
    pub fn max_time(mut self, time: Option<Duration>) -> Self {
        self.max_time = time;
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
    /// - `max_time`: [`None`]
    /// - `move_orderer`: A function that returns the valid moves in descending order by pit number.
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
            for (i, pit) in s.board()[s.current_turn()].as_ref().iter().enumerate() {
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
            move_orderer,
            evaluator,
            heuristic,
        }
    }
}
