use std::io::{self, stdout, Write};

use serde::{Deserialize, Serialize};

use crate::{
    common::get_bit,
    game::{
        card::CARD_NAMES, done_move::DoneMove, game_state::GameState, piece::PieceKind,
        r#move::Move,
    },
};

use super::agent::Agent;

#[derive(Clone, Serialize, Deserialize)]
pub struct HumanConsole;

impl HumanConsole {
    pub fn read_console_input(input_string: &str) -> io::Result<String> {
        print!("{}", input_string);
        stdout().flush().unwrap();
        let mut user_input = String::new();
        let stdin = io::stdin(); // We get `Stdin` here.
        stdin.read_line(&mut user_input)?;
        Ok(user_input)
    }

    pub fn read_card_index() -> u32 {
        let card_idx: u32;
        loop {
            let user_input = match HumanConsole::read_console_input("Card index(0 or 1): ") {
                Ok(u) => u,
                Err(e) => {
                    println!("An error occurred while reading the input: {}", e);
                    continue;
                }
            };

            if user_input.trim().len() != 1 {
                println!("Incorrect length of the input!");
                continue;
            }

            card_idx = match user_input.chars().nth(0) {
                Some(x) => match x.to_digit(10) {
                    Some(x) => match x {
                        0..=1 => x,
                        _ => {
                            println!("A number must be 0 or 1!");
                            continue;
                        }
                    },
                    None => {
                        println!("Was not able to parse value: {}", x);
                        continue;
                    }
                },
                None => continue,
            };
            break;
        }
        card_idx
    }

    pub fn read_notation(input_string: &str) -> u32 {
        let from: u32;
        loop {
            let user_input = match HumanConsole::read_console_input(input_string) {
                Ok(u) => u,
                Err(e) => {
                    println!("An error occurred while reading the input: {}", e);
                    continue;
                }
            };

            if user_input.trim().len() != 2 {
                println!("Incorrect length of the input!");
                continue;
            }

            from = match Move::convert_notation_to_idx(&user_input) {
                Ok(x) => x,
                Err(e) => {
                    println!("An error while converting notation: {}", e);
                    continue;
                }
            };
            break;
        }
        from
    }
}

impl Agent for HumanConsole {
    fn name(&self) -> &'static str {
        "Human in Console"
    }

    fn generate_move(&self, game_state: &GameState) -> (DoneMove, f64) {
        let player_color = game_state.curr_player_color;
        let state = &game_state.state;
        let mut moves = vec![];
        for card in state.deck.get_player_cards(player_color) {
            println!("All possible moves for card {}", CARD_NAMES[card.index]);
            for mov in state.generate_legal_moves(player_color, card) {
                println!("{}", mov);
                moves.push(mov);
            }
        }

        let mut card_idx: u32;
        let mut from: u32;
        let mut to: u32;

        loop {
            // TODO: need to fix to use all indices not only 0 and 1
            card_idx = HumanConsole::read_card_index();
            from = HumanConsole::read_notation("From: ");
            to = HumanConsole::read_notation("To: ");

            let mov_exist = moves.iter().any(|m| m.from == from && m.to == to);
            if !mov_exist {
                println!(
                    "Move from {} to {} does not exist!",
                    Move::convert_idx_to_notation(from),
                    Move::convert_idx_to_notation(to)
                );
                continue;
            }
            break;
        }

        let piece = if get_bit(state.pawns[player_color as usize], from as usize) == 1 {
            PieceKind::Pawn
        } else {
            PieceKind::King
        };

        (
            DoneMove {
                mov: Move { from, to, piece },
                used_card_idx: card_idx as usize,
            },
            0.,
        )
    }

    fn clone_dyn(&self) -> Box<dyn Agent> {
        Box::new(self.clone())
    }

    fn id(&self) -> u64 {
        "human".parse::<u64>().unwrap()
    }
}
