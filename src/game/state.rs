use super::{deck::Deck, move_result::MoveResult, player_color::PlayerColor, r#move::Move};

// Initial figure positions
// 00000
// 00000
// 00000
// 00000
// 00100
// 0000000 - Trailing zeroes
pub const RED_KING: u32 = 0x0000_0200;
// 00100
// 00000
// 00000
// 00000
// 00000
// 0000000 - Trailing zeroes
pub const BLUE_KING: u32 = 0x2000_0000;
// 11011
// 00000
// 00000
// 00000
// 00000
// 0000000 - Trailing zeroes
pub const BLUE_PAWNS: u32 = 0xD800_0000;
// 00000
// 00000
// 00000
// 00000
// 11011
// 0000000 - Trailing zeroes
pub const RED_PAWNS: u32 = 0x0000_0D80;

#[derive(Debug, Clone)]
pub struct State {
    pub deck: Deck,
    pub kings: [u32; 2],
    pub pawns: [u32; 2],
}

impl State {
    pub fn new(deck: Deck) -> Self {
        State {
            deck,
            kings: [RED_KING, BLUE_KING],
            pawns: [RED_PAWNS, BLUE_PAWNS],
        }
    }

    pub fn display(&self) {}

    pub fn make_move(&mut self, mov: Move, player_color: PlayerColor) -> MoveResult {
        MoveResult::InProgress
    }
}
