use std::time::Duration;

use crate::game::{done_move::DoneMove, game::GameState};

use super::agent::Agent;

#[derive(Clone)]
pub struct Mcts {
    pub search_time: Duration,
}

impl Agent for Mcts {
    fn generate_move(&self, game_state: &GameState) -> DoneMove {
        todo!()
    }

    fn name(&self) -> &'static str {
        "MCTS AI"
    }

    fn clone_dyn(&self) -> Box<dyn Agent> {
        Box::new(self.clone())
    }
}
