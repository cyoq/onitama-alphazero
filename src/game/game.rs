use crate::ai::agent::Agent;

use super::{move_result::MoveResult, player_color::PlayerColor, state::State};

pub struct Game<'a> {
    /// Represents a current state of the game
    pub state: State,
    /// A history of the played state. TODO: Need to understand how the string cloning affects the performance
    pub history: Vec<State>,
    /// Agents that are in the game. First agent is a red one, second is a blue one
    pub agents: [&'a dyn Agent; 2],
    pub max_plies: u32,
    pub curr_agent_idx: usize,
    pub curr_player: PlayerColor,
}

impl<'a> Game<'a> {
    pub fn new(max_plies: u32, red_agent: &'a dyn Agent, blue_agent: &'a dyn Agent) -> Self {
        let state = State::new();
        let player_color = state.deck.neutral_card().player_color;
        let current_player_idx = if player_color == PlayerColor::Red {
            0
        } else {
            1
        };

        Self {
            state,
            history: vec![],
            agents: [red_agent, blue_agent],
            max_plies,
            curr_agent_idx: current_player_idx,
            curr_player: player_color,
        }
    }

    pub fn progress(&mut self) -> MoveResult {
        let mov = self.agents[self.curr_agent_idx].generate_move(&self.state);

        MoveResult::InProgress
    }

    pub fn undo(&mut self) {}
}
