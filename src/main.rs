mod game;
mod minimax;

use crate::game::{GameState, Mancala};
use crate::minimax::MinimaxBuilder;

fn main() {
    let s = GameState::default();
    println!("{}", s);
    let s = s.make_move_pit(2).unwrap();
    println!("{}", s);

    let minimax = MinimaxBuilder::new_standard().build();
}

// TODO: Finish implementing the minimax algorithm using alpha-beta pruning.
