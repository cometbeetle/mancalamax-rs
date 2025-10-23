use mancalamax::game::{DynGameState, GameState};
use mancalamax::game::{Mancala, Move, Player};
use mancalamax::minimax::MinimaxBuilder;
use mancalamax::ui::{player_v_minimax, player_v_minimax_default, player_v_player_default};

fn softmax(v: &[f32]) -> Vec<f32> {
    let max = v.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let exp_values: Vec<f32> = v.iter().map(|&x| (x - max).exp()).collect();
    let sum: f32 = exp_values.iter().sum();
    exp_values.iter().map(|&x| x / sum).collect()
}

fn main() {
    //player_v_player_default();
    //player_v_minimax_default(Player::One);
    //let minimax = MinimaxBuilder::new().max_depth(Some(18));
    //player_v_minimax(&GameState::default(), &minimax, Player::One);
    //mancalamax::ui::gui::make_gui();
    //println!("{:?}", GameState::default().valid_moves());

    // Dataset generation experiments.
    // Prints lines of [store1, store2, side1, side2, current_turn, best_move].
    // TODO: Clean this up and move into its own module; make more robust.
    // TODO: Fix docstrings everywhere to be consistent with the code fonts.
    for _ in 0..5 {
        let mut n_moves = 0;
        while n_moves < 70 {
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
            let minimax = MinimaxBuilder::new()
                .optimize_for(state.current_turn())
                .max_depth(Some(16))
                .build();
            let result = minimax.search_utility_all(&state).unwrap();

            // Print stores.
            for i in state.stores() {
                print!("{} ", i)
            }

            // Print board.
            for b in state.board() {
                for i in b {
                    print!("{} ", i)
                }
            }

            // Print current turn.
            let turn: usize = state.current_turn().into();
            print!("{} ", turn);

            // Print softmax values for utility on each move.
            let mut vec: Vec<f32> = vec![f32::NEG_INFINITY; 7];
            for (mv, util) in result {
                let mv = match mv {
                    Move::Swap => 0,
                    Move::Pit(i) => i,
                };
                vec[mv] = util;
            }
            vec = softmax(&vec);
            for i in 0..vec.len() {
                if i == vec.len() - 1 {
                    println!("{}", vec[i])
                } else {
                    print!("{} ", vec[i])
                }
            }

            n_moves += 1;
        }
    }
}
