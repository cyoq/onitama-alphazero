pub mod evaluation;

use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::game::{
    done_move::DoneMove, game_state::GameState, move_result::MoveResult, player_color::PlayerColor,
};

use self::evaluation::Evaluation;

use super::agent::Agent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlphaBeta {
    pub max_depth: u8,
    pub search_time: Duration,
}

impl Default for AlphaBeta {
    fn default() -> Self {
        Self {
            max_depth: 6,
            search_time: Duration::from_secs(1),
        }
    }
}

struct CalculationResult {
    best_move: Option<DoneMove>,
    best_score: i32,
}

impl AlphaBeta {
    fn alpha_beta(
        &self,
        depth: u8,
        max_depth: u8,
        mut alpha: i32,
        mut beta: i32,
        game_state: &mut GameState,
        move_result: Option<MoveResult>,
        positions: &mut i32,
    ) -> CalculationResult {
        *positions += 1;
        let player_color = game_state.curr_player_color;

        if depth == max_depth
            || move_result == Some(MoveResult::BlueWin)
            || move_result == Some(MoveResult::RedWin)
        {
            return CalculationResult {
                best_move: None,
                best_score: Evaluation::evaluate(&game_state.state, player_color, &move_result),
            };
        }

        let cards = game_state.state.deck.get_player_cards_idx(player_color);

        let mut best_score;
        if player_color == PlayerColor::Red {
            best_score = std::i32::MIN;
        } else {
            best_score = std::i32::MAX;
        }

        let mut best_move = None;

        for card_idx in cards {
            let allowed_moves = game_state
                .state
                .generate_legal_moves_card_idx(player_color, card_idx);

            'br: for mov in allowed_moves.iter() {
                let done_move = DoneMove {
                    mov: *mov,
                    used_card_idx: card_idx,
                };
                let result = game_state.progress(done_move);

                // go deeper the tree
                let calc_result = self.alpha_beta(
                    depth + 1,
                    max_depth,
                    alpha,
                    beta,
                    game_state,
                    Some(result),
                    positions,
                );

                let score = calc_result.best_score;

                // Undo all made moves
                game_state.undo();

                if player_color == PlayerColor::Red {
                    if score > best_score {
                        best_score = score;
                        best_move = Some(done_move);
                    }

                    if score >= beta {
                        break 'br;
                    }

                    alpha = std::cmp::max(alpha, score);
                } else {
                    if score < best_score {
                        best_score = score;
                        best_move = Some(done_move);
                    }

                    if score <= alpha {
                        break 'br;
                    }

                    beta = std::cmp::min(beta, score);
                }

                if alpha >= beta {
                    break 'br;
                }
            }
        }

        CalculationResult {
            best_move,
            best_score,
        }
    }
}

impl Agent for AlphaBeta {
    fn generate_move(&self, game_state: &GameState) -> (DoneMove, f64) {
        let mut positions = 0;

        let mut game_state = game_state.clone();
        let mut result = None;

        let mut depth = 1;
        let now = Instant::now();

        while now.elapsed() < self.search_time && depth < self.max_depth {
            result = Some(self.alpha_beta(
                0,
                depth,
                std::i32::MIN,
                std::i32::MAX,
                &mut game_state,
                None,
                &mut positions,
            ));
            depth += 1;
        }

        println!("Last depth {}", depth);
        println!("Analyzed {} positions", positions);
        // We must receive some kind of result
        let result = result.unwrap();
        println!("Best score is {}", result.best_score);

        (
            result
                .best_move
                .expect("AlphaBeta agent must produce a move!"),
            result.best_score as f64,
        )
    }

    fn name(&self) -> &'static str {
        "AlphaBeta AI"
    }

    fn clone_dyn(&self) -> Box<dyn Agent> {
        Box::new(self.clone())
    }

    fn id(&self) -> u64 {
        self.search_time.as_nanos() as u64 + self.max_depth as u64
    }
}
