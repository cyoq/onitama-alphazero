use onitama_game::ai::agent::Agent;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum PlayerType {
    Human,
    Ai,
}

pub struct Player {
    pub typ: PlayerType,
    pub agent: Box<dyn Agent>,
}

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Clone)]
pub enum Participant {
    Human,
    Random,
    AlphaBeta,
    Mcts,
}

impl Participant {
    pub fn to_player_type(&self) -> PlayerType {
        match self {
            Participant::Human => PlayerType::Human,
            _ => PlayerType::Ai,
        }
    }
}

impl ToString for Participant {
    fn to_string(&self) -> String {
        match self {
            Participant::Human => "Human".to_owned(),
            Participant::Random => "Random".to_owned(),
            Participant::AlphaBeta => "AlphaBeta".to_owned(),
            Participant::Mcts => "MCTS".to_owned(),
        }
    }
}
