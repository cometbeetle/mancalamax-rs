use mancalamax_rs::game::GameState;
use mancalamax_rs::game::Player;
use mancalamax_rs::ui::{player_v_minimax_default, player_v_player_default};

fn main() {
    //player_v_player_default();
    player_v_minimax_default(Player::One);
}
