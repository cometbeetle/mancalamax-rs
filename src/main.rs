mod game;
mod minimax;

use crate::game::{GameState, Mancala};
use crate::minimax::MinimaxBuilder;
use std::time::Duration;

fn main() {
    let s = GameState::default();
    println!("{}", s);

    let mut minimax = MinimaxBuilder::new_standard()
        .max_depth(None)
        .max_time(Duration::from_secs(10))
        .build();
    let best_move = minimax.search(&s);
    println!("Best move: {:?}", best_move);
}

// TODO: Finish iterative deepening.
