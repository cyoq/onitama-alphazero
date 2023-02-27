use crate::game::r#move::Move;

use super::agent::Agent;

struct Random;

impl Agent for Random {
    fn generate_move(&self, state: &State) -> Option<Move> {
        todo!()
    }
}
