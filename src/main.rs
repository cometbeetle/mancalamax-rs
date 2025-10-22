use mancalamax::game::{DynGameState, GameState};
use mancalamax::game::{Mancala, Move, Player};
use mancalamax::minimax::MinimaxBuilder;
use mancalamax::ui::{player_v_minimax, player_v_minimax_default, player_v_player_default};

fn main() {
    //player_v_player_default();
    //player_v_minimax_default(Player::One);
    //let minimax = MinimaxBuilder::new().max_depth(Some(18));
    //player_v_minimax(&GameState::default(), &minimax, Player::One);
    //mancalamax::ui::gui::make_gui();
    //println!("{:?}", GameState::default().valid_moves());

    // Dataset generation experiments.
    // Prints lines of [store1, store2, side1, side2, current_turn, best_move].
    // TODO: Move this into its own module; make more robust.
    for _ in 0..100 {
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
                .build();
            let result = minimax.search_utility(&state).unwrap();

            for i in state.stores() {
                print!("{} ", i)
            }
            for b in state.board() {
                for i in b {
                    print!("{} ", i)
                }
            }
            let turn: usize = state.current_turn().into();
            let mv = match result.0 {
                Move::Swap => -1,
                Move::Pit(i) => i as isize,
            };
            println!("{} {}", turn, mv);

            n_moves += 1;
        }
    }
}
