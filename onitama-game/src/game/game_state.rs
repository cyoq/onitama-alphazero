use super::{
    deck::Deck, done_move::DoneMove, move_result::MoveResult, player_color::PlayerColor,
    state::State,
};

#[derive(Clone)]
pub struct GameState {
    /// Represents a current state of the game
    pub state: State,
    /// A history of the played state. TODO: Need to understand how the string cloning affects the performance
    pub history: Vec<State>,
    pub curr_agent_idx: usize,
    pub curr_player_color: PlayerColor,
}

impl GameState {
    pub fn new() -> Self {
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
            curr_agent_idx: current_player_idx,
            curr_player_color: player_color,
        }
    }

    pub fn with_deck(deck: Deck) -> Self {
        let state = State::with_deck(deck);
        let player_color = state.deck.neutral_card().player_color;
        let current_player_idx = match player_color {
            PlayerColor::Red => 0,
            PlayerColor::Blue => 1,
        };

        Self {
            state,
            history: vec![],
            curr_agent_idx: current_player_idx,
            curr_player_color: player_color,
        }
    }

    pub fn clear(&mut self) {
        self.state = match self.history.first() {
            Some(s) => s.clone(),
            None => State::with_deck(self.state.deck.clone()),
        };

        self.history.clear();

        self.curr_player_color = self.state.deck.neutral_card().player_color;
        self.curr_agent_idx = match self.curr_player_color {
            PlayerColor::Red => 0,
            PlayerColor::Blue => 1,
        };
    }

    pub fn progress(&mut self, done_move: DoneMove) -> MoveResult {
        // Save the history
        self.history.push(self.state.clone());

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
        self.curr_agent_idx = (self.curr_agent_idx + 1) % 2;
        self.curr_player_color.switch();
    }
}
