use std::ops::{Deref, DerefMut};

use onitama_game::game::{
    card::Card, move_result::MoveResult, player_color::PlayerColor, r#move::Move, state::State,
};
use serde::{Deserialize, Serialize};

use crate::player::Participant;

#[derive(Debug, Serialize, Deserialize)]
pub struct MoveHistory {
    // TODO: later on we should also save
    // each agent parameters
    // but should decide if serde will be the choice
    // https://stackoverflow.com/a/50026579 can be helpful
    red_player: Participant,
    blue_player: Participant,
    history: Vec<MoveInformation>,
}

impl MoveHistory {
    pub fn new(red_player: Participant, blue_player: Participant) -> Self {
        Self {
            red_player,
            blue_player,
            history: vec![],
        }
    }

    pub fn update_players(&mut self, red_player: Participant, blue_player: Participant) {
        self.red_player = red_player;
        self.blue_player = blue_player;
    }

    pub fn push(&mut self, move_information: MoveInformation) {
        self.history.push(move_information);
    }

    pub fn clear(&mut self) {
        self.history.clear();
    }

    pub fn save(&self) -> Result<(), String> {
        Ok(())
    }
}

impl Deref for MoveHistory {
    type Target = Vec<MoveInformation>;

    fn deref(&self) -> &Self::Target {
        &self.history
    }
}

impl DerefMut for MoveHistory {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.history
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MoveInformation {
    pub state: State,
    pub player_color: PlayerColor,
    pub done_move: Move,
    pub card: Card,
    pub evaluation: f64,
    pub ply: usize,
    pub move_result: MoveResult,
}
