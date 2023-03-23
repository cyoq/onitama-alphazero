use std::time::Duration;

use crate::game::{done_move::DoneMove, player_color::PlayerColor, state::State};

use super::agent::Agent;

pub struct Mcts {
    pub search_time: Duration,
}

impl Agent for Mcts {
    fn generate_move(&self, state: &State, player_color: PlayerColor) -> DoneMove {
        todo!()
    }

    fn name(&self) -> &'static str {
        "MCTS AI"
    }
}
