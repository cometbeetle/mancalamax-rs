//! Components for generating synthetic datasets for machine learning tasks.

use crate::game::{GameState, Mancala, Move};
use crate::minimax::MinimaxBuilder;
use burn::prelude::*;
use burn::tensor::activation;
use rayon::prelude::*;
use std::path::Path;

pub fn save_dataset<B: Backend, P: AsRef<Path>>(data: &Tensor<B, 2>, out_file: P) {
    // Save dataset to file.
}

pub fn generate_dataset_default<B: Backend>(max_moves: usize, runs: usize) -> Tensor<B, 2> {
    generate_dataset(&MinimaxBuilder::new(), max_moves, runs)
}

pub fn generate_dataset<B: Backend>(
    minimax: &MinimaxBuilder<GameState<6>>,
    max_moves: usize,
    runs: usize,
) -> Tensor<B, 2> {
    let device = B::Device::default();

    let generate = || {
        let mut data: Vec<Vec<f32>> = Vec::new();
        let mut n_moves = 0;
        while n_moves < max_moves {
            // Generate a random game state n_moves ahead from the initial state.
            // If out of moves, just use the last one that worked.
            let mut state = GameState::default();
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
            let mut util_vec: Vec<f32> = vec![f32::NEG_INFINITY; 7];
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

    let result: Vec<Vec<f32>> = (0..runs)
        .into_par_iter()
        .map(|_| generate())
        .flatten()
        .collect();
    let (dim1, dim2) = (result.len(), result[0].len());
    let data = TensorData::new(result.into_iter().flatten().collect(), &[dim1, dim2]);
    Tensor::<B, 2>::from_data(data, &device)
}

// TODO: Add docs, clean up, make this optimal. May want to use Dataset from Burn directly.
