use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

use crate::game::{done_move::DoneMove, game_state::GameState, r#move::Move};

use super::agent::Agent;

#[derive(Clone, Serialize, Deserialize)]
pub struct Random;

impl Agent for Random {
    fn generate_move(&self, game_state: &GameState) -> (DoneMove, f64) {
        let player_color = game_state.curr_player_color;
        let state = &game_state.state;
        let mut rng = thread_rng();
        let cards = state.deck.get_player_cards(player_color);
        let card_idx = rng.gen_range(0..2);
        let card = cards[card_idx];

        let moves = state.generate_legal_moves(player_color, card);
        // TODO: I have not implemented a turn skip when a random player does not have a legal move.
        // Therefore for the random agent, he will just make one move disregarding any rule.
        // However, it won't help him to win, because every other agent beats him up.
        let mov = if moves.len() > 0 {
            moves[rng.gen_range(0..moves.len())]
        } else {
            // He will just imitate a move from a5 to a4 with a pawn.
            // If it does not exist, it will just appear on the board.
            Move {
                from: 0,
                to: 5,
                piece: crate::game::piece::PieceKind::Pawn,
            }
        };

        (
            DoneMove {
                mov,
                used_card_idx: card_idx,
            },
            0.,
        )
    }

    fn name(&self) -> &'static str {
        "Random AI"
    }

    fn clone_dyn(&self) -> Box<dyn Agent> {
        Box::new(self.clone())
    }

    fn id(&self) -> u64 {
        "random".parse::<u64>().unwrap()
    }
}
