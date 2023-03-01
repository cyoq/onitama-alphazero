use std::io::{self, stdout, Write};

use crate::{
    common::{from_2d_to_1d, get_bit},
    game::{
        done_move::DoneMove, figure::Figure, player_color::PlayerColor, r#move::Move, state::State,
    },
};

use super::agent::Agent;

pub struct Human;

// TODO: THIS CODE IS TERRYFYING, IT NEEDS REFACTORING ASAP!!!!!!
impl Agent for Human {
    fn generate_move(&self, state: &State, player_color: PlayerColor) -> DoneMove {
        for card in state.deck.get_player_cards(player_color) {
            println!("All possible moves for card {}", card.name);
            for mov in state.generate_legal_moves(player_color, card) {
                println!("{}", mov);
            }
        }

        let from: u32;
        let to: u32;
        let card_idx: u32;
        loop {
            print!("Card idx(0 or 1): ");
            stdout().flush().unwrap();
            let mut user_input = String::new();
            let stdin = io::stdin(); // We get `Stdin` here.
            stdin.read_line(&mut user_input).unwrap();

            if user_input.trim().len() != 1 {
                println!("Incorrect length of the input!");
                continue;
            }

            card_idx = user_input.chars().nth(0).unwrap().to_digit(10).unwrap();
            break;
        }

        loop {
            print!("From: ");
            stdout().flush().unwrap();
            let mut user_input = String::new();
            let stdin = io::stdin(); // We get `Stdin` here.
            stdin.read_line(&mut user_input).unwrap();

            if user_input.trim().len() != 2 {
                println!("Incorrect length of the input!");
                continue;
            }

            let letter = user_input.chars().nth(0).unwrap();
            let col = match letter {
                'a' => 0,
                'b' => 1,
                'c' => 2,
                'd' => 3,
                'e' => 4,
                _ => {
                    println!("First character is not a letter a, b, c, d or e!");
                    continue;
                }
            };
            let row = 5 - user_input.chars().nth(1).unwrap().to_digit(10).unwrap();

            let value = from_2d_to_1d((row, col));
            from = value;
            break;
        }

        loop {
            print!("To: ");
            stdout().flush().unwrap();
            let mut user_input = String::new();
            let stdin = io::stdin(); // We get `Stdin` here.
            stdin.read_line(&mut user_input).unwrap();

            if user_input.trim().len() != 2 {
                println!("Incorrect length of the input!");
                continue;
            }

            let letter = user_input.chars().nth(0).unwrap();
            let col = match letter {
                'a' => 0,
                'b' => 1,
                'c' => 2,
                'd' => 3,
                'e' => 4,
                _ => {
                    println!("First character is not a letter a, b, c, d or e!");
                    continue;
                }
            };
            let row = 5 - user_input.chars().nth(1).unwrap().to_digit(10).unwrap();

            let value = from_2d_to_1d((row, col));
            to = value;
            break;
        }

        let figure = if get_bit(state.pawns[player_color as usize], from as usize) == 1 {
            Figure::Pawn
        } else {
            Figure::King
        };

        DoneMove {
            mov: Move { from, to, figure },
            used_card_idx: card_idx as usize,
        }
    }
}
