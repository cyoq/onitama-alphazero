use crate::common::get_bit;

use super::{
    card::Card, deck::Deck, move_result::MoveResult, player_color::PlayerColor, r#move::Move,
};

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
        let border = String::from("---+---+---+---+---+---+\n");
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

    pub fn make_move(&mut self, mov: &Move, player_color: &PlayerColor, card: &Card) -> MoveResult {
        MoveResult::InProgress
    }

    pub fn generate_legal_moves(&self, player_color: &PlayerColor) -> Vec<Move> {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use crate::game::{
        card::{CRAB, DRAGON, FROG, RABBIT, TIGER},
        deck::Deck,
        player_color::PlayerColor,
        r#move::Move,
    };

    use super::State;

    #[test]
    fn correct_display() {
        let expected = "
---+---+---+---+---+---+
 5 | b | b | B | b | b |
---+---+---+---+---+---+
 4 | . | . | . | . | . |
---+---+---+---+---+---+
 3 | . | . | . | . | . |
---+---+---+---+---+---+
 2 | . | . | . | . | . |
---+---+---+---+---+---+
 1 | r | r | R | r | r |
---+---+---+---+---+---+
   | a | b | c | d | e |
";
        let state = State::new();
        let result = state.display();
        // Add newlines as result variable is separated by them for better observability
        assert_eq!(format!("\n{}\n", result), expected);
    }

    #[test]
    fn create_all_legal_moves_for_red_in_starting_position() {
        let deck = Deck::new([CRAB, RABBIT, DRAGON, TIGER, FROG]);
        let state = State::with_deck(deck);

        let expected_moves = vec![
            // All legal moves for the crab at starting position
            // All moves go forward for red
            Move::from([(4, 0), (3, 0)]),
            Move::from([(4, 1), (3, 1)]),
            Move::from([(4, 2), (3, 2)]),
            Move::from([(4, 3), (3, 3)]),
            Move::from([(4, 4), (3, 4)]),
            // All legal moves for the rabbit at starting position
            // All moves go diagonally
            Move::from([(4, 0), (3, 1)]),
            Move::from([(4, 1), (3, 2)]),
            Move::from([(4, 2), (3, 3)]),
            Move::from([(4, 3), (3, 4)]),
        ];
        let moves = state.generate_legal_moves(&PlayerColor::Red);
        assert_eq!(moves, expected_moves);
    }
}
