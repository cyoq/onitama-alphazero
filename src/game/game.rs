use crate::ai::agent::Agent;

use super::{move_result::MoveResult, player_color::PlayerColor, state::State};

pub struct Game<'a> {
    /// Represents a current state of the game
    pub state: State,
    /// A history of the played state. TODO: Need to understand how the string cloning affects the performance
    pub history: Vec<State>,
    /// Agents that are in the game. First agent is a red one, second is a blue one
    pub agents: [&'a dyn Agent; 2],
    pub curr_agent_idx: usize,
    pub curr_player_color: PlayerColor,
}

impl<'a> Game<'a> {
    pub fn new(red_agent: &'a dyn Agent, blue_agent: &'a dyn Agent) -> Self {
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
            curr_agent_idx: current_player_idx,
            curr_player_color: player_color,
        }
    }

    pub fn next_turn(&mut self) -> MoveResult {
        // Save the history
        self.history.push(self.state.clone());

        // move must be a legal one
        let done_move =
            self.agents[self.curr_agent_idx].generate_move(&self.state, self.curr_player_color);

        let move_result = self.state.make_move(
            &done_move.mov,
            self.curr_player_color,
            done_move.used_card_idx,
        );

        // progress the game to the next turn
        self.curr_agent_idx = (self.curr_agent_idx + 1) % 2;
        self.curr_player_color.switch();

        move_result
    }

    pub fn undo(&mut self) {
        match self.history.pop() {
            Some(state) => self.state = state,
            None => (),
        }
    }
}
