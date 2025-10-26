use burn::data::dataset::Dataset;
use mancalamax::game::{DynGameState, GameState};
use mancalamax::game::{Mancala, Move, Player};
use mancalamax::minimax::MinimaxBuilder;
use mancalamax::ml::MancalaDataset;
use mancalamax::ui::{player_v_minimax, player_v_minimax_default, player_v_player_default};
use serde_json5::{from_reader, to_writer};
use std::fs::File;
use std::io::{BufReader, BufWriter};

fn main() {
    //player_v_player_default();
    //player_v_minimax_default(Player::One);
    //let minimax = MinimaxBuilder::new().max_depth(Some(18));
    //player_v_minimax(&GameState::default(), &minimax, Player::One);
    //mancalamax::ui::gui::make_gui();
    //println!("{:?}", GameState::default().valid_moves());

    let result = MancalaDataset::generate_default(70, 100).deduplicated();
    println!("{}", result.len());
    println!("{:?}", result.data()[11]);

    // Test CSV functionality.
    result.save_csv("mancala.csv").expect("Could not save csv");
    let z = MancalaDataset::from_csv("mancala.csv").expect("Could not load mancala.csv");
    println!("{}", z.data().len());
    println!("{:?}", z.data()[11]);

    // Test serde_json5
    let file = File::create("mancala.json").expect("Could not create file");
    let writer = BufWriter::new(file);
    to_writer(writer, &result).expect("Could not save json");
    let file = File::open("mancala.json").expect("Could not load mancala.json");
    let reader = BufReader::new(file);
    let q: MancalaDataset<DynGameState> = from_reader(reader).expect("Could not deserialize json");
    println!("{}", q.data().len());
    println!("{:?}", q.data()[11]);
}
