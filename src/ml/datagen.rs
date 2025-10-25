//! Components for generating synthetic datasets for machine learning tasks.

use crate::game::{GameState, Mancala, Move, Player};
use crate::minimax::MinimaxBuilder;
use burn::prelude::*;
use burn::tensor::activation;
use rayon::prelude::*;
use std::collections::HashSet;
use std::path::Path;

pub fn save_dataset<P: AsRef<Path>>(data: &Vec<Vec<f32>>, out_file: P) {
    // Save dataset to file.
}

pub fn generate_dataset_default<B: Backend>(
    max_moves: usize,
    runs: usize,
    deduplicate: bool,
) -> Vec<Vec<f32>> {
    generate_dataset::<B, 6>(&MinimaxBuilder::new(), max_moves, runs, deduplicate)
}

pub fn generate_dataset<B: Backend, const N: usize>(
    minimax: &MinimaxBuilder<GameState<N>>,
    max_moves: usize,
    runs: usize,
    deduplicate: bool,
) -> Vec<Vec<f32>> {
    let device = B::Device::default();

    let generate = || {
        let mut data: Vec<Vec<f32>> = Vec::new();
        let mut n_moves = 0;
        while n_moves < max_moves {
            // Generate a random game state n_moves ahead from the initial state.
            // If out of moves, just use the last one that worked.
            let mut state = GameState::new(4, 0, 0, Player::One, 1, false);
            for _ in 0..n_moves {
                (state, _) = match state.make_move_rand() {
                    Some(t) => t,
                    None => break,
                }
            }

            // Regenerate until random game is not in a terminal state.
            if state.is_over() {
                continue;
            }

            // Compute the optimal move and utility for that state for the current player.
            let minimax = minimax.clone().optimize_for(state.current_turn()).build();
            let utilities = minimax.search_utility_all(&state).unwrap();
            let mut example: Vec<f32> = Vec::new();

            // Push stores.
            for i in state.stores() {
                example.push(*i as f32);
            }

            // Push board.
            for b in state.board() {
                for i in b {
                    example.push(*i as f32);
                }
            }

            // Push current turn.
            example.push(state.current_turn() as usize as f32);

            // Push softmax values for utility on each move.
            let mut util_vec = vec![f32::NEG_INFINITY; N + 1];
            for (mv, util) in utilities {
                let mv = match mv {
                    Move::Swap => 0,
                    Move::Pit(i) => i,
                };
                util_vec[mv] = util;
            }
            let util_tensor = Tensor::<B, 1>::from_data(util_vec.as_slice(), &device);
            let softmax_utils = activation::softmax(util_tensor, 0);
            for i in softmax_utils.to_data().iter() {
                example.push(i);
            }

            data.push(example);
            n_moves += 1;
        }
        data
    };

    let mut result: Vec<Vec<f32>> = (0..runs)
        .into_par_iter()
        .map(|_| generate())
        .flatten()
        .collect();

    if deduplicate {
        let mut seen = HashSet::new();
        let mut unique = Vec::new();

        for item in result {
            let key: Vec<u32> = item.iter().map(|x| x.to_bits()).collect();
            if seen.insert(key) {
                unique.push(item);
            }
        }

        result = unique;
    }

    result
}

// TODO: Add docs, clean up, make this optimal. May want to use Dataset from Burn directly.
