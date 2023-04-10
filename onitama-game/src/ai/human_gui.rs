use crate::game::{done_move::DoneMove, game_state::GameState};

use super::agent::Agent;

#[derive(Clone)]
pub struct HumanGui;

impl Agent for HumanGui {
    fn generate_move(&self, _game_state: &GameState) -> (DoneMove, f64) {
        // Human agent will be playing with using the GUI
        unimplemented!();
    }

    fn name(&self) -> &'static str {
        "Human"
    }

    fn clone_dyn(&self) -> Box<dyn Agent> {
        Box::new(self.clone())
    }
}
