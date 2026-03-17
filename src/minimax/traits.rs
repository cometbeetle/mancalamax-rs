//! Traits used for the minimax algorithm implementation.

use crate::game::{Mancala, Player};

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
/// transposition table system. The default implementation will not
/// enable any Zobrist hashing, and can be safely added to new data
/// structures that implement the [`Mancala`] trait.
pub trait ZobristHash {
    /// Returns whether Zobrist hashing is enabled. Simply checks whether
    /// [`get_zobrist_hash`] returns [`Some`].
    fn zobrist_enabled(&self) -> bool {
        self.get_zobrist_hash().is_some()
    }

    /// Returns the Zobrist value for a given action index.
    /// Returns [`None`] if unimplemented.
    #[allow(unused_variables)]
    fn get_zobrist_val(&self, idx: ZobristIdx) -> Option<u64> {
        None
    }

    /// Returns the current Zobrist hash of the implementing data structure instance,
    /// if one exists. The default implementation simply returns [`None`]
    /// and should be overridden in an actual implementation.
    fn get_zobrist_hash(&self) -> Option<u64> {
        None
    }

    /// Sets the Zobrist hash of the implement data structure instance to a
    /// particular value. Acts as a no-op if unimplemented.
    #[allow(unused_variables)]
    fn set_zobrist_hash(&mut self, hash: u64) -> Result<(), ()> {
        Err(())
    }

    /// Performs a Zobrist hash update using XOR. The default implementation
    //  is sufficient, and does not need to be overridden.
    fn update_zobrist_hash(&mut self, idx: ZobristIdx) -> Result<(), ()> {
        let existing_hash = match self.get_zobrist_hash() {
            Some(hash) => hash,
            None => return Err(()),
        };

        let zobrist_val = match self.get_zobrist_val(idx) {
            Some(val) => val,
            None => return Err(()),
        };

        self.set_zobrist_hash(existing_hash ^ zobrist_val)
    }
}
