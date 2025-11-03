//! Components for generating synthetic datasets for machine learning tasks.

use crate::game::{DynGameState, GameState, Mancala, Move, Player};
use crate::minimax::MinimaxBuilder;
use burn::data::dataset::Dataset;
use csv::{Reader, Writer};
use rayon::prelude::*;
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::path::Path;

fn len_to_pits(len: usize) -> usize {
    (len - 5) / 3
}

fn pits_to_len(n_pits: usize) -> usize {
    3 * n_pits + 5
}

/// Represents a single training example in a dataset.
///
/// Can be converted to a [`Vec<f32>`] with components in the following order:
/// * Store 1 (number of stones)
/// * Store 2 (number of stones)
/// * Player 1, all pits (number of stones)
/// * Player 2, all pits (number of stones)
/// * Current turn (`1` or `2`)
/// * Current ply (at least `0`)
/// * Whether Player 2 has moved (`1` or `0`)
/// * Utilities for all moves (order: Swap, Pit 1, Pit 2, ...)
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MancalaExample<T: Mancala> {
    state: T,
    utilities: Vec<(Move, f32)>,
}

impl<T: Mancala> Hash for MancalaExample<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.state.hash(state);
        for (m, u) in &self.utilities {
            m.hash(state);
            u.to_bits().hash(state);
        }
    }
}

impl<T: Mancala> PartialEq for MancalaExample<T> {
    /// For the purposes of the dataset, we compare the bits of the
    /// [`f32`] utilities, allow `NaN` values to equal each other.
    fn eq(&self, other: &Self) -> bool {
        if self.state != other.state {
            return false;
        }
        if other.utilities.len() != self.utilities.len() {
            return false;
        }
        for (i, (m1, u1)) in self.utilities.iter().enumerate() {
            let (m2, u2) = &other.utilities[i];
            if u1.to_bits() != u2.to_bits() || *m1 != *m2 {
                return false;
            }
        }
        true
    }
}

impl<T: Mancala> Eq for MancalaExample<T> {}

impl<T: Mancala> From<&MancalaExample<T>> for Vec<f32> {
    fn from(value: &MancalaExample<T>) -> Self {
        let mut result: Vec<f32> = Vec::with_capacity(pits_to_len(value.state.pits()));

        // Push stores.
        for i in value.state.stores() {
            result.push(*i as f32);
        }

        // Push board.
        for b in value.state.board() {
            for i in b.as_ref() {
                result.push(*i as f32);
            }
        }

        // Push current turn.
        result.push(usize::from(value.state.current_turn()) as f32);

        // Push current ply.
        result.push(value.state.ply() as f32);

        // Push value indicating whether Player 2 has moved.
        result.push(value.state.p2_moved() as usize as f32);

        // Push values for utility on each move.
        let mut utils = vec![f32::NEG_INFINITY; value.state.pits() + 1];
        for (m, u) in &value.utilities {
            utils[usize::from(*m)] = *u;
        }

        result.extend(utils);
        result
    }
}

impl<T: Mancala> From<MancalaExample<T>> for Vec<f32> {
    fn from(value: MancalaExample<T>) -> Self {
        Self::from(&value)
    }
}

impl From<Vec<f32>> for MancalaExample<DynGameState> {
    fn from(value: Vec<f32>) -> Self {
        let n_pits = len_to_pits(value.len());
        let (store_1, store_2) = (value[0] as usize, value[1] as usize);
        let player1: Vec<usize> = value[2..2 + n_pits].iter().map(|x| *x as usize).collect();
        let player2: Vec<usize> = value[2 + n_pits..2 + 2 * n_pits]
            .iter()
            .map(|x| *x as usize)
            .collect();
        let players = vec![player1, player2];
        let turn: Player = (value[2 + 2 * n_pits] as usize).into();
        let ply = value[3 + 2 * n_pits] as usize;
        let p2_moved = value[4 + 2 * n_pits] != 0.0;
        let mut utilities: Vec<(Move, f32)> = value[6 + 2 * n_pits..]
            .iter()
            .enumerate()
            .map(|(m, u)| ((m + 1).into(), *u))
            .collect();
        utilities.insert(0, (Move::Swap, value[5 + 2 * n_pits]));

        Self {
            state: DynGameState::from_vec(&players, store_1, store_2, turn, ply, p2_moved),
            utilities,
        }
    }
}

impl<T: Mancala> MancalaExample<T> {
    /// Create a new Mancala example given a game state and vector of
    /// (move, utility) pairs.
    ///
    /// If the `utilities` parameter does not include a (move, utility)
    /// pair for every possible move, then, based on the supplied state, additional
    /// pairs will be added with utilities of negative infinity.
    ///
    /// Note that this means the [`serde_json`] crate will fail to properly
    /// serialize / deserialize examples, since standard JSON cannot handle
    /// infinite float values. Use [`serde_json5`] for a functional equivalent.
    pub fn new(state: T, utilities: Vec<(Move, f32)>) -> Self {
        let mut expanded_utils: Vec<(Move, f32)> = utilities;
        for i in 0..=state.pits() {
            if expanded_utils.iter().all(|(m, _)| usize::from(*m) != i) {
                expanded_utils.push((Move::from(i), f32::NEG_INFINITY));
            }
        }
        Self {
            state,
            utilities: expanded_utils,
        }
    }

    /// Provides a reference to the game state stored in the example.
    pub fn state(&self) -> &T {
        &self.state
    }

    /// Provides a reference to the (move, utility) pairs vector in the example.
    pub fn utilities(&self) -> &Vec<(Move, f32)> {
        &self.utilities
    }

    /// Convert the example into a vector, as described in [`MancalaExample`].
    pub fn make_vec(&self) -> Vec<f32> {
        self.into()
    }
}

/// Dataset for storing synthetic Mancala game data.
///
/// Can be converted to a [`Vec<Vec<f32>>`].
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MancalaDataset<T: Mancala> {
    data: Vec<MancalaExample<T>>,
}

impl<T: Mancala> Dataset<MancalaExample<T>> for MancalaDataset<T> {
    fn get(&self, index: usize) -> Option<MancalaExample<T>> {
        self.data.get(index).cloned()
    }
    fn len(&self) -> usize {
        self.data.len()
    }
}

impl<T: Mancala> From<MancalaDataset<T>> for Vec<Vec<f32>> {
    fn from(value: MancalaDataset<T>) -> Self {
        Vec::from_iter(value.data.iter().map(|m| Vec::from(m)))
    }
}

impl From<Vec<Vec<f32>>> for MancalaDataset<DynGameState> {
    fn from(value: Vec<Vec<f32>>) -> Self {
        MancalaDataset {
            data: Vec::from_iter(value.into_iter().map(|m| m.into())),
        }
    }
}

impl<T: Mancala> MancalaDataset<T> {
    /// Create a new Mancala dataset given a data vector.
    pub fn new(data: Vec<MancalaExample<T>>) -> Self {
        Self { data }
    }

    /// Provides a reference to the data vector in the dataset.
    pub fn data(&self) -> &Vec<MancalaExample<T>> {
        &self.data
    }

    /// Returns the number of pits from the first example.
    ///
    /// If the data are properly formatted, this should also be the
    /// number of pits for *every* example in the dataset.
    pub fn pits(&self) -> usize {
        assert!(
            self.data.len() > 0,
            "Cannot determine number of pits from empty dataset"
        );
        self.data[0].state.pits()
    }
}

impl MancalaDataset<GameState<6>> {
    /// Generate a dataset of random Mancala games and associated move utilities,
    /// based on the configuration specified by [`MinimaxBuilder::default`].
    ///
    /// See also: [`MancalaDataset::generate`].
    pub fn generate_default(max_moves: usize, runs: usize) -> Self {
        Self::generate(&MinimaxBuilder::new(), max_moves, runs)
    }
}

impl<const N: usize> MancalaDataset<GameState<N>> {
    /// Generate a dataset of random Mancala games and associated move utilities.
    ///
    /// Minimax will be run for games generated by making random moves between
    /// 0 and `max_moves` times. This generation is then repeated `runs` times,
    /// parallelized for efficiency.
    ///
    /// Note that this is implemented only for [`MancalaDataset<GameState<N>>`]
    /// to ensure minimax runs on statically allocated game state structs.
    pub fn generate(minimax: &MinimaxBuilder<GameState<N>>, max_moves: usize, runs: usize) -> Self {
        let generate = || {
            let mut data: Vec<MancalaExample<GameState<N>>> = Vec::new();
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

                data.push(MancalaExample::new(state, utilities));

                n_moves += 1;
            }
            data
        };

        let result: Vec<MancalaExample<GameState<N>>> = (0..runs)
            .into_par_iter()
            .map(|_| generate())
            .flatten()
            .collect();

        Self { data: result }
    }
}

impl<T: Mancala> MancalaDataset<T> {
    /// Consume and return the current dataset without duplicates.
    pub fn deduplicated(mut self) -> Self {
        let mut seen = HashSet::new();
        let mut unique = Vec::new();

        for item in self.data {
            if seen.insert(item.clone()) {
                unique.push(item);
            }
        }

        self.data = unique;
        self
    }

    /// Save the current dataset to a CSV file, based on the format described
    /// in [`MancalaExample`].
    pub fn save_csv<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let mut wtr = Writer::from_writer(File::create(path)?);

        let mut header: Vec<String> = vec!["store1".into(), "store2".into()];
        for i in 1..=2 {
            for p in 1..=self.pits() {
                header.push(format!("player{}p{}", i, p));
            }
        }
        header.extend(vec![
            "turn".into(),
            "ply".into(),
            "p2_moved".into(),
            "util_swap".into(),
        ]);
        for p in 1..=self.pits() {
            header.push(format!("util_{}", p));
        }

        wtr.write_record(header)?;

        for row in &self.data {
            let string_row: Vec<String> = row.make_vec().iter().map(|x| x.to_string()).collect();
            wtr.write_record(&string_row)?;
        }

        // Flush the writer to ensure all data is written
        wtr.flush()?;
        Ok(())
    }
}

impl MancalaDataset<DynGameState> {
    /// Construct a Mancala dataset from a CSV file.
    pub fn from_csv<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let mut rdr = Reader::from_reader(File::open(path)?);
        let mut data: Vec<Vec<f32>> = Vec::new();

        for record in rdr.records() {
            let row: Vec<f32> = record?
                .iter()
                .map(|s| s.parse::<f32>().unwrap_or(f32::NAN))
                .collect();
            data.push(row);
        }

        Ok(Self::from(data))
    }
}
