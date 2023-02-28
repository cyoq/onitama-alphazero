use game::{
    card::ORIGINAL_CARDS,
    deck::Deck,
    state::{State, BLUE_KING_IDX, BLUE_PAWNS_IDX, RED_KING_IDX, RED_PAWNS_IDX},
};

use crate::{
    common::get_bit,
    game::card::{CardRepresentation, ATTACK_MAPS},
};

pub mod ai;
pub mod common;
pub mod game;

fn main() {
    // let deck = Deck::new(ORIGINAL_CARDS.clone());

    // for card in deck.iter() {
    //     println!(
    //         "Card {}: \n{}\n",
    //         card.name,
    //         card.positions.represent_card()
    //     );
    // }
    let state = State::new();

    let result = state.display();
    println!("{}", result);

    let attack_map = ATTACK_MAPS[1][0][22];
    println!("{}", attack_map);
}
