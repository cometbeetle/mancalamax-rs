//! The `mancalamax` crate provides several structs, enums, and traits necessary
//! to play Mancala using a computer.
//!
//! `[DOCS IN PROGRESS]`

pub mod game;
pub mod minimax;
#[cfg(feature = "ml")]
pub mod ml;
pub mod ui;

pub mod net;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example() {}
}
