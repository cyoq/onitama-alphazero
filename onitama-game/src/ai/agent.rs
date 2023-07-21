use std::hash::Hasher;

use erased_serde::serialize_trait_object;

use crate::game::{done_move::DoneMove, game_state::GameState};

pub trait Agent: Send + erased_serde::Serialize {
    /// Returns best move and a score for it
    fn generate_move(&self, game_state: &GameState) -> (DoneMove, f64);

    fn name(&self) -> &'static str;

    // To clone the agent, it requires quite awful construction: https://stackoverflow.com/a/69891769
    fn clone_dyn(&self) -> Box<dyn Agent>;

    fn id(&self) -> u64;
}

impl Clone for Box<dyn Agent> {
    fn clone(&self) -> Self {
        self.clone_dyn()
    }
}

impl std::hash::Hash for Box<dyn Agent> {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.id().hash(state)
    }
}

serialize_trait_object!(Agent);
