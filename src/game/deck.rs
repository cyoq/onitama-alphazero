use std::ops::Deref;

use super::card::Card;

#[derive(Debug, Clone)]
pub struct Deck {
    pub cards: [Card; 5],
}

impl Deck {
    pub fn new(deck: [Card; 5]) -> Self {
        Deck { cards: deck }
    }
}

impl Deref for Deck {
    type Target = [Card; 5];

    fn deref(&self) -> &Self::Target {
        &self.cards
    }
}
