//! Traits and enums necessary for Mancala gameplay.

use std::fmt::Display;
use std::ops::{Index, IndexMut};

/// Mancala games have two players. Therefore, the [`Player`] enum can be one
/// of two variants, [`One`][Self::One], or [`Two`][Self::Two].
///
/// Can be converted to [`usize`], or used as an index, where [`One`][Self::One]
/// is index 0, and [`Two`][Self::Two] is index 1.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Player {
    One,
    Two,
}

impl<T> Index<Player> for [T] {
    type Output = T;

    fn index(&self, index: Player) -> &Self::Output {
        match index {
            Player::One => &self[0],
            Player::Two => &self[1],
        }
    }
}

impl<T> IndexMut<Player> for [T] {
    fn index_mut(&mut self, index: Player) -> &mut Self::Output {
        match index {
            Player::One => &mut self[0],
            Player::Two => &mut self[1],
        }
    }
}

impl From<Player> for usize {
    fn from(value: Player) -> Self {
        match value {
            Player::One => 1,
            Player::Two => 2,
        }
    }
}

/// Represents one of the two types of move during Mancala gameplay.
///
/// Players can either select a pit from which to distribute stones, or,
/// under certain circumstances at the beginning of the game, the second
/// player can swap the board.
///
/// If the [`Pit`][Self::Pit] variant is instantiated, it stores a [`usize`] value
/// indicating which pit (starting from pit 1) is to be chosen.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Move {
    Pit(usize),
    Swap,
}

/// Used to describe the current outcome of a game state.
///
/// During gameplay, there can either be a winner, a tie, or the game may still be ongoing.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GameOutcome {
    Winner(Player),
    Tie,
    Ongoing,
}

/// Provides a default implementation of Mancala gameplay for all implementors,
/// and specifies certain accessor and mutable reference methods that must be
/// implemented on a per-type basis (i.e., no default implementation can be provided).
pub trait Mancala: Clone + Display {
    /// Used to indicate the underlying array-like type used to store
    /// the board contents for each player.
    type Board: AsRef<[usize]> + AsMut<[usize]>;

    /// Converts the current board into an array of two [`Vec`] instances for easy access.
    fn board_as_vecs(&self) -> [Vec<usize>; 2] {
        [
            self.board()[0].as_ref().to_vec(),
            self.board()[1].as_ref().to_vec(),
        ]
    }

    /// Determines whether the game is over.
    fn is_over(&self) -> bool {
        for player in 0..2 {
            for pit in self.board()[player].as_ref() {
                if *pit != 0 {
                    return false;
                }
            }
        }
        true
    }

    /// Gets the current score for a player.
    fn score(&self, player: Player) -> isize {
        self.stores()[player] as isize
    }

    /// Determines the current outcome of the game.
    fn outcome(&self) -> GameOutcome {
        if self.is_over() {
            if self.score(Player::One) > self.score(Player::Two) {
                GameOutcome::Winner(Player::One)
            } else if self.score(Player::Two) > self.score(Player::One) {
                GameOutcome::Winner(Player::Two)
            } else {
                GameOutcome::Tie
            }
        } else {
            GameOutcome::Ongoing
        }
    }

    /// Determines whether the swap move is currently allowed.
    ///
    /// `[WORK IN PROGRESS]`
    fn swap_allowed(&self) -> bool {
        // TODO: Technically, this doesn't cover all cases where swap is allowed.
        self.current_turn() == Player::Two && self.ply() == 2
    }

    /// Returns a vector of moves that are currently valid for the current player.
    fn valid_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();

        // If the Swap move is available for player 2.
        if self.swap_allowed() {
            moves.insert(0, Move::Swap);
        }

        // List all pits where the number of stones > 0.
        for (i, pit) in self.board()[self.current_turn()]
            .as_ref()
            .iter()
            .enumerate()
        {
            if *pit > 0 {
                moves.insert(0, Move::Pit(i + 1));
            }
        }

        moves
    }

    /// Switches the current turn. Used inside [`make_move`][Self::make_move].
    fn switch_turn(&mut self) -> Player {
        let turn = *self.current_turn_mut();
        *self.current_turn_mut() = if turn == Player::One {
            Player::Two
        } else {
            Player::One
        };
        *self.current_turn_mut()
    }

    /// Rotates the board. Used inside [`make_move`][Self::make_move] when the swap move is requested.
    fn rotate_board(&mut self) {
        self.board_mut().swap(0, 1);
        self.stores_mut().swap(0, 1);
    }

    /// Returns a new board state, updated to reflect the result of making
    /// the specified move. If the move was invalid, returns [`None`].
    ///
    /// The default implementation of [`make_move`][Self::make_move] roughly
    /// follows the gameplay rules of the "Kalah" variant of Mancala.
    fn make_move(&self, selection: Move) -> Option<Self> {
        // Make a copy of the current state.
        let mut new_state = self.clone();

        let mut pit = match selection {
            // Handle swap inputs.
            Move::Swap => {
                if !new_state.swap_allowed() {
                    return None;
                }
                new_state.rotate_board();
                new_state.switch_turn();
                *new_state.ply_mut() += 1;
                return Some(new_state);
            }
            Move::Pit(pit) => {
                if pit < 1 || pit > new_state.pits() {
                    return None;
                }
                pit
            }
        };

        // Get current player, find adjusted pit index, and collect number of stones to distribute.
        let mut side = new_state.current_turn();
        let stones = {
            let player_side = new_state.board_mut()[side].as_mut();
            let stones = player_side[pit - 1];
            player_side[pit - 1] = 0;
            stones
        };

        // Ensure selected pit had stones in it.
        if stones == 0 {
            return None;
        }

        // Initialize turn variables.
        let mut go_again = false;

        // Distribute the stones of the selected pit.
        let mut i = 0;
        while i < stones {
            let last_stone = i == stones - 1;

            if pit != new_state.pits() {
                // Add stone to pit.
                new_state.board_mut()[side].as_mut()[pit] += 1;
            } else {
                // Only add stones to the current player's store.
                let add_to_store = side == self.current_turn();
                if add_to_store {
                    new_state.stores_mut()[side] += 1;
                    go_again = last_stone;
                }

                // Switch board sides.
                side = if side == Player::One {
                    Player::Two
                } else {
                    Player::One
                };
                pit = 0;

                // If we did not add to the store, make sure to add one to the next player's pit.
                // If we DID add to the store, and if that wasn't the last stone, add one to the
                // next player's pit, and increment i to avoid adding two stones for the same i.
                if !add_to_store {
                    new_state.board_mut()[side].as_mut()[pit] += 1;
                } else if !last_stone {
                    new_state.board_mut()[side].as_mut()[pit] += 1;
                    i += 1;
                }
            }

            // Determine which stones to capture (if any).
            if last_stone
                && side == new_state.current_turn()
                && new_state.board()[side].as_ref()[pit] == 1
            {
                let to_capture = if side == Player::One {
                    [pit, new_state.pits() - pit - 1]
                } else {
                    [new_state.pits() - pit - 1, pit]
                };

                new_state.stores_mut()[side] += new_state.board()[0].as_ref()[to_capture[0]];
                new_state.stores_mut()[side] += new_state.board()[1].as_ref()[to_capture[1]];
                new_state.board_mut()[0].as_mut()[to_capture[0]] = 0;
                new_state.board_mut()[1].as_mut()[to_capture[1]] = 0;
            }

            pit += 1;
            i += 1;
        }

        // Detect completed game.
        let final_stone_recipient: Option<usize> = {
            if new_state.board()[0].as_ref().iter().sum::<usize>() == 0 {
                Some(1)
            } else if new_state.board()[1].as_ref().iter().sum::<usize>() == 0 {
                Some(0)
            } else {
                None
            }
        };

        // If game is finished, player with stones on their side captures them all.
        if let Some(winner) = final_stone_recipient {
            for pit in 0..new_state.pits() {
                new_state.stores_mut()[winner] += new_state.board()[winner].as_ref()[pit];
                new_state.board_mut()[winner].as_mut()[pit] = 0;
            }
        }

        // Don't switch players if player goes again.
        if !go_again {
            new_state.switch_turn();
        }

        *new_state.ply_mut() += 1;

        Some(new_state)
    }

    /// Helper method to select a pit move without the encapsulating enum.
    fn make_move_pit(&self, pit: usize) -> Option<Self> {
        self.make_move(Move::Pit(pit))
    }

    /// Helper method to select the swap move without the encapsulating enum.
    fn make_move_swap(&self) -> Option<Self> {
        self.make_move(Move::Swap)
    }

    /// Returns the number of pits per player for the current game.
    fn pits(&self) -> usize {
        self.board()[0].as_ref().len()
    }

    /// Provides immutable access to the board.
    fn board(&self) -> &[Self::Board; 2];

    /// Provides immutable access to the stores.
    fn stores(&self) -> &[usize; 2];

    /// Returns the current ply.
    fn ply(&self) -> usize;

    /// Returns the player currently allowed to move.
    fn current_turn(&self) -> Player;

    /// Provides mutable access to the board.
    ///
    /// NOTE: If used incorrectly, this method can lead to errors if the
    /// board is put into an improper state.
    fn board_mut(&mut self) -> &mut [Self::Board; 2];

    /// Provides mutable access to the stores.
    fn stores_mut(&mut self) -> &mut [usize; 2];

    /// Provides mutable access to the current ply variable.
    fn ply_mut(&mut self) -> &mut usize;

    /// Provides mutable access to the current turn variable.
    fn current_turn_mut(&mut self) -> &mut Player;
}
