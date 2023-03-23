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
