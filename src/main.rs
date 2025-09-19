mod game;
mod minimax;

use crate::game::{GameState, Mancala, Move, Player};
use crate::minimax::MinimaxBuilder;
use rand::seq::IndexedRandom;
use std::io::Write;

fn main() {
    let mut rng = rand::rng();
    let mut s = GameState::default();

    while !s.is_over() {
        println!("{}", s);
        if s.current_turn() == Player::One {
            let minimax = MinimaxBuilder::new()
                .max_depth(Some(16))
                .optimize_for(Player::One)
                .build();
            let best_move = minimax
                .search(&s)
                .unwrap_or(*s.valid_moves().choose(&mut rng).unwrap());
            s = s.make_move(best_move).unwrap();
            println!("MINIMAX SELECTED: {:?}\n", best_move);
        } else {
            let mut selection: Option<Move> = None;
            while selection.is_none() {
                print!("PLAYER SELECTION: ");
                std::io::stdout().flush().unwrap();
                let mut input_line = String::new();
                std::io::stdin()
                    .read_line(&mut input_line)
                    .expect("Failed to read line");
                let input = input_line.trim();
                selection = if input.to_lowercase() == "swap" {
                    if s.valid_moves().contains(&Move::Swap) {
                        Some(Move::Swap)
                    } else {
                        None
                    }
                } else {
                    match input.parse::<usize>() {
                        Ok(n) if s.valid_moves().contains(&Move::Pit(n)) => Some(Move::Pit(n)),
                        _ => None,
                    }
                };
            }
            s = s.make_move(selection.unwrap()).unwrap();
            println!();
        }
    }
    println!("{}", s);
}
