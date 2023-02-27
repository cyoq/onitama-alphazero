use crate::game::r#move::Move;

pub trait Agent {
    fn make_move(&self) -> Option<Move>;
}
