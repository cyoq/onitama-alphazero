use onitama_game::ai::agent::Agent;
use serde::{Deserialize, Serialize};

pub struct Player {
    pub typ: PlayerType,
    pub agent: Box<dyn Agent>,
}

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Clone, Copy)]
pub enum PlayerType {
    Human,
    Random,
    AlphaBeta,
    Mcts,
}

impl PlayerType {
    pub fn is_ai(&self) -> bool {
        match self {
            PlayerType::Human => false,
            _ => true,
        }
    }
}

impl ToString for PlayerType {
    fn to_string(&self) -> String {
        match self {
            PlayerType::Human => "Human".to_owned(),
            PlayerType::Random => "Random".to_owned(),
            PlayerType::AlphaBeta => "AlphaBeta".to_owned(),
            PlayerType::Mcts => "MCTS".to_owned(),
        }
    }
}
