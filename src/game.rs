//! Implements the core game logic and state management.

use std::fmt::{Display, Formatter};

// --------------------------------------------
// ------------------ TRAITS ------------------
// --------------------------------------------

pub(crate) trait Mancala<B>: Clone + Display
where
    B: AsRef<[usize]>,
{
    fn board(&self) -> &[B; 2];
    fn board_as_vecs(&self) -> [Vec<usize>; 2] {
        [
            self.board()[0].as_ref().to_vec(),
            self.board()[1].as_ref().to_vec(),
        ]
    }
    fn stores(&self) -> &[usize; 2];
    fn ply(&self) -> usize;
    fn current_turn(&self) -> usize;
    fn is_over(&self) -> bool;
    fn score(&self, player: usize) -> usize;
    fn switch_turn(&mut self) -> usize;
    fn rotate_board(&mut self);
    fn valid_moves(&self, player: usize) -> Vec<usize>;
    fn make_move(&mut self, pit: usize);
}

// ---------------------------------------------
// ------------------ STRUCTS ------------------
// ---------------------------------------------

#[derive(Debug, Clone, Copy)]
pub(crate) struct GameState<const N: usize> {
    pub(crate) board: [[usize; N]; 2],
    pub(crate) stores: [usize; 2],
    pub(crate) ply: usize,
    pub(crate) current_turn: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct DynGameState {
    pub(crate) board: [Vec<usize>; 2],
    pub(crate) stores: [usize; 2],
    pub(crate) ply: usize,
    pub(crate) current_turn: usize,
}

// ----------------------------------------------------
// ------------------ IMPLEMENTATION ------------------
// ----------------------------------------------------

fn fmt_common<T, B>(f: &mut Formatter, state: &T, dynamic: bool) -> std::fmt::Result
where
    T: Mancala<B>,
    B: AsRef<[usize]>,
{
    let title = if dynamic {
        "Dynamic GameState"
    } else {
        "Static GameState"
    };
    writeln!(f, "Bird's-Eye View of {} {:p}", title, state)?;
    writeln!(f, "==============================================")?;

    let p1_select = if state.current_turn() == 0 { '*' } else { ' ' };
    writeln!(f, "{} P1:  ({:02})  [ ", p1_select, state.stores()[0])?;
    for pit in state.board()[0].as_ref() {
        write!(f, "{:02} ", pit)?;
    }
    writeln!(f, "]")?;

    let p2_select = if state.current_turn() == 1 { '*' } else { ' ' };
    writeln!(f, "{} P2:        [ ", p2_select)?;
    for pit in state.board()[1].as_ref() {
        write!(f, "{:02} ", pit)?;
    }
    writeln!(f, "]  ({:02})", state.stores()[1])?;
    writeln!(f, "Move Number: {}", state.ply())?;
    Ok(())
}

impl<const N: usize> Display for GameState<N> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        fmt_common(f, self, false)
    }
}

impl Default for GameState<6> {
    fn default() -> Self {
        Self {
            board: [[4; 6]; 2],
            stores: [0, 0],
            ply: 0,
            current_turn: 0,
        }
    }
}

impl<const N: usize> Mancala<[usize; N]> for GameState<N> {
    fn board(&self) -> &[[usize; N]; 2] {
        todo!()
    }

    fn stores(&self) -> &[usize; 2] {
        todo!()
    }

    fn ply(&self) -> usize {
        todo!()
    }

    fn current_turn(&self) -> usize {
        todo!()
    }

    fn is_over(&self) -> bool {
        todo!()
    }

    fn score(&self, player: usize) -> usize {
        todo!()
    }

    fn switch_turn(&mut self) -> usize {
        todo!()
    }

    fn rotate_board(&mut self) {
        todo!()
    }

    fn valid_moves(&self, player: usize) -> Vec<usize> {
        todo!()
    }

    fn make_move(&mut self, pit: usize) {
        todo!()
    }
}

impl<const N: usize> GameState<N> {
    pub(crate) fn new(
        stones_per: usize,
        store_1: usize,
        store_2: usize,
        current_turn: usize,
        ply: usize,
    ) -> Self {
        Self {
            board: [[stones_per; N]; 2],
            stores: [store_1, store_2],
            ply,
            current_turn,
        }
    }
    pub(crate) fn from_vec(
        board: Vec<Vec<usize>>,
        store_1: usize,
        store_2: usize,
        current_turn: usize,
        ply: usize,
    ) -> Self {
        todo!()
    }
    pub(crate) fn from_arr(
        board: [[usize; N]; 2],
        store_1: usize,
        store_2: usize,
        current_turn: usize,
        ply: usize,
    ) -> Self {
        Self {
            board,
            stores: [store_1, store_2],
            ply,
            current_turn,
        }
    }
}

impl Display for DynGameState {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        fmt_common(f, self, true)
    }
}

impl Default for DynGameState {
    fn default() -> Self {
        Self {
            board: [vec![4; 6], vec![4; 6]],
            stores: [0, 0],
            ply: 0,
            current_turn: 0,
        }
    }
}

impl Mancala<Vec<usize>> for DynGameState {
    fn board(&self) -> &[Vec<usize>; 2] {
        todo!()
    }

    fn stores(&self) -> &[usize; 2] {
        todo!()
    }

    fn ply(&self) -> usize {
        todo!()
    }

    fn current_turn(&self) -> usize {
        todo!()
    }

    fn is_over(&self) -> bool {
        todo!()
    }

    fn score(&self, player: usize) -> usize {
        todo!()
    }

    fn switch_turn(&mut self) -> usize {
        todo!()
    }

    fn rotate_board(&mut self) {
        todo!()
    }

    fn valid_moves(&self, player: usize) -> Vec<usize> {
        todo!()
    }

    fn make_move(&mut self, pit: usize) {
        todo!()
    }
}

impl DynGameState {
    pub(crate) fn new(
        pits: usize,
        stones_per: usize,
        store_1: usize,
        store_2: usize,
        current_turn: usize,
        ply: usize,
    ) -> Self {
        Self {
            board: [vec![stones_per; pits], vec![stones_per; pits]],
            stores: [store_1, store_2],
            ply,
            current_turn,
        }
    }
    pub(crate) fn from_vec(
        board: Vec<Vec<usize>>,
        store_1: usize,
        store_2: usize,
        current_turn: usize,
        ply: usize,
    ) -> Self {
        todo!()
    }
    pub(crate) fn from_arr<const N: usize>(
        board: [[usize; N]; 2],
        store_1: usize,
        store_2: usize,
        current_turn: usize,
        ply: usize,
    ) -> Self {
        Self {
            board: [board[0].to_vec(), board[1].to_vec()],
            stores: [store_1, store_2],
            ply,
            current_turn,
        }
    }
}
