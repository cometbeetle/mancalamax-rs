//! Components for the terminal user interface.

use crate::game::{GameOutcome, GameState, Mancala, Move, Player};
use crate::minimax::MinimaxBuilder;
use std::error::Error;
use std::fs;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;
use std::thread;
use std::time::Duration;

/// Enum specifying which file IO interface to use for external players.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ExternalInterface {
    Full,
    Minimal,
}

impl ExternalInterface {
    pub fn read_moves<P: AsRef<Path>>(&self, path: P) -> io::Result<Vec<Move>> {
        match self {
            ExternalInterface::Full => {
                todo!()
            }
            ExternalInterface::Minimal => {
                // Wait until the file exists.
                while !path.as_ref().exists() {
                    thread::sleep(Duration::from_millis(100));
                }

                // Read the file contents once it exists.
                let contents = fs::read_to_string(&path)?;

                // Parse space-separated usize values.
                let moves = contents
                    .split_whitespace()
                    .map(|s| Move::from(s.parse::<usize>().unwrap()))
                    .collect();

                Ok(moves)
            }
        }
    }

    pub fn write_board<T: Mancala, P: AsRef<Path>>(
        &self,
        path: P,
        state: &T,
    ) -> Result<(), Box<dyn Error>> {
        match self {
            ExternalInterface::Full => {
                todo!()
            }
            ExternalInterface::Minimal => {
                let mut file = OpenOptions::new().write(true).create(true).open(path)?;

                let mut values: Vec<usize> = Vec::new();

                // Orient the board so that it is from the current player's perspective.
                if state.current_turn() == Player::Two {
                    values.push(state.stores()[1]);
                    values.push(state.stores()[0]);
                    values.extend(state.board()[1].as_ref());
                    values.extend(state.board()[0].as_ref());
                } else {
                    values.push(state.stores()[0]);
                    values.push(state.stores()[1]);
                    values.extend(state.board()[0].as_ref());
                    values.extend(state.board()[1].as_ref());
                }

                let content = values
                    .iter()
                    .map(|c| c.to_string())
                    .collect::<Vec<_>>()
                    .join(" ");

                write!(file, "{}", content)?;
                Ok(())
            }
        }
    }
}

/// Helper function for collecting valid user inputs via standard input.
fn user_move_input<T: Mancala>(state: &T) -> Move {
    let mut selection: Option<Move> = None;
    let player_int: usize = state.current_turn().into();
    let valid_moves = state.valid_moves();

    // Loop until the player inputs a valid move.
    while selection.is_none() {
        print!("PLAYER {} SELECTION: ", player_int);
        io::stdout().flush().unwrap();
        let mut input_line = String::new();
        io::stdin()
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
/// minimax algorithm specified by [`MinimaxBuilder::default`], using the
/// default game state specified by [`GameState::default`].
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
/// the default game state specified by [`GameState::default`].
pub fn player_v_player_default() -> GameOutcome {
    player_v_player(&GameState::default())
}

pub fn player_v_external<T: Mancala, P: AsRef<Path>>(
    initial_state: &T,
    external_player: Player,
    interface: ExternalInterface,
    comm_dir: P,
) -> GameOutcome {
    let mut s = initial_state.clone();
    let mut current_move = 0usize;
    let comm_dir = comm_dir.as_ref();

    // Create communication directory if it does not exist.
    fs::create_dir_all(comm_dir).unwrap();

    while !s.is_over() {
        println!("{}", s);
        if s.current_turn() == external_player {
            // Write board to external player.
            interface
                .write_board(comm_dir.join(format!("board{}.txt", current_move)), &s)
                .unwrap();

            // Read from file until expected number of moves is found.
            let mut moves: Vec<Move> = Vec::new();
            while moves.len() != s.pits() + 1 {
                moves = match interface
                    .read_moves(comm_dir.join(format!("moves{}.txt", current_move)))
                {
                    Ok(m) => m,
                    Err(_) => continue,
                }
            }

            // Select first valid move.
            let mut chosen_move: Option<Move> = None;
            for m in moves {
                if s.valid_moves().contains(&m) {
                    chosen_move = Some(m);
                    break;
                }
            }

            s = s.make_move(chosen_move.unwrap()).unwrap();
            println!("EXTERNAL SELECTED: {:?}\n", chosen_move);
            current_move += 1;
        } else {
            s = s.make_move(user_move_input(&s)).unwrap();
            println!();
        }
    }

    let winner = s.outcome();

    match winner {
        GameOutcome::Winner(player) if player == external_player => {
            println!("{}\nWINNER: EXTERNAL PLAYER", s)
        }
        GameOutcome::Winner(player) if player != external_player => {
            println!("{}\nWINNER: PLAYER {}", s, usize::from(player))
        }
        GameOutcome::Tie => println!("{}\nWINNER: TIE", s),
        _ => println!("{}\nWINNER: N/A", s),
    }

    winner
}

pub fn minimax_v_external<T: Mancala, P: AsRef<Path>>(
    initial_state: &T,
    minimax_builder: &MinimaxBuilder<T>,
    external_player: Player,
    interface: ExternalInterface,
    comm_dir: P,
) -> GameOutcome {
    let mut s = initial_state.clone();
    let minimax = minimax_builder.build();
    let mut current_move = 0usize;
    let comm_dir = comm_dir.as_ref();

    // Create communication directory if it does not exist.
    fs::create_dir_all(comm_dir).unwrap();

    while !s.is_over() {
        println!("{}", s);
        if s.current_turn() == external_player {
            // Write board to external player.
            interface
                .write_board(comm_dir.join(format!("board{}.txt", current_move)), &s)
                .unwrap();

            // Read from file until expected number of moves is found.
            let mut moves: Vec<Move> = Vec::new();
            while moves.len() != s.pits() + 1 {
                moves = match interface
                    .read_moves(comm_dir.join(format!("moves{}.txt", current_move)))
                {
                    Ok(m) => m,
                    Err(_) => continue,
                }
            }

            // Select first valid move.
            let mut chosen_move: Option<Move> = None;
            for m in moves {
                if s.valid_moves().contains(&m) {
                    chosen_move = Some(m);
                    break;
                }
            }

            s = s.make_move(chosen_move.unwrap()).unwrap();
            println!("EXTERNAL SELECTED: {:?}\n", chosen_move);
            current_move += 1;
        } else {
            let chosen_move: Move;
            (s, chosen_move) = match minimax.search(&s) {
                Some(m) => (s.make_move(m).unwrap(), m),
                None => s.make_move_rand().unwrap(),
            };
            println!("MINIMAX SELECTED: {:?}\n", chosen_move);
        }
    }

    let winner = s.outcome();

    match winner {
        GameOutcome::Winner(player) if player == external_player => {
            println!("{}\nWINNER: EXTERNAL PLAYER", s)
        }
        GameOutcome::Winner(player) if player != external_player => {
            println!("{}\nWINNER: MINIMAX", s)
        }
        GameOutcome::Tie => println!("{}\nWINNER: TIE", s),
        _ => println!("{}\nWINNER: N/A", s),
    }

    winner
}

// TODO: CLEANUP, AVOID DUPLICATE CODE, MAKE DIRECTORY CLEARED UPON NEW GAME

// TODO: ALSO NEED SOME WAY TO SIGNAL RESTART!
