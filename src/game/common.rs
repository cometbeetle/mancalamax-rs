//! Utility components used by Mancala game state structs.

use super::mancala::{Mancala, Player};
use std::fmt::Formatter;

/// Format a struct implementing the `Mancala` trait as necessary for
/// the `Display` trait. Assists with printing to standard output without
/// excessive code duplication for both `GameState` and `DynGameState`.
pub(super) fn fmt_common<T>(f: &mut Formatter, state: &T, title: &str) -> std::fmt::Result
where
    T: Mancala,
{
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
    Ok(())
}
