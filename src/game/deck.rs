use std::ops::{Deref, DerefMut};

use rand::{seq::SliceRandom, thread_rng};

use super::{
    card::{Card, ORIGINAL_CARDS},
    player_color::PlayerColor,
};

// positions in the deck array, corresponding to the card used by the owner
pub const RED_CARD1: usize = 0;
pub const RED_CARD2: usize = 1;
pub const BLUE_CARD1: usize = 2;
pub const BLUE_CARD2: usize = 3;
pub const NEUTRAL: usize = 4;

#[derive(Debug, Clone)]
pub struct Deck {
    pub cards: [Card; 5],
}

impl Deck {
    pub fn new(deck: [Card; 5]) -> Self {
        Deck { cards: deck }
    }

    #[inline]
    pub fn get_player_cards(&self, player_color: &PlayerColor) -> [&Card; 2] {
        if *player_color == PlayerColor::Red {
            return [&self.cards[RED_CARD1], &self.cards[RED_CARD2]];
        }
        return [&self.cards[BLUE_CARD1], &self.cards[BLUE_CARD2]];
    }

    #[inline]
    pub fn neutral_card(&self) -> &Card {
        &self.cards[NEUTRAL]
    }
}

impl Default for Deck {
    fn default() -> Self {
        let mut rng = thread_rng();
        let mut shuffled = ORIGINAL_CARDS.clone();
        shuffled.shuffle(&mut rng);

        Self {
            cards: shuffled[0..5]
                .try_into()
                .expect("Deck should have 5 random cards"),
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
