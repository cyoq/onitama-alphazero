use crate::game::{r#move::Move, state::State};

pub trait Agent {
    fn generate_move(&self, state: &State) -> Option<Move>;
}
