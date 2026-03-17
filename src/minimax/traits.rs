//! Traits used for the minimax algorithm implementation.

use crate::game::{Mancala, Move, Player};

/// Enum used to represent an action that should be recorded by the
/// Zobrist hashing system. Each instance serves as an index into the
/// available Zobrist value table.
#[derive(Debug, Clone, Copy)]
pub enum ZobristIdx {
    Pit(Player, usize, usize),
    Store(Player, usize),
    SwitchTurn,
    P2Moved,
}

/// Trait used to implement Zobrist hashing for use with the minimax
/// transposition table system. Designed to be implemented on structs
/// that also implement [`Mancala`].
pub trait ZobristHash: Mancala {
    /// Makes a move using the underlying [`Mancala::make_move`] logic while
    /// simultaneously updating the Zobrist hash of the implementing object.
    fn make_move_zobrist(&self, selection: Move) -> Result<Self, ()> {
        let mut new_state = self.make_move(selection)?;
        perform_updates(self, &mut new_state);
        Ok(new_state)
    }

    /// Makes a random move using the underlying [`Mancala::make_move_rand`] logic
    /// while simultaneously updating the Zobrist hash of the implementing object.
    fn make_move_rand_zobrist(&self) -> Result<(Self, Move), ()> {
        let (mut new_state, m) = self.make_move_rand()?;
        perform_updates(self, &mut new_state);
        Ok((new_state, m))
    }

    /// Performs a Zobrist hash update using XOR. The default implementation
    //  is sufficient, and does not need to be overridden.
    fn update_zobrist_hash(&mut self, old: ZobristIdx, new: ZobristIdx) {
        let existing_hash = self.get_zobrist_hash();
        let old_zobrist_val = self.get_zobrist_val(old);
        let new_zobrist_val = self.get_zobrist_val(new);
        self.set_zobrist_hash(existing_hash ^ old_zobrist_val ^ new_zobrist_val)
    }

    fn update_zobrist_hash_partial(&mut self, idx: ZobristIdx) {
        let existing_hash = self.get_zobrist_hash();
        let zobrist_val = self.get_zobrist_val(idx);
        self.set_zobrist_hash(existing_hash ^ zobrist_val)
    }

    /// Returns the Zobrist value for a given action index.
    fn get_zobrist_val(&self, idx: ZobristIdx) -> u64;

    /// Returns the current Zobrist hash of the implementing data structure instance.
    fn get_zobrist_hash(&self) -> u64;

    /// Sets the Zobrist hash of the implement data structure instance to a
    /// particular value.
    fn set_zobrist_hash(&mut self, hash: u64);
}

/// Helper function to compare a new state with an old state, and update
/// its Zobrist hash value accordingly.
fn perform_updates<T: Mancala + ZobristHash>(old_state: &T, new_state: &mut T) {
    for player in [Player::One, Player::Two] {
        for pit in 0..old_state.pits() {
            let old_count = old_state.board()[player].as_ref()[pit];
            let new_count = new_state.board()[player].as_ref()[pit];
            if old_count == new_count {
                continue;
            }
            let old = ZobristIdx::Pit(player, pit, old_count);
            let new = ZobristIdx::Pit(player, pit, new_count);
            match new_state.update_zobrist_hash(old, new) {
                _ => {}
            }
        }
        let old_score = old_state.score(player);
        let new_score = new_state.score(player);
        if old_score == new_score {
            continue;
        }
        let old = ZobristIdx::Store(player, old_state.score(player));
        let new = ZobristIdx::Store(player, new_state.score(player));
        match new_state.update_zobrist_hash(old, new) {
            _ => {}
        }
    }
    if new_state.current_turn() != old_state.current_turn() {
        match new_state.update_zobrist_hash_partial(ZobristIdx::SwitchTurn) {
            _ => {}
        }
    }
    if new_state.p2_moved() != old_state.p2_moved() {
        match new_state.update_zobrist_hash_partial(ZobristIdx::P2Moved) {
            _ => {}
        }
    }
}
