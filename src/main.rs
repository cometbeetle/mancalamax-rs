//use burn::data::dataset::Dataset;
use mancalamax::game::{DynGameState, GameState};
use mancalamax::game::{Mancala, Move, Player};
use mancalamax::minimax::{MinimaxBuilder, ParMinimaxBuilder};
//use mancalamax::ml::MancalaDataset;
use mancalamax::ui::{
    ExternalInterface, minimax_v_external, minimax_v_minimax, player_v_external, player_v_minimax,
    player_v_minimax_default, player_v_player_default,
};

fn main() {
    // TODO: We have an issue where if the same minimax object is reused with a different
    //       state that starts at a different hash, the table entries will all be invalid.
    //       Need some way to invalidate the TT, and clear it in that case.

    //player_v_player_default();
    //player_v_minimax_default(Player::One);
    let par_minimax = ParMinimaxBuilder::new()
        .max_depth(Some(7))
        .iterative_deepening(true)
        .use_t_table(true)
        .max_time(None)
        .t_table_buckets(4096)
        .shared_t_table(true);
    let minimax = MinimaxBuilder::new()
        .max_depth(Some(7))
        .iterative_deepening(true)
        .use_t_table(true)
        .max_time(None);
    let start = std::time::Instant::now();
    //minimax_v_minimax(
    //    &GameState::<6>::new(16, 0, 0, Player::One, 0, false),
    //    &minimax,
    //    &minimax.optimize_for(Player::Two),
    //);
    let result = par_minimax
        .build()
        .search_utility_all(&GameState::<36>::new(12, 0, 0, Player::One, 0, false));
    //let result = minimax.build().search_utility(&GameState::default());
    println!("{:?}", result);
    let end = std::time::Instant::now();
    println!("Parallel: {} s", (end - start).as_secs_f32());

    let start = std::time::Instant::now();
    //minimax_v_minimax(
    //    &GameState::<6>::new(16, 0, 0, Player::One, 0, false),
    //    &minimax,
    //    &minimax.optimize_for(Player::Two),
    //);
    let result =
        minimax
            .build()
            .search_utility_all(&GameState::<36>::new(12, 0, 0, Player::One, 0, false));
    //let result = minimax.build().search_utility(&GameState::default());
    println!("{:?}", result);
    let end = std::time::Instant::now();
    println!("Sequential: {} s", (end - start).as_secs_f32());

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

    /*
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
    */
}

// TODO: Maybe, we should have the datasets just return Tensors instead of individual example structs.
// TODO: Might make more efficient for training? Focus on Python for now though.

// TODO: OR - BETTER IDEA - Have separate struct that is a dataset that is actually
//       ready for training (i.e., one made of tensors, proper bord reordering, etc.)

// TODO: Use Polars to handle the CSV writing / maybe some dataset management.
