//use burn::data::dataset::Dataset;
use mancalamax::game::{DynGameState, GameState};
use mancalamax::game::{Mancala, Move, Player};
use mancalamax::minimax::{MinimaxBuilder, ParMinimaxBuilder};
//use mancalamax::ml::MancalaDataset;
use mancalamax::ui::{
    ExternalInterface, minimax_v_external, minimax_v_minimax, player_v_external, player_v_minimax,
    player_v_minimax_default, player_v_player_default,
};
use std::collections::HashMap;

fn main() {
    // TODO: We have an issue where if the same minimax object is reused with a different
    //       state that starts at a different hash, the table entries will all be invalid.
    //       Need some way to invalidate the TT, and clear it in that case.

    // TODO: Cases where we actually do see speedup:
    //       - default game state, depth 18, no ID, TT enabled, NO shared TT, search all, vary counts to get different speedup
    //       - default game state, depth 18, no ID, TT enabled, NO shared TT, search regular, vary counts to get different speedup
    //       - GS<36> & 12stones, depth 7 or 8, ID, TT enabled, shared TT, search all
    //       - - GS<36> & 12stones, depth 7 or 8, ID, TT enabled, shared TT, search regular

    // TODO: Cases where we do NOT see speedup:
    //       - default game state, depth 20, ID, TT enabled, shared TT, search regular

    // TODO: Mention that without a shared TT, the tables are removed after each ID iteration.
    //       This means ID is not really useful if the T table is not shared.
    //       Could be improved to combine the T Tables at the end, and re-distribute
    //       to states after each ID iteration.

    // TODO: Note the applications for machine learning for the parallelized version of max_value_all.

    // TODO: Note that we tried to evenly compare parallel & sequential implementations.
    //       (i.e., avoid disabling TT / ID on sequential when it's enabled on the parallel version.)

    // TODO: Mention that we didn't do the distributed T table because move ordering depends
    //       on access to table, so essentially too much overhead. More effective to do
    //       root-level splitting. Tested different depths, but all more overhead than worth it.

    // TODO: Note that the optimizations to minimax (alpha/beta, ID, TT) are so effective
    //       yet so sequential that with a small effective branching factor, it is nearly
    //       impossible to get speedup. Mainly a/b is what kills parallelism.
    //       This explains why max_value_all shows much more speedup. a/b is disabled.

    // TODO: Clean up, and just compare separate TTs vs. shared TT vs. fully sequential.
    // TODO: Then do VTune analysis of cache, how long waiting for stuff, etc. Make detailed.
    // TODO: Mention cases where it happens to be faster under these restricted circumstances.

    // TODO: Note that if shared table implementation were better, it might be possible
    //       to get some speedup. BUT -- NOW IT SEEMS LIKE MY SYSTEM IS ABOUT AS GOOD AS DASHMAP!

    // TODO: Make terminal functions able to take ParMinimaxBuilder objects.
    // TODO: Probably need a Minimax trait...

    const RUN: bool = true;
    const FILE: &str = "results_longer4.json";

    if RUN {
        let seq1 = MinimaxBuilder::new()
            .max_depth(Some(18))
            .iterative_deepening(false)
            .use_t_table(true);
        let par1 = ParMinimaxBuilder::new()
            .max_depth(Some(18))
            .iterative_deepening(false)
            .use_t_table(true)
            .shared_t_table(false);
        let seq2 = MinimaxBuilder::new()
            .max_depth(Some(8))
            .iterative_deepening(true)
            .use_t_table(true);
        let par2 = ParMinimaxBuilder::new()
            .max_depth(Some(8))
            .iterative_deepening(true)
            .use_t_table(true)
            .shared_t_table(true);
        let seq3 = MinimaxBuilder::new()
            .max_depth(Some(20))
            .iterative_deepening(true)
            .use_t_table(true);
        let par3 = ParMinimaxBuilder::new()
            .max_depth(Some(20))
            .iterative_deepening(true)
            .use_t_table(true)
            .shared_t_table(true);

        let state1 = GameState::default();
        let state2 = GameState::<36>::new(12, 0, 0, Player::One, 1, false);

        let mut results = HashMap::new();

        // Sequential version on default state (single move search).
        let m = seq1.build();
        println!("Started seq1 on state1...");
        let start = std::time::Instant::now();
        _ = m.search_utility(&state1);
        let end = std::time::Instant::now();
        results.insert(format!("seq1_{}", 1), end - start);

        // Parallel version on default state at varying thread counts (single move search).
        for t in 1..=6 {
            let m = par1.max_threads(t).build();
            println!("Started par1 on state1 (t={})...", t);
            let start = std::time::Instant::now();
            _ = m.search_utility(&state1);
            let end = std::time::Instant::now();
            results.insert(format!("par1_{}", t), end - start);
        }

        // Sequential version on default state (all move search).
        let m = seq1.build();
        println!("Started seq1_all on state1...");
        let start = std::time::Instant::now();
        _ = m.search_utility_all(&state1);
        let end = std::time::Instant::now();
        results.insert(format!("seq1_all_{}", 1), end - start);

        // Parallel version on default state at varying thread counts (all move search).
        for t in 1..=6 {
            let m = par1.max_threads(t).build();
            println!("Started par1_all on state1 (t={})...", t);
            let start = std::time::Instant::now();
            _ = m.search_utility_all(&state1);
            let end = std::time::Instant::now();
            results.insert(format!("par1_all_{}", t), end - start);
        }

        // Sequential version on expanded state (single move search).
        let m = seq2.build();
        println!("Started seq2 on state2...");
        let start = std::time::Instant::now();
        _ = m.search_utility(&state2);
        let end = std::time::Instant::now();
        results.insert(format!("seq2_{}", 1), end - start);

        // Parallel version on expanded state at varying thread counts (single move search).
        for t in 1..=36 {
            let m = par2.max_threads(t).build();
            println!("Started par2 on state2 (t={})...", t);
            let start = std::time::Instant::now();
            _ = m.search_utility(&state2);
            let end = std::time::Instant::now();
            results.insert(format!("par2_{}", t), end - start);
        }

        // Sequential version on expanded state (all move search).
        let m = seq2.build();
        println!("Started seq2_all on state2...");
        let start = std::time::Instant::now();
        _ = m.search_utility_all(&state2);
        let end = std::time::Instant::now();
        results.insert(format!("seq2_all_{}", 1), end - start);

        // Parallel version on expanded state at varying thread counts (all move search).
        for t in 1..=36 {
            let m = par2.max_threads(t).build();
            println!("Started par2_all on state2 (t={})...", t);
            let start = std::time::Instant::now();
            _ = m.search_utility_all(&state2);
            let end = std::time::Instant::now();
            results.insert(format!("par2_all_{}", t), end - start);
        }

        // Sequential version on default state, with ID + shared TT (single move search).
        let m = seq3.build();
        println!("Started seq3 on state1...");
        let start = std::time::Instant::now();
        _ = m.search_utility(&state1);
        let end = std::time::Instant::now();
        results.insert(format!("seq3_{}", 1), end - start);

        // Parallel version on default state, with ID + shared TT, at varying thread counts (single move search).
        for t in 1..=6 {
            let m = par3.max_threads(t).build();
            println!("Started par3 on state1 (t={})...", t);
            let start = std::time::Instant::now();
            _ = m.search_utility(&state1);
            let end = std::time::Instant::now();
            results.insert(format!("par3_{}", t), end - start);
        }

        // Save results using Serde.
        let file = std::fs::File::create(FILE).unwrap();
        let w = std::io::BufWriter::new(file);
        serde_json::ser::to_writer(w, &results).unwrap();
    }

    let contents = std::fs::read_to_string(FILE).unwrap();
    let results: HashMap<String, std::time::Duration> =
        serde_json::from_str(&contents).unwrap();

    // Print results for CSV.
    println!("n_threads,seq1,seq1_all,seq2,seq2_all,seq3,par1,par1_all,par2,par2_all,par3");
    print!("1,{},", results.get("seq1_1").unwrap().as_secs_f32());
    print!("{},", results.get("seq1_all_1").unwrap().as_secs_f32());
    print!("{},", results.get("seq2_1").unwrap().as_secs_f32());
    print!("{},", results.get("seq2_all_1").unwrap().as_secs_f32());
    print!("{},", results.get("seq3_1").unwrap().as_secs_f32());
    print!("{},", results.get("par1_1").unwrap().as_secs_f32());
    print!("{},", results.get("par1_all_1").unwrap().as_secs_f32());
    print!("{},", results.get("par2_1").unwrap().as_secs_f32());
    print!("{},", results.get("par2_all_1").unwrap().as_secs_f32());
    println!("{}", results.get("par3_1").unwrap().as_secs_f32());
    for t in 2..=36 {
        if t <= 6 {
            print!("{},,,,,,{},", t, results.get(&format!("par1_{}", t)).unwrap().as_secs_f32());
            print!("{},", results.get(&format!("par1_all_{}", t)).unwrap().as_secs_f32());
            print!("{},", results.get(&format!("par2_{}", t)).unwrap().as_secs_f32());
            print!("{},", results.get(&format!("par2_all_{}", t)).unwrap().as_secs_f32());
            println!("{}", results.get(&format!("par3_{}", t)).unwrap().as_secs_f32());
        } else {
            print!("{},,,,,,,,{},", t, results.get(&format!("par2_{}", t)).unwrap().as_secs_f32());
            println!("{},", results.get(&format!("par2_all_{}", t)).unwrap().as_secs_f32());
        }
    }

    //player_v_player_default();
    //player_v_minimax_default(Player::One);

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
