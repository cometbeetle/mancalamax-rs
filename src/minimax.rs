//! Components relating to the use of the minimax algorithm with Mancala
//! board states.

pub mod algorithm;
pub mod builder;

pub use algorithm::Minimax;
pub use builder::MinimaxBuilder;

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
