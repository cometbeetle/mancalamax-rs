use mancalamax::game::Player;
use mancalamax::game::{DynGameState, GameState};
use mancalamax::ui::{player_v_minimax_default, player_v_player_default};

fn main() {
    //player_v_player_default();
    //player_v_minimax_default(Player::One);
    mancalamax::ui::gui::make_gui();
}
