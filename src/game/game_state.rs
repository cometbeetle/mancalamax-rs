use super::common::fmt_common;
use super::dyn_game_state::DynGameState;
use super::mancala::sealed::MancalaPrivate;
use super::mancala::{Mancala, Player};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy)]
pub(crate) struct GameState<const N: usize> {
    board: [[usize; N]; 2],
    stores: [usize; 2],
    ply: usize,
    current_turn: Player,
}

impl<const N: usize> Display for GameState<N> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        fmt_common(f, self, "Static GameState")
    }
}

impl Default for GameState<6> {
    fn default() -> Self {
        Self {
            board: [[4; 6]; 2],
            stores: [0, 0],
            ply: 1,
            current_turn: Player::One,
        }
    }
}

impl<const N: usize> MancalaPrivate<[usize; N]> for GameState<N> {
    fn board_mut(&mut self) -> &mut [[usize; N]; 2] {
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

impl<const N: usize> Mancala<[usize; N]> for GameState<N> {
    fn board(&self) -> &[[usize; N]; 2] {
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
}

impl<const N: usize> From<DynGameState> for GameState<N> {
    fn from(value: DynGameState) -> Self {
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
        }
    }
}

impl<const N: usize> GameState<N> {
    pub(crate) fn new(
        stones_per: usize,
        store_1: usize,
        store_2: usize,
        current_turn: Player,
        ply: usize,
    ) -> Self {
        Self {
            board: [[stones_per; N]; 2],
            stores: [store_1, store_2],
            ply,
            current_turn,
        }
    }
    pub(crate) fn from_vec(
        board: &Vec<Vec<usize>>,
        store_1: usize,
        store_2: usize,
        current_turn: Player,
        ply: usize,
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
        }
    }
    pub(crate) fn from_arr(
        board: [[usize; N]; 2],
        store_1: usize,
        store_2: usize,
        current_turn: Player,
        ply: usize,
    ) -> Self {
        Self {
            board,
            stores: [store_1, store_2],
            ply,
            current_turn,
        }
    }
}
