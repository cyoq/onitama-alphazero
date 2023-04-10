use onitama_game::game::{done_move::DoneMove, state::State};

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

    pub fn push(&mut self, state: State, done_move: DoneMove, evaluation: f64) {
        self.history.push(MoveInformation {
            state,
            done_move,
            evaluation,
        });
    }

    pub fn clear(&mut self) {
        self.history.clear();
    }

    pub fn save(&self) -> Result<(), String> {
        Ok(())
    }
}

pub struct MoveInformation {
    pub state: State,
    pub done_move: DoneMove,
    pub evaluation: f64,
}
