use game::state::State;

use crate::game::card::{ATTACK_MAPS, DRAGON};

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

    let attack_map = ATTACK_MAPS[1][DRAGON.index][2];
    println!("{}", attack_map);
}
