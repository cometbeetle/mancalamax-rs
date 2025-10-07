use mancalamax::game::{DynGameState, GameState};
use mancalamax::game::{Mancala, Player};
use mancalamax::minimax::MinimaxBuilder;
use mancalamax::ui::{player_v_minimax, player_v_minimax_default, player_v_player_default};

fn main() {
    //player_v_player_default();
    //player_v_minimax_default(Player::One);
    //let minimax = MinimaxBuilder::new().max_depth(Some(18));
    //player_v_minimax(&GameState::default(), &minimax, Player::One);
    mancalamax::ui::gui::make_gui();
    //println!("{:?}", GameState::default().valid_moves());
}
