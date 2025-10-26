use burn::data::dataset::Dataset;
use mancalamax::game::{DynGameState, GameState};
use mancalamax::game::{Mancala, Move, Player};
use mancalamax::minimax::MinimaxBuilder;
use mancalamax::ml::MancalaDataset;
use mancalamax::ui::{player_v_minimax, player_v_minimax_default, player_v_player_default};

fn main() {
    //player_v_player_default();
    //player_v_minimax_default(Player::One);
    //let minimax = MinimaxBuilder::new().max_depth(Some(18));
    //player_v_minimax(&GameState::default(), &minimax, Player::One);
    //mancalamax::ui::gui::make_gui();
    //println!("{:?}", GameState::default().valid_moves());

    let result = MancalaDataset::generate_default(70, 500).deduplicated();
    println!("{}", result.len());
    println!("{:?}", result.data()[11]);

    result.save_csv("mancala.csv").expect("Could not save csv");

    let z = MancalaDataset::from_csv("mancala.csv").unwrap();
    println!("{}", z.data().len());
    println!("{:?}", z.data()[11]);
}
