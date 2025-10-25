use mancalamax::game::{DynGameState, GameState};
use mancalamax::game::{Mancala, Move, Player};
use mancalamax::minimax::MinimaxBuilder;
use mancalamax::ml::datagen::generate_dataset_default;
use mancalamax::ui::{player_v_minimax, player_v_minimax_default, player_v_player_default};

fn main() {
    //player_v_player_default();
    //player_v_minimax_default(Player::One);
    //let minimax = MinimaxBuilder::new().max_depth(Some(18));
    //player_v_minimax(&GameState::default(), &minimax, Player::One);
    //mancalamax::ui::gui::make_gui();
    //println!("{:?}", GameState::default().valid_moves());
    //type B = burn::backend::Cuda;
    type B = burn::backend::NdArray;
    let result = generate_dataset_default::<B>(70, 100, true);
    for e in result {
        for i in e {
            let i: f32 = i;
            print!("{} ", i);
        }
        println!();
    }
}
