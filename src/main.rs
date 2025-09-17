use crate::game::{DynGameState, GameState, Mancala};

mod game;
mod minimax;

fn main() {
    let board = [[1, 1, 1], [1, 1, 1]];
    let b2 = vec![vec![1, 1, 1], vec![1, 1, 1]];
    let x = GameState::from_arr(board, 1, 2, 0, 0);
    let y = DynGameState::from_vec(b2, 1, 1, 0, 0);

    let z = GameState::default();
}

// TODO: Implement the minimax algorithm using
