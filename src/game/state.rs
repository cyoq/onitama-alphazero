use crate::common::get_bit;

use super::{
    card::{Card, ATTACK_MAPS},
    deck::Deck,
    move_result::MoveResult,
    player_color::PlayerColor,
    r#move::Move,
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

pub const RED_PLAYER_IDX: usize = 0;
pub const BLUE_PLAYER_IDX: usize = 1;

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

    pub fn generate_legal_moves(&self, player_color: &PlayerColor, card: &Card) -> Vec<Move> {
        let mut result: Vec<Move> = Vec::new();
        // Save pawns for the specific player
        let pawns: u32;
        // Save king for the specific player
        let king: u32;
        // Save index for the attack lookup map
        let player_idx: usize;

        // Get pawns and king position considering the player position
        if *player_color == PlayerColor::Red {
            pawns = self.pawns[RED_PLAYER_IDX];
            king = self.kings[RED_PLAYER_IDX];
            player_idx = RED_PLAYER_IDX;
        } else {
            pawns = self.pawns[BLUE_PLAYER_IDX];
            king = self.kings[BLUE_PLAYER_IDX];
            player_idx = BLUE_PLAYER_IDX;
        }

        for n in 0..25 {
            // Getting a position bit for a pawn
            let pawn_bit = get_bit(pawns, n);
            let king_bit = get_bit(king, n);

            // If bits contain 0 then there is no piece at this position,
            // No need to process
            if pawn_bit == 0 && king_bit == 0 {
                continue;
            }

            // Get attack map for the specific player, card and the position
            let attack_map = ATTACK_MAPS[player_idx][card.index][n];

            let map: u32;
            // Generate attacks
            if pawn_bit == 1 {
                // 1. apply all pawns to the attack map
                // 2. mask out all the pawns to remove overlapping moves with the same color figures
                // 3. mask out the king figure if it is overlapping
                map = ((attack_map | pawns) & !pawns) & !king;
            } else {
                // 1. apply a king to the attack map
                // 2. mask out the king figure if it is overlapping
                // 3. mask out all the pawns to remove overlapping moves of mask and pawns
                map = ((attack_map | king) & !king) & !pawns;
            }

            for i in 0..25 {
                let bit = get_bit(map, i);

                if bit == 0 {
                    continue;
                }

                result.push(Move {
                    from: n as u32,
                    to: i as u32,
                });
            }
        }

        result
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

        let cards = state.deck.get_player_cards(&PlayerColor::Red);
        let crab = cards[0];

        let mut crab_expected_moves = vec![
            // All legal moves for the crab at starting position
            // All moves go forward for red
            Move::from([(4, 0), (3, 0)]),
            Move::from([(4, 1), (3, 1)]),
            Move::from([(4, 2), (3, 2)]),
            Move::from([(4, 3), (3, 3)]),
            Move::from([(4, 4), (3, 4)]),
        ];
        let mut crab_moves = state.generate_legal_moves(&PlayerColor::Red, crab);
        crab_moves.sort();
        crab_expected_moves.sort();
        assert_eq!(crab_moves, crab_expected_moves);

        let rabbit = cards[1];
        let mut rabbit_expected_moves = vec![
            // All legal moves for the rabbit at starting position
            // All moves go diagonally
            Move::from([(4, 0), (3, 1)]),
            Move::from([(4, 1), (3, 2)]),
            Move::from([(4, 2), (3, 3)]),
            Move::from([(4, 3), (3, 4)]),
        ];
        let mut rabbit_moves = state.generate_legal_moves(&PlayerColor::Red, rabbit);
        rabbit_moves.sort();
        rabbit_expected_moves.sort();
        assert_eq!(rabbit_moves, rabbit_expected_moves);
    }

    #[test]
    fn create_all_legal_moves_for_blue_in_starting_position() {
        let deck = Deck::new([DRAGON, TIGER, CRAB, RABBIT, FROG]);
        let state = State::with_deck(deck);

        let cards = state.deck.get_player_cards(&PlayerColor::Blue);

        let crab = cards[0];

        let mut crab_expected_moves = vec![
            // All legal moves for the crab at starting position
            // All moves go forward for red
            Move::from([(0, 0), (1, 0)]),
            Move::from([(0, 1), (1, 1)]),
            Move::from([(0, 2), (1, 2)]),
            Move::from([(0, 3), (1, 3)]),
            Move::from([(0, 4), (1, 4)]),
        ];
        let mut crab_moves = state.generate_legal_moves(&PlayerColor::Blue, crab);
        crab_moves.sort();
        crab_expected_moves.sort();
        assert_eq!(crab_moves, crab_expected_moves);

        let rabbit = cards[1];
        let mut rabbit_expected_moves = vec![
            // All legal moves for the rabbit at starting position
            // All moves go diagonally
            Move::from([(0, 1), (1, 0)]),
            Move::from([(0, 2), (1, 1)]),
            Move::from([(0, 3), (1, 2)]),
            Move::from([(0, 4), (1, 3)]),
        ];
        let mut rabbit_moves = state.generate_legal_moves(&PlayerColor::Blue, rabbit);
        rabbit_moves.sort();
        rabbit_expected_moves.sort();
        assert_eq!(rabbit_moves, rabbit_expected_moves);
    }
}
