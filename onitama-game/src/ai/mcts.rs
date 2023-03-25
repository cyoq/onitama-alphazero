use std::time::Duration;

use crate::game::{done_move::DoneMove, game_state::GameState};

use super::agent::Agent;

#[derive(Clone)]
pub struct Mcts {
    pub search_time: Duration,
    pub c: f64,
}

impl Default for Mcts {
    fn default() -> Self {
        Self {
            search_time: Duration::from_secs(1),
            c: (2f64).sqrt(),
        }
    }
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
