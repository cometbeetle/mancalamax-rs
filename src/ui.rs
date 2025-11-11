//! Components of the user interface system.

#[cfg(feature = "gui")]
pub mod gui;
pub mod terminal;

pub use terminal::{
    ExternalInterface, minimax_v_external, minimax_v_external_default, player_v_external,
    player_v_external_default, player_v_minimax, player_v_minimax_default, player_v_player,
    player_v_player_default,
};
