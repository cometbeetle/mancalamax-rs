//! Definitions and implementations for statically sized Mancala game states.

use super::common::fmt_common;
use super::dyn_game_state::DynGameState;
use super::mancala::{Mancala, Player};
use crate::minimax::{ZobristHash, ZobristIdx};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::sync::OnceLock;

/// Stores the necessary components of a Mancala game, including the board,
/// each player's store, the current ply, and the player currently allowed to move.
///
/// Uses a statically sized board for use in scenarios where the desired board size is
/// known at compile time. This provides higher performance than [`DynGameState`],
/// especially in scenarios where repeated state creation / modification is necessary
/// (i.e., during the execution of the minimax algorithm).
///
/// Implements the [`Mancala`] trait, and can be converted to and from
/// [`DynGameState`] structs. Also implements [`TTHash`] for use with minimax.
///
/// If the `serde` feature is enabled, this struct will be serializable and
/// deserializable, via automatic conversion to and from [`DynGameState`].
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct GameState<const N: usize> {
    board: [[usize; N]; 2],
    stores: [usize; 2],
    ply: usize,
    current_turn: Player,
    p2_moved: bool,
    zobrist_hash: u64,
}

#[cfg(feature = "serde")]
impl<const N: usize> serde::Serialize for GameState<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serde::Serialize::serialize(&DynGameState::from(*self), serializer)
    }
}

#[cfg(feature = "serde")]
impl<'a, const N: usize> serde::Deserialize<'a> for GameState<N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        let dyn_state = DynGameState::deserialize(deserializer)?;
        Ok(GameState::from(dyn_state))
    }
}

impl<const N: usize> Display for GameState<N> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        fmt_common(f, self, "Static GameState")
    }
}

impl Default for GameState<6> {
    /// The default Mancala board state is one in which each player
    /// has 6 pits, each containing 4 stones. The ply is set to 1,
    /// and the current turn is set to Player 1. Both stores start
    /// empty.
    fn default() -> Self {
        Self {
            board: [[4; 6]; 2],
            stores: [0, 0],
            ply: 1,
            current_turn: Player::One,
            p2_moved: false,
            zobrist_hash: 0,
        }
    }
}

impl<const N: usize> Mancala for GameState<N> {
    type Board = [usize; N];

    fn board(&self) -> &[Self::Board; 2] {
        &self.board
    }

    fn stores(&self) -> &[usize; 2] {
        &self.stores
    }

    fn ply(&self) -> usize {
        self.ply
    }

    fn current_turn(&self) -> Player {
        self.current_turn
    }

    fn p2_moved(&self) -> bool {
        self.p2_moved
    }

    fn set_p2_moved(&mut self, value: bool) {
        self.p2_moved = value;
    }

    fn board_mut(&mut self) -> &mut [Self::Board; 2] {
        &mut self.board
    }

    fn stores_mut(&mut self) -> &mut [usize; 2] {
        &mut self.stores
    }

    fn ply_mut(&mut self) -> &mut usize {
        &mut self.ply
    }

    fn current_turn_mut(&mut self) -> &mut Player {
        &mut self.current_turn
    }
}

impl<const N: usize> ZobristHash for GameState<N> {
    fn get_zobrist_val(&self, idx: ZobristIdx) -> Option<u64> {
        const SEED: u64 = 0x49CB86856BB06133;
        static ZOBRIST_PITS: OnceLock<Vec<u64>> = OnceLock::new();
        static ZOBRIST_STORES: OnceLock<Vec<u64>> = OnceLock::new();
        static ZOBRIST_SWITCH_TURN: OnceLock<u64> = OnceLock::new();
        static ZOBRIST_P2_MOVED: OnceLock<u64> = OnceLock::new();
        static TOTAL_STONES: OnceLock<usize> = OnceLock::new();

        let total_stones = TOTAL_STONES.get_or_init(|| {
            self.board.iter().flatten().sum::<usize>() + self.stores.iter().sum::<usize>()
        });

        let zobrist_pits = ZOBRIST_PITS.get_or_init(|| {
            let mut rng = StdRng::seed_from_u64(SEED);
            let total_entries = 2 * N * total_stones;
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

        match idx {
            ZobristIdx::Pit(player, pit, stones) => {
                let player = usize::from(player) - 1;
                let pit = pit - 1;
                let index = player * (N * total_stones) + pit * total_stones + stones;
                Some(zobrist_pits[index])
            }
            ZobristIdx::Store(player, stones) => {
                let player = usize::from(player) - 1;
                let index = player * total_stones + stones;
                Some(zobrist_stores[index])
            }
            ZobristIdx::SwitchTurn => Some(zobrist_switch_turn),
            ZobristIdx::P2Moved => Some(zobrist_p2_moved),
        }
    }

    fn get_zobrist_hash(&self) -> Option<u64> {
        Some(self.zobrist_hash)
    }

    fn set_zobrist_hash(&mut self, hash: u64) -> Result<(), ()> {
        Ok(self.zobrist_hash = hash)
    }
}

impl<const N: usize> From<DynGameState> for GameState<N> {
    fn from(value: DynGameState) -> Self {
        assert_eq!(
            value.board().len(),
            2,
            "Failed to convert DynGameState to GameState due to invalid dynamic board \
            (got Vec of length {}, expected 2)",
            value.board().len()
        );
        assert!(
            value.board()[0].len() == N && value.board()[1].len() == N,
            "Failed to convert DynGameState to GameState due to invalid dynamic board \
            (board[0] had length {}, board[1] had length {}; expected \
            {} for both)",
            value.board()[0].len(),
            value.board()[1].len(),
            N
        );
        let mut board = [[0; N]; 2];
        for i in 0..2 {
            for j in 0..N {
                board[i][j] = value.board()[i][j];
            }
        }
        Self {
            board,
            stores: *value.stores(),
            ply: value.ply(),
            current_turn: value.current_turn(),
            p2_moved: value.p2_moved(),
            zobrist_hash: value.zobrist_hash(),
        }
    }
}

impl<const N: usize> GameState<N> {
    /// Create a new [`GameState`] based on a series of parameters used
    /// to construct a starting game of Mancala.
    pub fn new(
        stones_per: usize,
        store_1: usize,
        store_2: usize,
        current_turn: Player,
        ply: usize,
        p2_moved: bool,
    ) -> Self {
        Self {
            board: [[stones_per; N]; 2],
            stores: [store_1, store_2],
            ply,
            current_turn,
            p2_moved,
            zobrist_hash: 0,
        }
    }

    /// Create a new [`GameState`] based on a preexisting board, stored as a
    /// [`Vec`] of [`Vec`] structs. The input vector must have an effective shape
    /// of `(2, N)`, where `N` is the number of pits per player.
    ///
    /// Note that because the size of [`Vec`] structs is not known at compile time,
    /// the board length `N` must be specified correctly in the generic call to
    /// [`GameState::from_vec`].
    pub fn from_vec(
        board: &Vec<Vec<usize>>,
        store_1: usize,
        store_2: usize,
        current_turn: Player,
        ply: usize,
        p2_moved: bool,
    ) -> Self {
        assert_eq!(
            board.len(),
            2,
            "GameState::from_vec failed due to invalid board input \
            (got Vec of length {}, expected 2)",
            board.len()
        );
        assert!(
            board[0].len() == N && board[1].len() == N,
            "GameState::from_vec failed due to invalid board input \
            (board[0] had length {}, board[1] had length {}; expected \
            {} for both)",
            board[0].len(),
            board[1].len(),
            N
        );
        let mut arr = [[0; N]; 2];
        for i in 0..2 {
            for j in 0..N {
                arr[i][j] = board[i][j];
            }
        }
        Self {
            board: arr,
            stores: [store_1, store_2],
            ply,
            current_turn,
            p2_moved,
            zobrist_hash: 0,
        }
    }

    /// Create a new [`GameState`] based on a preexisting board array.
    pub fn from_arr(
        board: [[usize; N]; 2],
        store_1: usize,
        store_2: usize,
        current_turn: Player,
        ply: usize,
        p2_moved: bool,
    ) -> Self {
        Self {
            board,
            stores: [store_1, store_2],
            ply,
            current_turn,
            p2_moved,
            zobrist_hash: 0,
        }
    }

    /// Returns the current Zobrist hash value stored in the state.
    pub fn zobrist_hash(&self) -> u64 {
        self.zobrist_hash
    }
}
