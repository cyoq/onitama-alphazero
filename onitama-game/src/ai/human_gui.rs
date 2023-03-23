use crate::game::{done_move::DoneMove, player_color::PlayerColor, state::State};

use super::agent::Agent;

pub struct HumanGui;

impl Agent for HumanGui {
    fn generate_move(&self, _state: &State, _player_color: PlayerColor) -> DoneMove {
        // Human agent will be playing by using the GUI
        unimplemented!();
    }
}
