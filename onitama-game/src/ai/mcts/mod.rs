pub mod mcts_arena;

use std::time::Duration;

use crate::game::{done_move::DoneMove, game_state::GameState};

use self::mcts_arena::MctsArena;

use super::agent::Agent;

#[derive(Clone)]
pub struct Mcts {
    pub search_time: Duration,
    pub min_node_visits: u32,
    pub exploration_c: f32,
}

impl Default for Mcts {
    fn default() -> Self {
        Self {
            search_time: Duration::from_secs(1),
            exploration_c: 2f32.sqrt(),
            min_node_visits: 5,
        }
    }
}

impl Agent for Mcts {
    fn generate_move(&self, game_state: &GameState) -> DoneMove {
        let mut arena = MctsArena::new(
            game_state.state.clone(),
            self.search_time,
            game_state.curr_player_color,
            self.min_node_visits,
            self.exploration_c,
        );
        let mov = arena.search();
        // println!("Tree: {}", arena.debug_tree());
        println!("Playouts: {}", arena.playouts);
        println!("Avg: {}", arena.arena[0].winrate);
        mov
    }

    fn name(&self) -> &'static str {
        "MCTS AI"
    }

    fn clone_dyn(&self) -> Box<dyn Agent> {
        Box::new(self.clone())
    }
}
