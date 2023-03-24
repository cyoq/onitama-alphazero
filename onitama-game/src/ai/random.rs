use rand::{thread_rng, Rng};

use crate::game::{done_move::DoneMove, game::GameState};

use super::agent::Agent;

#[derive(Clone)]
pub struct Random;

impl Agent for Random {
    fn generate_move(&self, game_state: &GameState) -> DoneMove {
        let player_color = game_state.curr_player_color;
        let state = &game_state.state;
        let mut rng = thread_rng();
        let cards = state.deck.get_player_cards(player_color);
        let card_idx = rng.gen_range(0..2);
        let card = cards[card_idx];

        let moves = state.generate_all_legal_moves(player_color, card);
        let mov = moves[rng.gen_range(0..moves.len())];

        DoneMove {
            mov: mov,
            used_card_idx: card_idx,
        }
    }

    fn name(&self) -> &'static str {
        "Random AI"
    }

    fn clone_dyn(&self) -> Box<dyn Agent> {
        Box::new(self.clone())
    }
}
