use game::{card::ORIGINAL_CARDS, deck::Deck};

use crate::game::card::CardRepresentation;

pub mod ai;
pub mod game;

fn main() {
    let deck = Deck::new(ORIGINAL_CARDS.clone());

    for card in deck.iter() {
        println!(
            "Card {}: \n{}\n",
            card.name,
            card.positions.represent_card()
        );
    }
}
