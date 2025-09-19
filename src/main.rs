mod game;
mod minimax;

use crate::game::{GameState, Mancala, Move};

fn main() {
    let s = GameState::default();
    println!("{}", s);
    let s = s.make_move(Move::Pit(3)).unwrap();
    println!("{}", s);
}

// TODO: Implement the minimax algorithm using alpha-beta pruning.
