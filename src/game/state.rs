use crate::common::get_bit;

use super::{deck::Deck, move_result::MoveResult, player_color::PlayerColor, r#move::Move};

// Figure starting positions(SP)
// 00000
// 00000
// 00000
// 00000
// 00100
// 0000000 - Trailing zeroes
pub const RED_KING_SP: u32 = 0x0000_0200;
// 00100
// 00000
// 00000
// 00000
// 00000
// 0000000 - Trailing zeroes
pub const BLUE_KING_SP: u32 = 0x2000_0000;
// 11011
// 00000
// 00000
// 00000
// 00000
// 0000000 - Trailing zeroes
pub const BLUE_PAWNS_SP: u32 = 0xD800_0000;
// 00000
// 00000
// 00000
// 00000
// 11011
// 0000000 - Trailing zeroes
pub const RED_PAWNS_SP: u32 = 0x0000_0D80;

// Array acces indices
pub const RED_KING_IDX: usize = 0;
pub const BLUE_KING_IDX: usize = 1;
pub const RED_PAWNS_IDX: usize = 0;
pub const BLUE_PAWNS_IDX: usize = 1;

#[derive(Debug, Clone)]
pub struct State {
    pub deck: Deck,
    pub kings: [u32; 2],
    pub pawns: [u32; 2],
}

impl State {
    pub fn new() -> Self {
        State {
            deck: Deck::default(),
            kings: [RED_KING_SP, BLUE_KING_SP],
            pawns: [RED_PAWNS_SP, BLUE_PAWNS_SP],
        }
    }

    pub fn with_deck(deck: Deck) -> Self {
        State {
            deck,
            kings: [RED_KING_SP, BLUE_KING_SP],
            pawns: [RED_PAWNS_SP, BLUE_PAWNS_SP],
        }
    }

    pub fn display(&self) -> String {
        let border = String::from("   +---+---+---+---+---+\n");
        let mut result = border.clone();
        let length: usize = 25;
        for i in 0..length {
            // Add a number if it is as start of a string
            if i % 5 == 0 {
                result += &format!(" {} ", (5 - i / 5).to_string());
            }

            // Check cell type
            if get_bit(self.pawns[RED_PAWNS_IDX], i) == 1 {
                result += "| r "
            } else if get_bit(self.pawns[BLUE_PAWNS_IDX], i) == 1 {
                result += "| b "
            } else if get_bit(self.kings[RED_KING_IDX], i) == 1 {
                result += "| R "
            } else if get_bit(self.kings[BLUE_KING_IDX], i) == 1 {
                result += "| B "
            } else {
                result += "| . ";
            }

            // Add a border and ending wall if it is an end of the line
            if (i + 1) % 5 == 0 {
                result += "|\n";
                result += &border;
            }
        }

        // Add a column identifier
        result += "   | a | b | c | d | e |";
        result
    }

    pub fn make_move(&mut self, mov: Move, player_color: PlayerColor) -> MoveResult {
        MoveResult::InProgress
    }

    pub fn generate_legal_moves(&self) -> Vec<Move> {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::State;

    #[test]
    fn correct_display() {
        let expected = "
   +---+---+---+---+---+
 5 | b | b | B | b | b |
   +---+---+---+---+---+
 4 | . | . | . | . | . |
   +---+---+---+---+---+
 3 | . | . | . | . | . |
   +---+---+---+---+---+
 2 | . | . | . | . | . |
   +---+---+---+---+---+
 1 | r | r | R | r | r |
   +---+---+---+---+---+
   | a | b | c | d | e |
";
        let state = State::new();
        let result = state.display();
        assert_eq!(format!("\n{}\n", result), expected);
    }
}
