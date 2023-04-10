use std::ops::{Deref, DerefMut};

use onitama_game::game::{card::Card, player_color::PlayerColor, r#move::Move, state::State};

use crate::player::Participant;

pub struct MoveHistory {
    participants: [Participant; 2],
    history: Vec<MoveInformation>,
}

impl MoveHistory {
    pub fn new(participants: [Participant; 2]) -> Self {
        Self {
            participants,
            history: vec![],
        }
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

pub struct MoveInformation {
    pub state: State,
    pub player_color: PlayerColor,
    pub done_move: Move,
    pub card: Card,
    pub evaluation: f64,
    pub ply: usize,
    pub is_win: bool,
    pub is_capture: bool,
}
