mod common;
pub mod dyn_game_state;
pub mod game_state;
pub mod mancala;

pub use dyn_game_state::DynGameState;
pub use game_state::GameState;
pub use mancala::{GameOutcome, Mancala, Move, Player};
