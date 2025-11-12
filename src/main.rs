use burn::data::dataset::Dataset;
use mancalamax::game::{DynGameState, GameState};
use mancalamax::game::{Mancala, Move, Player};
use mancalamax::minimax::MinimaxBuilder;
use mancalamax::ml::MancalaDataset;
use mancalamax::ui::{
    ExternalInterface, minimax_v_external, minimax_v_minimax, player_v_external, player_v_minimax,
    player_v_minimax_default, player_v_player_default,
};

fn main() {
    //player_v_player_default();
    //player_v_minimax_default(Player::One);
    //let minimax = MinimaxBuilder::new().max_depth(Some(18));
    //player_v_minimax(&GameState::default(), &minimax, Player::One);
    //mancalamax::ui::gui::make_gui();
    //println!("{:?}", GameState::default().valid_moves());

    //let result = MancalaDataset::generate_default(70, 100000).deduplicated();
    //println!("{}", result.len());

    // Test CSV functionality.
    //result.save_csv("mancala.csv").expect("Could not save csv");

    //player_v_external(
    //    &GameState::default(),
    //    Player::Two,
    //    ExternalInterface::Minimal,
    //    "C:\\Users\\ethan\\Desktop\\test_dir",
    //);

    //minimax_v_external(
    //    &GameState::default(),
    //    &MinimaxBuilder::default().max_depth(Some(8)),
    //    Player::Two,
    //    ExternalInterface::Minimal,
    //    "C:\\Users\\ethan\\Desktop\\test_dir",
    //);

    let mut gnn_wins = Vec::new();
    let mut minimax_wins = Vec::new();
    for _ in 0..500 {
        let s = minimax_v_external(
            &GameState::default(),
            &MinimaxBuilder::new().max_depth(Some(0)),
            Player::One,
            ExternalInterface::Minimal,
            "C:\\Users\\ethan\\Desktop\\test_dir",
        );
        gnn_wins.push(s.score(Player::One));

        let s = minimax_v_minimax(
            &GameState::default(),
            &MinimaxBuilder::new()
                .optimize_for(Player::One)
                .max_depth(Some(12)),
            &MinimaxBuilder::new()
                .optimize_for(Player::Two)
                .max_depth(Some(0)),
        );
        minimax_wins.push(s.score(Player::One));
    }

    println!("GNN WINS: {:?}", gnn_wins);
    println!("MINIMAX-12 WINS: {:?}", minimax_wins)
}

// TODO: Maybe, we should have the datasets just return Tensors instead of individual example structs.
// TODO: Might make more efficient for training? Focus on Python for now though.

// TODO: OR - BETTER IDEA - Have separate struct that is a dataset that is actually
//       ready for training (i.e., one made of tensors, proper bord reordering, etc.)

// TODO: Use Polars to handle the CSV writing / maybe some dataset management.
