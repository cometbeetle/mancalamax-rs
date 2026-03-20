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

/// Helper macro for implementing the Zobrist hashing system in both
/// [`GameState`] and [`DynGameState`].
macro_rules! zobrist_val_impl {
    ($self:ident, $pits:expr, $idx:expr) => {
        const SEED: u64 = 0x49CB86856BB06133;
        static ZOBRIST_PITS: OnceLock<Vec<u64>> = OnceLock::new();
        static ZOBRIST_STORES: OnceLock<Vec<u64>> = OnceLock::new();
        static ZOBRIST_SWITCH_TURN: OnceLock<u64> = OnceLock::new();
        static ZOBRIST_P2_MOVED: OnceLock<u64> = OnceLock::new();
        static TOTAL_STONES: OnceLock<usize> = OnceLock::new();

        let total_stones = TOTAL_STONES.get_or_init(|| {
            $self.board.iter().flatten().sum::<usize>() + $self.stores.iter().sum::<usize>()
        });

        let zobrist_pits = ZOBRIST_PITS.get_or_init(|| {
            let mut rng = StdRng::seed_from_u64(SEED);
            let total_entries = 2 * $pits * total_stones;
            (0..total_entries).map(|_| rng.next_u64()).collect()
        });

        let zobrist_stores = ZOBRIST_STORES.get_or_init(|| {
            let mut rng = StdRng::seed_from_u64(SEED ^ 1);
            let total_entries = 2 * total_stones;
            (0..total_entries).map(|_| rng.next_u64()).collect()
        });

        let zobrist_switch_turn = *ZOBRIST_SWITCH_TURN.get_or_init(|| {
            let mut rng = StdRng::seed_from_u64(SEED ^ 2);
            rng.next_u64()
        });

        let zobrist_p2_moved = *ZOBRIST_P2_MOVED.get_or_init(|| {
            let mut rng = StdRng::seed_from_u64(SEED ^ 3);
            rng.next_u64()
        });

        return match $idx {
            ZobristIdx::Pit(player, pit, stones) => {
                let player = usize::from(player) - 1;
                let index = player * ($pits * total_stones) + pit * total_stones + stones;
                zobrist_pits[index]
            }
            ZobristIdx::Store(player, stones) => {
                let player = usize::from(player) - 1;
                let index = player * total_stones + stones;
                zobrist_stores[index]
            }
            ZobristIdx::SwitchTurn => zobrist_switch_turn,
            ZobristIdx::P2Moved => zobrist_p2_moved,
        }
    };
}

pub(super) use zobrist_val_impl;
