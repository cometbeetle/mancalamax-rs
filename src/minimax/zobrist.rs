//! Components used for Zobrist hashing during minimax.

use crate::game::{Mancala, Move, Player};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

/// Enum used to represent an action that should be recorded by the
/// Zobrist hashing system.
#[derive(Debug, Clone, Copy)]
pub enum ZobristAction {
    Pit(Player, usize, usize),
    Store(Player, usize),
    SwitchTurn,
    P2Moved,
}

/// Struct used to store appropriately sized tables of Zobrist values
/// which can be used to update the Zobrist hash of a game state.
#[derive(Debug, Clone)]
pub struct ZobristData {
    pit_vals: Vec<u64>,
    store_vals: Vec<u64>,
    switch_turn_val: u64,
    p2_moved_val: u64,
}

impl Default for ZobristData {
    /// The default value for [`ZobristData`] instances is normally not
    /// useful, since the internal vectors which store the Zobrist values
    /// are initialized as empty.
    ///
    /// To create an instance with useful data, call [`for_state_like`][Self::for_state_like].
    fn default() -> Self {
        let mut rng = rand::rng();
        Self {
            pit_vals: vec![],
            store_vals: vec![],
            switch_turn_val: rng.next_u64(),
            p2_moved_val: rng.next_u64(),
        }
    }
}

impl ZobristData {
    /// Returns the total number of stones assumed to be present for any
    /// game state that makes use of the current data.
    #[inline]
    pub fn total_stones(&self) -> usize {
        self.store_vals.len() / 2
    }

    /// Returns the number of pits assumed to be present for any
    /// game state that makes use of the current data.
    #[inline]
    pub fn num_pits(&self) -> usize {
        self.pit_vals.len() / self.total_stones() / 2
    }

    /// Create a new set of Zobrist values for use with game states "like" the
    /// supplied state. Here, "like" means the state must have the same
    /// number of total stones and the same number of pits.
    pub fn for_state_like(state: &impl Mancala, seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);

        let pits: Vec<u64> = {
            let total_entries = 2 * state.pits() * state.total_stones();
            (0..total_entries).map(|_| rng.next_u64()).collect()
        };

        let stores: Vec<u64> = {
            let total_entries = 2 * state.total_stones();
            (0..total_entries).map(|_| rng.next_u64()).collect()
        };

        Self {
            pit_vals: pits.into(),
            store_vals: stores.into(),
            switch_turn_val: rng.next_u64(),
            p2_moved_val: rng.next_u64(),
        }
    }

    /// Returns a boolean indicating whether the supplied game state
    /// can be used with the current Zobrist data.
    pub fn is_valid_for(&self, state: &impl Mancala) -> bool {
        self.total_stones() == state.total_stones() && self.num_pits() == state.pits()
    }

    /// Gets the Zobrist value for a supplied state and Zobrist action.
    ///
    /// Assumes the data is valid for the supplied state, and only panics if
    /// an out-of-bounds vector access occurs.
    pub fn get_val(&self, state: &impl Mancala, action: ZobristAction) -> u64 {
        match action {
            ZobristAction::Pit(player, pit, stones) => {
                let player = usize::from(player) - 1;
                let index = player * (state.pits() * self.total_stones())
                    + pit * self.total_stones()
                    + stones;
                self.pit_vals[index]
            }
            ZobristAction::Store(player, stones) => {
                let player = usize::from(player) - 1;
                let index = player * self.total_stones() + stones;
                self.store_vals[index]
            }
            ZobristAction::SwitchTurn => self.switch_turn_val,
            ZobristAction::P2Moved => self.p2_moved_val,
        }
    }
}

/// Trait used to implement Zobrist hashing for use with the minimax
/// transposition table system. Must be implemented on structs
/// that also implement [`Mancala`].
pub trait MancalaZobrist: Mancala {
    /// Makes a move using the underlying [`Mancala::make_move`] logic while
    /// simultaneously updating the Zobrist hash of the implementing object.
    fn make_move_zobrist(&self, data: &ZobristData, selection: Move) -> Result<Self, ()> {
        let mut new_state = self.make_move(selection)?;
        perform_updates(data, self, &mut new_state);
        Ok(new_state)
    }

    /// Makes a random move using the underlying [`Mancala::make_move_rand`] logic
    /// while simultaneously updating the Zobrist hash of the implementing object.
    fn make_move_rand_zobrist(&self, data: &ZobristData) -> Result<(Self, Move), ()> {
        let (mut new_state, m) = self.make_move_rand()?;
        perform_updates(data, self, &mut new_state);
        Ok((new_state, m))
    }

    /// Performs a Zobrist hash update based on an old action and a new action.
    fn update_zobrist_hash(&mut self, data: &ZobristData, old: ZobristAction, new: ZobristAction) {
        let existing_hash = self.zobrist_hash();
        let old_zobrist_val = self.zobrist_val(data, old);
        let new_zobrist_val = self.zobrist_val(data, new);
        self.set_zobrist_hash(existing_hash ^ old_zobrist_val ^ new_zobrist_val)
    }

    /// Performs a Zobrist hash update only based on a single action.
    fn update_zobrist_hash_partial(&mut self, data: &ZobristData, action: ZobristAction) {
        let existing_hash = self.zobrist_hash();
        let zobrist_val = self.zobrist_val(data, action);
        self.set_zobrist_hash(existing_hash ^ zobrist_val)
    }

    /// Returns the Zobrist value for a given action.
    fn zobrist_val(&self, data: &ZobristData, action: ZobristAction) -> u64 {
        data.get_val(self, action)
    }

    /// Returns the current Zobrist hash of the implementing data structure instance.
    fn zobrist_hash(&self) -> u64;

    /// Sets the Zobrist hash of the implement data structure instance to a
    /// particular value.
    fn set_zobrist_hash(&mut self, hash: u64);
}

/// Helper function to compare a new state with an old state, and update
/// its Zobrist hash value accordingly.
fn perform_updates<T: MancalaZobrist>(data: &ZobristData, old_state: &T, new_state: &mut T) {
    for player in [Player::One, Player::Two] {
        for pit in 0..old_state.pits() {
            let old_count = old_state.board()[player].as_ref()[pit];
            let new_count = new_state.board()[player].as_ref()[pit];
            if old_count == new_count {
                continue;
            }
            let old = ZobristAction::Pit(player, pit, old_count);
            let new = ZobristAction::Pit(player, pit, new_count);
            match new_state.update_zobrist_hash(data, old, new) {
                _ => {}
            }
        }
        let old_score = old_state.score(player);
        let new_score = new_state.score(player);
        if old_score == new_score {
            continue;
        }
        let old = ZobristAction::Store(player, old_state.score(player));
        let new = ZobristAction::Store(player, new_state.score(player));
        match new_state.update_zobrist_hash(data, old, new) {
            _ => {}
        }
    }
    if new_state.current_turn() != old_state.current_turn() {
        match new_state.update_zobrist_hash_partial(data, ZobristAction::SwitchTurn) {
            _ => {}
        }
    }
    if new_state.p2_moved() != old_state.p2_moved() {
        match new_state.update_zobrist_hash_partial(data, ZobristAction::P2Moved) {
            _ => {}
        }
    }
}
