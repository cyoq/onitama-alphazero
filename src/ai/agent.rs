use crate::game::{done_move::DoneMove, player_color::PlayerColor, state::State};

pub trait Agent {
    fn generate_move(&self, state: &State, player_color: PlayerColor) -> DoneMove;
}
