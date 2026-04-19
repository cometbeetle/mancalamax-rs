//! Components relating to the use of the minimax algorithm with Mancala
//! board states.

pub mod algorithm;
pub mod builder;
pub mod parallel;
mod table;
pub mod zobrist;

pub use algorithm::Minimax;
pub use builder::{MinimaxBuilder, ParMinimaxBuilder};
pub use parallel::ParMinimax;
pub use zobrist::{MancalaZobrist, ZobristAction, ZobristData};

use crate::game::{Move, Player};

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

/// Stores the value of a minimax search result.
///
/// If the [`fully_searched`][Self::fully_searched] field is [`true`], then the heuristic
/// was never used in finding the current result (i.e., the search evaluated all
/// possible terminal states).
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SearchResult {
    pub found_move: Move,
    pub utility: f32,
    pub depth_searched: Option<usize>,
    pub fully_searched: bool,
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
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MultiSearchResult {
    pub found_moves: Vec<Move>,
    pub utilities: Vec<f32>,
    pub depth_searched: Option<usize>,
    pub fully_searched: bool,
}

/// Helper enum to store the internal results of minimax searches.
#[derive(Debug, Clone, Copy, PartialEq)]
enum InternalResult {
    Node {
        found_move: Option<Move>,
        utility: f32,
        fully_searched: bool,
    },
    Timeout,
}
