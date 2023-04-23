use onitama_game::{
    common::get_bit_array,
    game::{player_color::PlayerColor, state::State},
};

use tch::{IndexOp, Tensor};

#[derive(Debug, Clone, Copy)]
pub struct Options {
    pub kind: tch::Kind,
    pub device: tch::Device,
}

impl Options {
    pub fn new(options: (tch::Kind, tch::Device)) -> Self {
        let (kind, device) = options;
        Self { kind, device }
    }

    pub fn to_tuple(&self) -> (tch::Kind, tch::Device) {
        (self.kind, self.device)
    }
}

pub fn create_tensor_from_state(
    state: &State,
    player_color: PlayerColor,
    options: (tch::Kind, tch::Device),
) -> Tensor {
    let state_tensor = Tensor::zeros(&[21, 5, 5], options);
    if player_color == PlayerColor::Blue {
        // Set player color channel to Blue color, i.e. 1 if Blue, leave as 0 if player is Red
        state_tensor
            .i((20, .., ..))
            .copy_(&Tensor::ones(&[5, 5], options));
    }

    let red_pawns = get_bit_array::<i64>(state.pawns[PlayerColor::Red as usize]);
    state_tensor
        .i((0, .., ..))
        .copy_(&Tensor::of_slice(&red_pawns).reshape(&[5, 5]));

    let red_king = get_bit_array::<i64>(state.kings[PlayerColor::Red as usize]);
    state_tensor
        .i((1, .., ..))
        .copy_(&Tensor::of_slice(&red_king).reshape(&[5, 5]));

    let blue_pawns = get_bit_array::<i64>(state.pawns[PlayerColor::Blue as usize]);
    state_tensor
        .i((2, .., ..))
        .copy_(&Tensor::of_slice(&blue_pawns).reshape(&[5, 5]));

    let blue_king = get_bit_array::<i64>(state.kings[PlayerColor::Blue as usize]);
    state_tensor
        .i((3, .., ..))
        .copy_(&Tensor::of_slice(&blue_king).reshape(&[5, 5]));

    // let cards = state.deck.get_player_cards(player_color);
    // let card1_idx = cards[0].index;
    // let (col, row) = ((card1_idx / 5) as i64, (card1_idx % 5) as i64);
    // *(&mut state_tensor.i((4, col, row))) += 1;

    // let card2_idx = cards[1].index;
    // let (col, row) = ((card2_idx / 5) as i64, (card2_idx % 5) as i64);
    // *(&mut state_tensor.i((4, col, row))) += 1;

    let cards = state.deck.get_player_cards(player_color);
    let card1_idx = cards[0].index;
    state_tensor
        .i(((card1_idx + 4) as i64, .., ..))
        .copy_(&Tensor::ones(&[5, 5], options));

    let card2_idx = cards[1].index;
    state_tensor
        .i(((card2_idx + 4) as i64, .., ..))
        .copy_(&Tensor::ones(&[5, 5], options));

    state_tensor
}
