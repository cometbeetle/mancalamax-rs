//! Utility components used by Mancala game state structs.

use super::mancala::{Mancala, Move, Player};
use std::fmt::Formatter;

/// Format a struct implementing the [`Mancala`] trait as necessary for
/// the [`Display`] trait. Assists with printing to standard output without
/// excessive code duplication for both [`GameState`] and [`DynGameState`].
pub(super) fn fmt_common(f: &mut Formatter, state: &impl Mancala, title: &str) -> std::fmt::Result {
    let header = format!("Bird's-Eye View of {} {:p}", title, state);
    writeln!(f, "{}", header)?;
    writeln!(f, "{}", "=".repeat(header.len()))?;

    let p1_select = match state.current_turn() {
        Player::One => '*',
        Player::Two => ' ',
    };
    write!(f, "{} P1:  ({:02})  [ ", p1_select, state.stores()[0])?;
    for pit in state.board()[0].as_ref().iter().rev() {
        write!(f, "{:02} ", pit)?;
    }
    writeln!(f, "]")?;

    let p2_select = match state.current_turn() {
        Player::One => ' ',
        Player::Two => '*',
    };
    write!(f, "{} P2:        [ ", p2_select)?;
    for pit in state.board()[1].as_ref() {
        write!(f, "{:02} ", pit)?;
    }
    writeln!(f, "]  ({:02})", state.stores()[1])?;

    writeln!(f, "Move Number: {}", state.ply())?;

    write!(f, "Valid Moves: ")?;
    let mut valid_moves = state.valid_moves();
    let mut valid_str = String::new();
    valid_moves.sort();
    for m in valid_moves {
        match m {
            Move::Pit(n) => valid_str += format!("{}, ", n).as_str(),
            Move::Swap => valid_str += "SWAP, ",
        }
    }
    if !valid_str.is_empty() {
        valid_str.truncate(valid_str.len() - 2);
    } else {
        valid_str = "None".to_string();
    }
    writeln!(f, "{}", valid_str)?;
    Ok(())
}

/// Helper macro for implementing the transposition table hash function.
macro_rules! tt_hash_common {
    () => {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.0.board.hash(state);
            self.0.stores.hash(state);
            self.0.current_turn.hash(state);
            self.0.p2_moved.hash(state);
        }
    };
}

/// Helper macro for implementing the transition table equality operator.
///
/// Used only in [`GameState`] and [`DynGameState`] structs.
macro_rules! tt_eq_common {
    () => {
        fn eq(&self, other: &Self) -> bool {
            self.0.board == other.0.board
                && self.0.stores == other.0.stores
                && self.0.current_turn == other.0.current_turn
                && self.0.p2_moved == other.0.p2_moved
        }
    };
}

pub(crate) use {tt_eq_common, tt_hash_common};
