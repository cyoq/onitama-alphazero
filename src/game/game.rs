use crate::ai::agent::Agent;

use super::{player_color::PlayerColor, state::State};

pub struct Game<'a> {
    pub state: State,
    pub history: Vec<State>,
    pub agents: [&'a dyn Agent; 2],
    pub max_plies: u32,
    pub current_agent_idx: u32,
    pub current_player: Option<PlayerColor>,
}

impl<'a> Game<'a> {
    pub fn undo(&mut self) {}
}
