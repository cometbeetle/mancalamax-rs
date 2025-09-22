//! Definitions and implementations for dynamically sized Mancala game states.

use super::common::fmt_common;
use super::game_state::GameState;
use super::mancala::{Mancala, Player};
use std::fmt::{Display, Formatter};

/// Stores the necessary components of a Mancala game, including the board,
/// each player's store, the current ply, and the player currently allowed to move.
///
/// Uses a dynamically sized board for use in scenarios where the desired board size
/// is not known at compile time.
///
/// Implements the [`Mancala`] trait, and can be converted to and from
/// [`GameState`] structs.
#[derive(Debug, Clone)]
pub struct DynGameState {
    board: [Vec<usize>; 2],
    stores: [usize; 2],
    ply: usize,
    current_turn: Player,
}

impl Display for DynGameState {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        fmt_common(f, self, "Dynamic GameState")
    }
}

impl Default for DynGameState {
    fn default() -> Self {
        Self {
            board: [vec![4; 6], vec![4; 6]],
            stores: [0, 0],
            ply: 1,
            current_turn: Player::One,
        }
    }
}

impl Mancala for DynGameState {
    type Board = Vec<usize>;

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

impl<const N: usize> From<GameState<N>> for DynGameState {
    fn from(value: GameState<N>) -> Self {
        Self {
            board: [value.board()[0].to_vec(), value.board()[1].to_vec()],
            stores: *value.stores(),
            ply: value.ply(),
            current_turn: value.current_turn(),
        }
    }
}

impl DynGameState {
    /// Create a new [`DynGameState`] based on a series of parameters used
    /// to construct a starting game of Mancala.
    pub fn new(
        pits: usize,
        stones_per: usize,
        store_1: usize,
        store_2: usize,
        current_turn: Player,
        ply: usize,
    ) -> Self {
        Self {
            board: [vec![stones_per; pits], vec![stones_per; pits]],
            stores: [store_1, store_2],
            ply,
            current_turn,
        }
    }

    /// Create a new [`DynGameState`] based on a preexisting board, stored as a
    /// [`Vec`] of [`Vec`] structs. The input vector must have an effective shape
    /// of `(2, N)`, where `N` is the number of pits per player.
    pub fn from_vec(
        board: &Vec<Vec<usize>>,
        store_1: usize,
        store_2: usize,
        current_turn: Player,
        ply: usize,
    ) -> Self {
        assert_eq!(
            board.len(),
            2,
            "DynGameState::from_vec failed due to invalid board input \
            (got Vec of length {}, expected 2)",
            board.len()
        );
        assert_eq!(
            board[0].len(),
            board[1].len(),
            "DynGameState::from_vec failed due to invalid board input \
            (board[0] had length {}, board[1] had length {}; expected \
            the same for both)",
            board[0].len(),
            board[1].len(),
        );
        Self {
            board: [board[0].clone(), board[1].clone()],
            stores: [store_1, store_2],
            ply,
            current_turn,
        }
    }

    /// Create a new [`DynGameState`] based on a preexisting board array.
    pub fn from_arr<const N: usize>(
        board: [[usize; N]; 2],
        store_1: usize,
        store_2: usize,
        current_turn: Player,
        ply: usize,
    ) -> Self {
        Self {
            board: [board[0].to_vec(), board[1].to_vec()],
            stores: [store_1, store_2],
            ply,
            current_turn,
        }
    }
}
