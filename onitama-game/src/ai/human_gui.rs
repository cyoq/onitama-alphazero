use crate::game::{done_move::DoneMove, game::GameState};

use super::agent::Agent;

#[derive(Clone)]
pub struct HumanGui;

impl Agent for HumanGui {
    fn generate_move(&self, _game_state: &GameState) -> DoneMove {
        // Human agent will be playing by using the GUI
        unimplemented!();
    }

    fn name(&self) -> &'static str {
        "Human"
    }

    fn clone_dyn(&self) -> Box<dyn Agent> {
        Box::new(self.clone())
    }
}
