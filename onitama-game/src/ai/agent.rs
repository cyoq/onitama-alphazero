use crate::game::{done_move::DoneMove, game_state::GameState};

pub trait Agent: Send {
    /// Returns best move and a score for it
    fn generate_move(&self, game_state: &GameState) -> (DoneMove, f64);

    fn name(&self) -> &'static str;

    // To clone the agent, it requires quite awful construction: https://stackoverflow.com/a/69891769
    fn clone_dyn(&self) -> Box<dyn Agent>;
}

impl Clone for Box<dyn Agent> {
    fn clone(&self) -> Self {
        self.clone_dyn()
    }
}
