use std::ops::{Deref, DerefMut};

use rand::{rngs::SmallRng, seq::SliceRandom, SeedableRng};
use serde::{Deserialize, Serialize};

use crate::common::get_bit;

use super::{
    card::{Card, CARD_NAMES, ORIGINAL_CARDS},
    player_color::PlayerColor,
};

// positions in the deck array, corresponding to the card used by the owner
pub const RED_CARD1: usize = 0;
pub const RED_CARD2: usize = 1;
pub const BLUE_CARD1: usize = 2;
pub const BLUE_CARD2: usize = 3;
pub const NEUTRAL: usize = 4;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deck {
    pub cards: [Card; 5],
}

impl Deck {
    pub fn new(deck: [Card; 5]) -> Self {
        Deck { cards: deck }
    }

    #[inline]
    /// Return if the card should be mirrored. Mirroring is default to the blue player
    pub fn is_mirrored(&self, card: &Card) -> Option<bool> {
        match self.cards.iter().position(|c| c == card) {
            Some(pos) => Some(pos == BLUE_CARD1 || pos == BLUE_CARD2), // blue player cards positions
            None => None,
        }
    }

    #[inline]
    pub fn get_player_cards(&self, player_color: PlayerColor) -> [&Card; 2] {
        match player_color {
            PlayerColor::Red => [&self.cards[RED_CARD1], &self.cards[RED_CARD2]],
            PlayerColor::Blue => [&self.cards[BLUE_CARD1], &self.cards[BLUE_CARD2]],
        }
    }

    #[inline]
    pub fn get_player_cards_idx(&self, player_color: PlayerColor) -> [usize; 2] {
        match player_color {
            PlayerColor::Red => [RED_CARD1, RED_CARD2],
            PlayerColor::Blue => [BLUE_CARD1, BLUE_CARD2],
        }
    }

    #[inline]
    pub fn neutral_card(&self) -> &Card {
        &self.cards[NEUTRAL]
    }

    #[inline]
    pub fn get_card_idx(&self, card: &Card) -> Option<usize> {
        self.cards.iter().position(|c| c == card)
    }

    #[inline]
    pub fn get_card(&self, card_idx: usize) -> &Card {
        assert!(card_idx < 5);
        &self.cards[card_idx]
    }

    #[inline]
    pub fn get_card_owner(&self, card: &Card) -> Option<PlayerColor> {
        match self.cards.iter().position(|c| c == card) {
            Some(pos) => match pos {
                RED_CARD1 | RED_CARD2 => Some(PlayerColor::Red),
                BLUE_CARD1 | BLUE_CARD2 => Some(PlayerColor::Blue),
                _ => None,
            },
            None => None,
        }
    }

    /// Rotates cards between the used one and the neutral
    ///
    /// Be aware that the index should only be from 0 to 1,
    #[inline]
    pub fn rotate(&mut self, idx: usize) {
        assert!(idx < 4);
        self.cards.swap(idx, NEUTRAL);
    }

    pub fn display(&self) -> String {
        let border = String::from("---+---+---+---+---+---+\n");
        let mut result = border.clone();
        let length: usize = 25;
        for (i, card) in self.cards.iter().enumerate() {
            let positions = match i {
                0 | 1 => card.positions,
                2 | 3 => card.mirror,
                _ => card.positions,
            };
            let name = match i {
                0 | 1 => "PLAYER1",
                2 | 3 => "PLAYER2",
                4 => "NEUTRAL",
                _ => "UNKNOWN",
            };
            for n in 0..length {
                // Add a number if it is as start of a string
                if n % 5 == 0 {
                    result += &format!(" {} ", (5 - n / 5).to_string());
                }

                // Check cell type
                if get_bit(positions, n) == 1 {
                    result += "| X ";
                } else if n == 12 {
                    result += "| O ";
                } else {
                    result += "| . ";
                }

                // Add a border and ending wall if it is an end of the line
                if (n + 1) % 5 == 0 {
                    result += "|\n";
                    result += &border;
                }
            }

            // Add a column identifier
            result += "   | a | b | c | d | e |\n";
            result += &format!("   {}({})\n\n", name, CARD_NAMES[card.index]);
        }

        result
    }
}

impl Default for Deck {
    fn default() -> Self {
        let mut rng = SmallRng::from_entropy();
        let mut shuffled = ORIGINAL_CARDS.clone();
        shuffled.shuffle(&mut rng);

        Self {
            cards: shuffled[0..5]
                .try_into()
                .expect("Deck must have 5 random cards"),
        }
    }
}

impl Deref for Deck {
    type Target = [Card; 5];

    fn deref(&self) -> &Self::Target {
        &self.cards
    }
}

impl DerefMut for Deck {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cards
    }
}
