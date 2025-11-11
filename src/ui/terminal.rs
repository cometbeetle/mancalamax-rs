//! Components for the terminal user interface.

use crate::game::{GameOutcome, GameState, Mancala, Move, Player};
use crate::minimax::MinimaxBuilder;
use regex::Regex;
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

    pub fn write_board<T: Mancala, P: AsRef<Path>>(&self, path: P, state: &T) -> io::Result<()> {
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

/// Helper function for collecting valid external moves from an external interface.
fn external_move_input<T: Mancala, P: AsRef<Path>>(
    state: &T,
    interface: ExternalInterface,
    comm_dir: P,
    current_move: usize,
) -> Move {
    let comm_dir = comm_dir.as_ref();

    // Write board to external player.
    interface
        .write_board(comm_dir.join(format!("board{}.txt", current_move)), state)
        .unwrap();

    // Read from file until expected number of moves is found.
    let mut moves: Vec<Move> = Vec::new();
    while moves.len() != state.pits() + 1 {
        moves = match interface.read_moves(comm_dir.join(format!("moves{}.txt", current_move))) {
            Ok(m) => m,
            Err(_) => continue,
        }
    }

    // Select first valid move.
    let mut chosen_move: Option<Move> = None;
    for m in moves {
        if state.valid_moves().contains(&m) {
            chosen_move = Some(m);
            break;
        }
    }

    chosen_move.unwrap()
}

// Helper function to send a reset signal to the external program.
fn external_reset<P: AsRef<Path>>(comm_dir: P) {
    let re = Regex::new(r"^(board|moves)\d+\.txt$").unwrap();
    match fs::read_dir(&comm_dir) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                            if re.is_match(filename) {
                                match fs::remove_file(&path) {
                                    Ok(_) => (),
                                    Err(e) => eprintln!("Failed to remove {:?}: {}", path, e),
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(e) => eprintln!("Failed to read directory: {}", e),
    }
    OpenOptions::new()
        .write(true)
        .create(true)
        .open(comm_dir.as_ref().join("RESET.txt"))
        .expect("Failed to create file");
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

/// Start a terminal-based game of Mancala between a player and an external
/// agent, using the selected communication interface and directory. A supplied
/// starting state is used.
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

    // Reset directory.
    external_reset(comm_dir);

    while !s.is_over() {
        println!("{}", s);
        if s.current_turn() == external_player {
            let chosen_move = external_move_input(&s, interface, comm_dir, current_move);
            s = s.make_move(chosen_move).unwrap();
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
            println!("{}\nWINNER: EXTERNAL AGENT", s)
        }
        GameOutcome::Winner(player) if player != external_player => {
            println!("{}\nWINNER: PLAYER {}", s, usize::from(player))
        }
        GameOutcome::Tie => println!("{}\nWINNER: TIE", s),
        _ => println!("{}\nWINNER: N/A", s),
    }

    winner
}

/// Start a terminal-based game of Mancala between a player and an external
/// agent, using the selected communication interface and directory. The
/// default starting state is used.
pub fn player_v_external_default<P: AsRef<Path>>(
    external_player: Player,
    interface: ExternalInterface,
    comm_dir: P,
) -> GameOutcome {
    player_v_external(&GameState::default(), external_player, interface, comm_dir)
}

/// Start a terminal-based game of Mancala between minimax and an external
/// agent, using the selected communication interface and directory. A supplied
/// starting state and minimax configuration are used.
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

    // Reset directory.
    external_reset(comm_dir);

    while !s.is_over() {
        println!("{}", s);
        if s.current_turn() == external_player {
            let chosen_move = external_move_input(&s, interface, comm_dir, current_move);
            s = s.make_move(chosen_move).unwrap();
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
            println!("{}\nWINNER: EXTERNAL AGENT", s)
        }
        GameOutcome::Winner(player) if player != external_player => {
            println!("{}\nWINNER: MINIMAX", s)
        }
        GameOutcome::Tie => println!("{}\nWINNER: TIE", s),
        _ => println!("{}\nWINNER: N/A", s),
    }

    winner
}

/// Start a terminal-based game of Mancala between minimax and an external
/// agent, using the selected communication interface and directory. The default
/// starting state and minimax configuration are used.
pub fn minimax_v_external_default<P: AsRef<Path>>(
    external_player: Player,
    interface: ExternalInterface,
    comm_dir: P,
) -> GameOutcome {
    minimax_v_external(
        &GameState::default(),
        &MinimaxBuilder::default(),
        external_player,
        interface,
        comm_dir,
    )
}
