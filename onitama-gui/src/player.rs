use onitama_game::ai::agent::Agent;

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum PlayerType {
    Human,
    Ai,
}

pub struct Player {
    pub typ: PlayerType,
    pub agent: Box<dyn Agent>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Participant {
    Human,
    Random,
    AlphaBeta,
    Mcts,
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
