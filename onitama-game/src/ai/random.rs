use rand::{thread_rng, Rng};

use crate::game::{done_move::DoneMove, player_color::PlayerColor, state::State};

use super::agent::Agent;

pub struct Random;

impl Agent for Random {
    fn generate_move(&self, state: &State, player_color: PlayerColor) -> DoneMove {
        let mut rng = thread_rng();
        let cards = state.deck.get_player_cards(player_color);
        let card_idx = rng.gen_range(0..2);
        let card = cards[card_idx];

        let moves = state.generate_legal_moves(player_color, card);
        let mov = moves[rng.gen_range(0..moves.len())];

        DoneMove {
            mov: mov,
            used_card_idx: card_idx,
        }
    }
}
