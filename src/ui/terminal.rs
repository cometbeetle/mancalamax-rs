//! Components for the terminal user interface.

use crate::game::{GameOutcome, GameState, Mancala, Move, Player};
use crate::minimax::MinimaxBuilder;
use std::io::Write;

/// Helper function for collecting valid user inputs via standard input.
fn user_move_input<T: Mancala>(state: &T) -> Move {
    let mut selection: Option<Move> = None;
    let player_int: usize = state.current_turn().into();
    let valid_moves = state.valid_moves();

    // Loop until the player inputs a valid move.
    while selection.is_none() {
        print!("PLAYER {} SELECTION: ", player_int);
        std::io::stdout().flush().unwrap();
        let mut input_line = String::new();
        std::io::stdin()
            .read_line(&mut input_line)
            .expect("Failed to read line");
        let input = input_line.trim();
        selection = if input.to_lowercase() == "swap" {
            if valid_moves.contains(&Move::Swap) {
                Some(Move::Swap)
            } else {
                None
            }
        } else {
            match input.parse::<usize>() {
                Ok(n) if valid_moves.contains(&Move::Pit(n)) => Some(Move::Pit(n)),
                _ => None,
            }
        };
    }
    selection.unwrap()
}

/// Start a terminal-based game of Mancala between a player and a specified
/// minimax algorithm based on an initial state.
pub fn player_v_minimax<T: Mancala>(
    initial_state: &T,
    minimax_builder: &MinimaxBuilder<T>,
    minimax_player: Player,
) -> GameOutcome {
    let mut s = initial_state.clone();
    let minimax = minimax_builder.build();

    while !s.is_over() {
        println!("{}", s);
        if s.current_turn() == minimax_player {
            let chosen_move: Move;
            (s, chosen_move) = match minimax.search(&s) {
                Some(m) => (s.make_move(m).unwrap(), m),
                None => s.make_move_rand().unwrap(),
            };
            println!("MINIMAX SELECTED: {:?}\n", chosen_move);
        } else {
            s = s.make_move(user_move_input(&s)).unwrap();
            println!();
        }
    }

    let winner = s.outcome();

    match winner {
        GameOutcome::Winner(player) if player == minimax_player => {
            println!("{}\nWINNER: MINIMAX", s)
        }
        GameOutcome::Winner(player) if player != minimax_player => {
            println!("{}\nWINNER: PLAYER {}", s, usize::from(player))
        }
        GameOutcome::Tie => println!("{}\nWINNER: TIE", s),
        _ => println!("{}\nWINNER: N/A", s),
    }

    winner
}

/// Start a terminal-based game of Mancala between a player and the default
/// minimax algorithm specified by [`MinimaxBuilder::default()`], using the
/// default game state specified by [`GameState::default()`].
pub fn player_v_minimax_default(minimax_player: Player) -> GameOutcome {
    let minimax_builder = MinimaxBuilder::default().optimize_for(minimax_player);
    player_v_minimax(&GameState::default(), &minimax_builder, minimax_player)
}

/// Start a terminal-based game of Mancala between two players based on
/// an initial board state.
pub fn player_v_player<T: Mancala>(initial_state: &T) -> GameOutcome {
    let mut s = initial_state.clone();

    while !s.is_over() {
        println!("{}", s);
        s = s.make_move(user_move_input(&s)).unwrap();
        println!();
    }

    let winner = s.outcome();

    match winner {
        GameOutcome::Winner(player) => println!("{}\nWINNER: PLAYER {}", s, usize::from(player)),
        GameOutcome::Tie => println!("{}\nWINNER: TIE", s),
        _ => println!("{}\nWINNER: N/A", s),
    }

    winner
}

/// Start a terminal-based game of Mancala between two players based on
/// the default game state specified by [`GameState::default()`].
pub fn player_v_player_default() -> GameOutcome {
    player_v_player(&GameState::default())
}
