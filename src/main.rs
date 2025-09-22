use mancalamax::game::GameState;
use mancalamax::game::Player;
use mancalamax::ui::{player_v_minimax_default, player_v_player_default};

fn main() {
    //player_v_player_default();
    player_v_minimax_default(Player::One);
}
