mod game;
mod minimax;

use crate::game::{DynGameState, GameState, Mancala, Player};
use crate::minimax::MinimaxBuilder;
use std::time::Duration;

fn main() {
    let s = GameState::default();
    println!("{}", s);

    let minimax = MinimaxBuilder::new()
        .max_depth(Some(12000000))
        //.max_time(Duration::from_secs(10))
        .build();
    let best_move = minimax.search(&s);
    println!("Best move: {:?}", best_move);
}

// TODO: Finish iterative deepening.
