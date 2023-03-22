use crate::game::{done_move::DoneMove, player_color::PlayerColor, state::State};

use super::agent::Agent;

pub struct HumanGui;

impl Agent for HumanGui {
    fn generate_move(&self, state: &State, player_color: PlayerColor) -> DoneMove {
        unimplemented!();
    }
}
