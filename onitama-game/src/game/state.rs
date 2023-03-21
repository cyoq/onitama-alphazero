use crate::common::{clear_bit, from_2d_to_1d, get_bit, set_bit};

use super::{
    card::{Card, ATTACK_MAPS},
    deck::Deck,
    figure::Figure,
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

// Indexes where the temples are located
pub const BLUE_TEMPLE: usize = 2;
pub const RED_TEMPLE: usize = 22;

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
            if get_bit(self.pawns[PlayerColor::Red as usize], i) == 1 {
                result += "| r "
            } else if get_bit(self.pawns[PlayerColor::Blue as usize], i) == 1 {
                result += "| b "
            } else if get_bit(self.kings[PlayerColor::Red as usize], i) == 1 {
                result += "| R "
            } else if get_bit(self.kings[PlayerColor::Blue as usize], i) == 1 {
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

    /// When making a move, we assume that the move is completely legal by rules
    pub fn make_move(
        &mut self,
        mov: &Move,
        player_color: PlayerColor,
        used_card_idx: usize,
    ) -> MoveResult {
        let from = mov.from as usize;
        let to = mov.to as usize;
        let figure = mov.figure;
        let mut move_result = MoveResult::InProgress;

        // Clear 'from' position
        match figure {
            Figure::Pawn => clear_bit(&mut self.pawns[player_color as usize], from),
            Figure::King => clear_bit(&mut self.kings[player_color as usize], from),
        }

        // Check if there is a capture of an enemy piece
        let enemy = player_color.enemy();

        let enemy_pawn = get_bit(self.pawns[enemy as usize], to);
        let enemy_king = get_bit(self.kings[enemy as usize], to);

        if enemy_pawn == 1 {
            // Clear enemy bit
            clear_bit(&mut self.pawns[enemy as usize], to);
            // Set a move result
            move_result = MoveResult::Capture;
        } else if enemy_king == 1 {
            // Clear enemy bit
            clear_bit(&mut self.kings[enemy as usize], to);
            // Set a move result
            match player_color {
                PlayerColor::Red => move_result = MoveResult::RedWin,
                PlayerColor::Blue => move_result = MoveResult::BlueWin,
            }
        }

        // Set a piece to the 'to' position
        match figure {
            Figure::Pawn => set_bit(&mut self.pawns[player_color as usize], to),
            Figure::King => set_bit(&mut self.kings[player_color as usize], to),
        }

        // If it is a king, check if it is coming to the temple
        if figure == Figure::King {
            match (player_color, to) {
                (PlayerColor::Red, BLUE_TEMPLE) => move_result = MoveResult::RedWin,
                (PlayerColor::Blue, RED_TEMPLE) => move_result = MoveResult::BlueWin,
                _ => (),
            }
        }

        // Card rotation:
        self.deck.rotate(used_card_idx, player_color);

        move_result
    }

    /// Generates all legal moves for the specific card and the player
    pub fn generate_legal_moves_card_idx(
        &self,
        player_color: PlayerColor,
        card_idx: usize,
        pos: (u32, u32),
    ) -> Vec<Move> {
        println!("Current deck: {:?}", self.deck);
        let card = self.deck.get_card(card_idx);
        self.generate_legal_moves(card, player_color, pos)
    }

    pub fn generate_legal_moves(
        &self,
        card: &Card,
        player_color: PlayerColor,
        pos: (u32, u32),
    ) -> Vec<Move> {
        let mut moves = vec![];

        // Save pawns for the specific player
        let pawns: u32 = self.pawns[player_color as usize];
        // Save king for the specific player
        let king: u32 = self.kings[player_color as usize];

        let n = from_2d_to_1d(pos) as usize;
        // Getting a position bit for a pawn
        let pawn_bit = get_bit(pawns, n);
        let king_bit = get_bit(king, n);

        // If bits contain 0 then there is no piece at this position,
        // No need to process
        if pawn_bit == 0 && king_bit == 0 {
            return moves;
        }

        // Get attack map for the specific player, card and the position
        let attack_map = ATTACK_MAPS[player_color as usize][card.index][n];

        let map: u32;
        let figure: Figure;
        // Generate attacks
        if pawn_bit == 1 {
            // 1. apply all pawns to the attack map
            // 2. mask out all the pawns to remove overlapping moves with the same color figures
            // 3. mask out the same color king figure if it is overlapping
            map = ((attack_map | pawns) & !pawns) & !king;
            figure = Figure::Pawn;
        } else {
            // 1. apply a king to the attack map
            // 2. mask out the same color king figure if it is overlapping
            // 3. mask out all the pawns to remove overlapping moves of mask and pawns
            map = ((attack_map | king) & !king) & !pawns;
            figure = Figure::King;
        }

        // Generate moves
        for i in 0..25 {
            let bit = get_bit(map, i);

            if bit == 0 {
                continue;
            }

            moves.push(Move {
                from: n as u32,
                to: i as u32,
                figure,
            });
        }

        moves
    }

    /// Generates all legal moves for the specific card and the player
    pub fn generate_all_legal_moves_card_idx(
        &self,
        player_color: PlayerColor,
        card_idx: usize,
    ) -> Vec<Move> {
        let card = self.deck.get_card(card_idx);
        self.generate_all_legal_moves(player_color, card)
    }

    /// Generates all legal moves for the specific card and the player
    pub fn generate_all_legal_moves(&self, player_color: PlayerColor, card: &Card) -> Vec<Move> {
        let mut result: Vec<Move> = Vec::new();
        // Save pawns for the specific player
        let pawns: u32 = self.pawns[player_color as usize];
        // Save king for the specific player
        let king: u32 = self.kings[player_color as usize];

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
            let attack_map = ATTACK_MAPS[player_color as usize][card.index][n];

            let map: u32;
            let figure: Figure;
            // Generate attacks
            if pawn_bit == 1 {
                // 1. apply all pawns to the attack map
                // 2. mask out all the pawns to remove overlapping moves with the same color figures
                // 3. mask out the same color king figure if it is overlapping
                map = ((attack_map | pawns) & !pawns) & !king;
                figure = Figure::Pawn;
            } else {
                // 1. apply a king to the attack map
                // 2. mask out the same color king figure if it is overlapping
                // 3. mask out all the pawns to remove overlapping moves of mask and pawns
                map = ((attack_map | king) & !king) & !pawns;
                figure = Figure::King;
            }

            // Generate moves
            for i in 0..25 {
                let bit = get_bit(map, i);

                if bit == 0 {
                    continue;
                }

                result.push(Move {
                    from: n as u32,
                    to: i as u32,
                    figure,
                });
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        common::get_bit,
        game::{
            card::{CRAB, DRAGON, FROG, RABBIT, TIGER},
            deck::Deck,
            figure::Figure,
            move_result::MoveResult,
            player_color::PlayerColor,
            r#move::Move,
        },
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

        let cards = state.deck.get_player_cards(PlayerColor::Red);
        let crab = cards[0];

        let mut crab_expected_moves = vec![
            // All legal moves for the crab at starting position
            // All moves go forward for red
            Move::from(([(4, 0), (3, 0)], Figure::Pawn)),
            Move::from(([(4, 1), (3, 1)], Figure::Pawn)),
            Move::from(([(4, 2), (3, 2)], Figure::King)),
            Move::from(([(4, 3), (3, 3)], Figure::Pawn)),
            Move::from(([(4, 4), (3, 4)], Figure::Pawn)),
        ];
        let mut crab_moves = state.generate_all_legal_moves(PlayerColor::Red, crab);
        crab_moves.sort();
        crab_expected_moves.sort();
        assert_eq!(crab_moves, crab_expected_moves);

        let rabbit = cards[1];
        let mut rabbit_expected_moves = vec![
            // All legal moves for the rabbit at starting position
            // All moves go diagonally
            Move::from(([(4, 0), (3, 1)], Figure::Pawn)),
            Move::from(([(4, 1), (3, 2)], Figure::Pawn)),
            Move::from(([(4, 2), (3, 3)], Figure::King)),
            Move::from(([(4, 3), (3, 4)], Figure::Pawn)),
        ];
        let mut rabbit_moves = state.generate_all_legal_moves(PlayerColor::Red, rabbit);
        rabbit_moves.sort();
        rabbit_expected_moves.sort();
        assert_eq!(rabbit_moves, rabbit_expected_moves);
    }

    #[test]
    fn create_all_legal_moves_for_blue_in_starting_position() {
        let deck = Deck::new([DRAGON, TIGER, CRAB, RABBIT, FROG]);
        let state = State::with_deck(deck);

        let cards = state.deck.get_player_cards(PlayerColor::Blue);

        let crab = cards[0];

        let mut crab_expected_moves = vec![
            // All legal moves for the crab at starting position
            // All moves go forward for red
            Move::from(([(0, 0), (1, 0)], Figure::Pawn)),
            Move::from(([(0, 1), (1, 1)], Figure::Pawn)),
            Move::from(([(0, 2), (1, 2)], Figure::King)),
            Move::from(([(0, 3), (1, 3)], Figure::Pawn)),
            Move::from(([(0, 4), (1, 4)], Figure::Pawn)),
        ];
        let mut crab_moves = state.generate_all_legal_moves(PlayerColor::Blue, crab);
        crab_moves.sort();
        crab_expected_moves.sort();
        assert_eq!(crab_moves, crab_expected_moves);

        let rabbit = cards[1];
        let mut rabbit_expected_moves = vec![
            // All legal moves for the rabbit at starting position
            // All moves go diagonally
            Move::from(([(0, 1), (1, 0)], Figure::Pawn)),
            Move::from(([(0, 2), (1, 1)], Figure::King)),
            Move::from(([(0, 3), (1, 2)], Figure::Pawn)),
            Move::from(([(0, 4), (1, 3)], Figure::Pawn)),
        ];
        let mut rabbit_moves = state.generate_all_legal_moves(PlayerColor::Blue, rabbit);
        rabbit_moves.sort();
        rabbit_expected_moves.sort();
        assert_eq!(rabbit_moves, rabbit_expected_moves);
    }

    #[test]
    fn make_move_as_red() {
        let deck = Deck::new([CRAB, RABBIT, DRAGON, TIGER, FROG]);
        let mut state = State::with_deck(deck);

        let cards = state.deck.get_player_cards(PlayerColor::Red);
        // Cloning to avoid immutable borrow before mutable
        let crab = cards[0].clone();

        let mov = Move {
            from: 20, // a1
            to: 15,   // a2
            figure: Figure::Pawn,
        };
        let mov_result = state.make_move(&mov, PlayerColor::Red, 0);

        // Check if the game stays in progress after the move
        assert_eq!(mov_result, MoveResult::InProgress);
        // a2 has a pawn
        assert_eq!(get_bit(state.pawns[PlayerColor::Red as usize], 15), 1);
        // a1 does not have a pawn
        assert_eq!(get_bit(state.pawns[PlayerColor::Red as usize], 20), 0);
        // Check card rotation
        assert_eq!(*state.deck.neutral_card(), crab)
    }

    #[test]
    fn make_move_as_blue() {
        let deck = Deck::new([CRAB, RABBIT, DRAGON, TIGER, FROG]);
        let mut state = State::with_deck(deck);

        let cards = state.deck.get_player_cards(PlayerColor::Blue);
        // Cloning to avoid immutable borrow before mutable
        let tiger = cards[1].clone();

        let mov = Move {
            from: 1, // b5
            to: 11,  // b3
            figure: Figure::Pawn,
        };
        let mov_result = state.make_move(&mov, PlayerColor::Blue, 1);

        // Check if the game stays in progress after the move
        assert_eq!(mov_result, MoveResult::InProgress);
        // b3 has a pawn
        assert_eq!(get_bit(state.pawns[PlayerColor::Blue as usize], 11), 1);
        // b5 does not have a pawn
        assert_eq!(get_bit(state.pawns[PlayerColor::Blue as usize], 1), 0);
        // Check card rotation
        assert_eq!(*state.deck.neutral_card(), tiger);
    }

    #[test]
    fn capture_as_red() {
        let deck = Deck::new([CRAB, RABBIT, DRAGON, TIGER, FROG]);
        let mut state = State::with_deck(deck);

        // Set the following state:
        /*
            ---+---+---+---+---+---+
             5 | . | b | B | b | b |
            ---+---+---+---+---+---+
             4 | . | . | . | . | . |
            ---+---+---+---+---+---+
             3 | . | . | . | . | . |
            ---+---+---+---+---+---+
             2 | b | . | . | . | . |
            ---+---+---+---+---+---+
             1 | r | r | R | r | r |
            ---+---+---+---+---+---+
               | a | b | c | d | e |
        */
        state.pawns[PlayerColor::Blue as usize] = 0x5801_0000;

        let cards = state.deck.get_player_cards(PlayerColor::Red);
        // Cloning to avoid immutable borrow before mutable
        let crab = cards[0].clone();

        let mov = Move {
            from: 20, // a1
            to: 15,   // a2
            figure: Figure::Pawn,
        };
        let mov_result = state.make_move(&mov, PlayerColor::Red, 0);

        // Check if capture occurred
        assert_eq!(mov_result, MoveResult::Capture);
        // a2 has a pawn
        assert_eq!(get_bit(state.pawns[PlayerColor::Red as usize], 15), 1);
        // a1 does not have a pawn
        assert_eq!(get_bit(state.pawns[PlayerColor::Red as usize], 20), 0);
        // Check card rotation
        assert_eq!(*state.deck.neutral_card(), crab);
        // Check if blue has only 3 pawns on b5, d5 and e5
        assert_eq!(state.pawns[PlayerColor::Blue as usize], 0x5800_0000);
    }

    #[test]
    fn capture_as_blue() {
        let deck = Deck::new([CRAB, RABBIT, DRAGON, TIGER, FROG]);
        let mut state = State::with_deck(deck);

        // Set the following state:
        /*
            ---+---+---+---+---+---+
             5 | . | b | B | b | b |
            ---+---+---+---+---+---+
             4 | . | . | . | . | . |
            ---+---+---+---+---+---+
             3 | b | . | . | . | . |
            ---+---+---+---+---+---+
             2 | . | . | . | . | . |
            ---+---+---+---+---+---+
             1 | r | r | R | r | r |
            ---+---+---+---+---+---+
               | a | b | c | d | e |
        */
        state.pawns[PlayerColor::Blue as usize] = 0x5820_0000;

        let cards = state.deck.get_player_cards(PlayerColor::Blue);
        // Cloning to avoid immutable borrow before mutable
        let tiger = cards[1].clone();

        let mov = Move {
            from: 10, // a3
            to: 20,   // a1
            figure: Figure::Pawn,
        };
        let mov_result = state.make_move(&mov, PlayerColor::Blue, 1);

        // Check if capture occurred
        assert_eq!(mov_result, MoveResult::Capture);
        // a1 has a pawn
        assert_eq!(get_bit(state.pawns[PlayerColor::Blue as usize], 20), 1);
        // a3 does not have a pawn
        assert_eq!(get_bit(state.pawns[PlayerColor::Blue as usize], 10), 0);
        // Check card rotation
        assert_eq!(*state.deck.neutral_card(), tiger);
        // Check if red has only 3 pawns on b1, d1 and e1
        assert_eq!(state.pawns[PlayerColor::Red as usize], 0x0000_0580);
    }

    #[test]
    fn capture_win_as_red() {
        let deck = Deck::new([CRAB, RABBIT, DRAGON, TIGER, FROG]);
        let mut state = State::with_deck(deck);

        // Set the following state:
        /*
            ---+---+---+---+---+---+
             5 | b | b | B | b | b |
            ---+---+---+---+---+---+
             4 | . | . | r | . | . |
            ---+---+---+---+---+---+
             3 | . | . | . | . | . |
            ---+---+---+---+---+---+
             2 | . | . | . | . | . |
            ---+---+---+---+---+---+
             1 | . | r | R | r | r |
            ---+---+---+---+---+---+
               | a | b | c | d | e |
        */
        state.pawns[PlayerColor::Red as usize] = 0x0100_0680;

        let cards = state.deck.get_player_cards(PlayerColor::Red);
        // Cloning to avoid immutable borrow before mutable
        let crab = cards[0].clone();

        let mov = Move {
            from: 7, // c4
            to: 2,   // c5
            figure: Figure::Pawn,
        };
        let mov_result = state.make_move(&mov, PlayerColor::Red, 0);

        // Check if the game is won
        assert_eq!(mov_result, MoveResult::RedWin);
        // c5 has a pawn
        assert_eq!(get_bit(state.pawns[PlayerColor::Red as usize], 2), 1);
        // c4 does not have a pawn
        assert_eq!(get_bit(state.pawns[PlayerColor::Red as usize], 7), 0);
        // Check card rotation
        assert_eq!(*state.deck.neutral_card(), crab);
        // Check if blue king is dead
        assert_eq!(state.kings[PlayerColor::Blue as usize], 0);
    }

    #[test]
    fn capture_win_as_blue() {
        let deck = Deck::new([CRAB, RABBIT, DRAGON, TIGER, FROG]);
        let mut state = State::with_deck(deck);

        // Set the following state:
        /*
            ---+---+---+---+---+---+
             5 | . | b | B | b | b |
            ---+---+---+---+---+---+
             4 | . | . | . | . | . |
            ---+---+---+---+---+---+
             3 | . | . | b | . | . |
            ---+---+---+---+---+---+
             2 | . | . | . | . | . |
            ---+---+---+---+---+---+
             1 | r | r | R | r | r |
            ---+---+---+---+---+---+
               | a | b | c | d | e |
        */
        state.pawns[PlayerColor::Blue as usize] = 0x5808_0000;

        let cards = state.deck.get_player_cards(PlayerColor::Blue);
        // Cloning to avoid immutable borrow before mutable
        let tiger = cards[1].clone();

        let mov = Move {
            from: 12, // c3
            to: 22,   // c1
            figure: Figure::Pawn,
        };
        let mov_result = state.make_move(&mov, PlayerColor::Blue, 1);

        // Check if the game is won
        assert_eq!(mov_result, MoveResult::BlueWin);
        // c1 has a pawn
        assert_eq!(get_bit(state.pawns[PlayerColor::Blue as usize], 22), 1);
        // c3 does not have a pawn
        assert_eq!(get_bit(state.pawns[PlayerColor::Blue as usize], 12), 0);
        // Check card rotation
        assert_eq!(*state.deck.neutral_card(), tiger);
        // Check if red king is dead
        assert_eq!(state.kings[PlayerColor::Red as usize], 0);
    }

    #[test]
    fn king_in_temple_as_blue() {
        let deck = Deck::new([CRAB, RABBIT, DRAGON, TIGER, FROG]);
        let mut state = State::with_deck(deck);

        // Set the following state:
        /*
            ---+---+---+---+---+---+
             5 | b | b | . | b | b |
            ---+---+---+---+---+---+
             4 | . | . | . | . | . |
            ---+---+---+---+---+---+
             3 | . | . | B | . | . |
            ---+---+---+---+---+---+
             2 | . | . | . | R | . |
            ---+---+---+---+---+---+
             1 | r | r | . | r | r |
            ---+---+---+---+---+---+
               | a | b | c | d | e |
        */
        state.kings[PlayerColor::Blue as usize] = 0x0008_0000;
        state.kings[PlayerColor::Red as usize] = 0x0000_2000;

        let cards = state.deck.get_player_cards(PlayerColor::Blue);
        // Cloning to avoid immutable borrow before mutable
        let tiger = cards[1].clone();

        let mov = Move {
            from: 12, // c3
            to: 22,   // c1
            figure: Figure::King,
        };
        let mov_result = state.make_move(&mov, PlayerColor::Blue, 1);

        // Check if the game is won
        assert_eq!(mov_result, MoveResult::BlueWin);
        // a1 has a king
        assert_eq!(get_bit(state.kings[PlayerColor::Blue as usize], 22), 1);
        // a3 does not have a king
        assert_eq!(get_bit(state.kings[PlayerColor::Blue as usize], 12), 0);
        // Check card rotation
        assert_eq!(*state.deck.neutral_card(), tiger);
        // Check if red king is alive, but he lost
        assert!(state.kings[PlayerColor::Red as usize] > 0);
    }

    #[test]
    fn king_in_temple_as_red() {
        let deck = Deck::new([CRAB, RABBIT, DRAGON, TIGER, FROG]);
        let mut state = State::with_deck(deck);

        // Set the following state:
        /*
            ---+---+---+---+---+---+
             5 | b | b | . | b | b |
            ---+---+---+---+---+---+
             4 | . | . | R | . | . |
            ---+---+---+---+---+---+
             3 | . | . | B | . | . |
            ---+---+---+---+---+---+
             2 | . | . | . | . | . |
            ---+---+---+---+---+---+
             1 | r | r | . | r | r |
            ---+---+---+---+---+---+
               | a | b | c | d | e |
        */
        state.kings[PlayerColor::Blue as usize] = 0x0008_0000;
        state.kings[PlayerColor::Red as usize] = 0x0100_0000;

        let cards = state.deck.get_player_cards(PlayerColor::Red);
        // Cloning to avoid immutable borrow before mutable
        let crab = cards[0].clone();

        let mov = Move {
            from: 7, // c4
            to: 2,   // c5
            figure: Figure::King,
        };
        let mov_result = state.make_move(&mov, PlayerColor::Red, 0);

        // Check if the game is won
        assert_eq!(mov_result, MoveResult::RedWin);
        // c5 has a king
        assert_eq!(get_bit(state.kings[PlayerColor::Red as usize], 2), 1);
        // c4 does not have a king
        assert_eq!(get_bit(state.kings[PlayerColor::Red as usize], 7), 0);
        // Check card rotation
        assert_eq!(*state.deck.neutral_card(), crab);
        // Check if blue king is alive, but he lost
        assert!(state.kings[PlayerColor::Blue as usize] > 0);
    }
}
