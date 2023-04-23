pub mod mcts_arena;

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::game::{done_move::DoneMove, game_state::GameState};

use self::mcts_arena::MctsArena;

use super::agent::Agent;

#[derive(Clone, Serialize, Deserialize)]
pub struct Mcts {
    pub search_time: Duration,
    pub min_node_visits: u32,
    pub exploration_c: f32,
    pub max_playouts: u32,
}

impl Default for Mcts {
    fn default() -> Self {
        Self {
            search_time: Duration::from_secs(1),
            exploration_c: 2f32.sqrt(),
            min_node_visits: 5,
            max_playouts: 5000,
        }
    }
}

impl Agent for Mcts {
    fn generate_move(&self, game_state: &GameState) -> (DoneMove, f64) {
        let mut arena = MctsArena::new(
            game_state.state.clone(),
            self.search_time,
            game_state.curr_player_color,
            self.min_node_visits,
            self.exploration_c,
            self.max_playouts,
        );
        let mov = arena.search();

        // println!("Tree: {}", arena.debug_tree());
        println!("Playouts: {}", arena.playouts);
        mov
    }

    fn name(&self) -> &'static str {
        "MCTS AI"
    }

    fn clone_dyn(&self) -> Box<dyn Agent> {
        Box::new(self.clone())
    }

    fn id(&self) -> u64 {
        self.search_time.as_nanos() as u64
            + self.exploration_c as u64
            + self.max_playouts as u64
            + self.min_node_visits as u64
    }
}
