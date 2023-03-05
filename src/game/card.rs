use super::player_color::PlayerColor;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Card {
    /// The configuration of the card
    pub positions: u32,
    /// Mirrored version of the card for the second player
    pub mirror: u32,
    /// Starting player color
    pub player_color: PlayerColor,
    /// Name of the card to display
    pub name: &'static str,
    /// unique number for a card to identify it in the attack lookup table
    pub index: usize,
}

pub const TIGER: Card = Card {
    // 0 0 1 0 0   |  . . X . .
    // 0 0 0 0 0   |  . . . . .
    // 0 0 0 0 0   |  . . O . .
    // 0 0 1 0 0   |  . . X . .
    // 0 0 0 0 0   |  . . . . .
    // ---- Trailing ----
    // 0 0 0 0 0 0 0
    // -------
    // 0010 0000 0000 1000 0100 0000 0000 0000
    positions: 0x2000_4000,
    // 0 0 0 0 0   |  . . . . .
    // 0 0 1 0 0   |  . . X . .
    // 0 0 0 0 0   |  . . O . .
    // 0 0 0 0 0   |  . . . . .
    // 0 0 1 0 0   |  . . X . .
    // ---- Trailing ----
    // 0 0 0 0 0 0 0
    // -------
    // 0000 0001 0000 1000 0000 0010 0000 0000
    mirror: 0x0100_0200,
    player_color: PlayerColor::Blue,
    name: "Tiger",
    index: 0,
};

pub const DRAGON: Card = Card {
    // 0 0 0 0 0   |  . . . . .
    // 1 0 0 0 1   |  X . . . X
    // 0 0 0 0 0   |  . . O . .
    // 0 1 0 1 0   |  . X . X .
    // 0 0 0 0 0   |  . . . . .
    // ---- Trailing ----
    // 0 0 0 0 0 0 0
    // -------
    // 0000 0100 0100 1000 1010 0000 0000 0000
    positions: 0x0440_A000,
    // 0 0 0 0 0   |  . . . . .
    // 0 1 0 1 0   |  . X . X .
    // 0 0 0 0 0   |  . . O . .
    // 1 0 0 0 1   |  X . . . X
    // 0 0 0 0 0   |  . . . . .
    // ---- Trailing ----
    // 0 0 0 0 0 0 0
    // -------
    // 0000 0010 1000 1001 0001 0000 0000 0000
    mirror: 0x0281_1000,
    player_color: PlayerColor::Red,
    name: "Dragon",
    index: 1,
};

pub const FROG: Card = Card {
    // 0 0 0 0 0  |  . . . . .
    // 0 1 0 0 0  |  . X . . .
    // 1 0 0 0 0  |  X . O . .
    // 0 0 0 1 0  |  . . . X .
    // 0 0 0 0 0  |  . . . . .
    // ---- Trailing ----
    // 0 0 0 0 0 0 0
    // -------
    // 0000 0010 0010 0000 0010 0000 0000 0000
    positions: 0x0220_2000,
    // 0 0 0 0 0  |  . . . . .
    // 0 1 0 0 0  |  . X . . .
    // 0 0 0 0 1  |  . . O . X
    // 0 0 0 1 0  |  . . . X .
    // 0 0 0 0 0  |  . . . . .
    // ---- Trailing ----
    // 0 0 0 0 0 0 0
    // -------
    // 0000 0010 0000 1010 0010 0000 0000 0000
    mirror: 0x0202_2000,
    player_color: PlayerColor::Red,
    name: "Frog",
    index: 2,
};

pub const RABBIT: Card = Card {
    // 0 0 0 0 0  |  . . . . .
    // 0 0 0 1 0  |  . . . X .
    // 0 0 0 0 1  |  . . O . X
    // 0 1 0 0 0  |  . X . . .
    // 0 0 0 0 0  |  . . . . .
    // ---- Trailing ----
    // 0 0 0 0 0 0 0
    // -------
    // 0000 0000 1000 0010 1000 0000 0000 0000
    positions: 0x0082_8000,
    // 0 0 0 0 0  |  . . . . .
    // 0 0 0 1 0  |  . . . X .
    // 1 0 0 0 0  |  X . O . .
    // 0 1 0 0 0  |  . X . . .
    // 0 0 0 0 0  |  . . . . .
    // ---- Trailing ----
    // 0 0 0 0 0 0 0
    // -------
    // 0000 0000 1010 0000 1000 0000 0000 0000
    mirror: 0x00A0_8000,
    player_color: PlayerColor::Red,
    name: "Rabbit",
    index: 3,
};

pub const CRAB: Card = Card {
    /*
      0 0 0 0 0  |  . . . . .
      0 0 1 0 0  |  . . X . .
      1 0 0 0 1  |  X . O . X
      0 0 0 0 0  |  . . . . .
      0 0 0 0 0  |  . . . . .
      ---- Trailing ----
      0 0 0 0 0 0 0
      -------
      0000 0001 0010 0010 0000 0000 0000 0000
    */
    positions: 0x0122_0000,
    /*
      0 0 0 0 0  |  . . . . .
      0 0 0 0 0  |  . . . . .
      1 0 0 0 1  |  X . O . X
      0 0 1 0 0  |  . . X . .
      0 0 0 0 0  |  . . . . .
      ---- Trailing ----
      0 0 0 0 0 0 0
      -------
      0000 0000 0010 0010 0100 0000 0000 0000
    */
    mirror: 0x0022_4000,
    player_color: PlayerColor::Blue,
    name: "Crab",
    index: 4,
};

pub const ORIGINAL_CARDS: [Card; 5] = [TIGER, DRAGON, FROG, RABBIT, CRAB];

pub const ATTACK_MAPS: [[[u32; 25]; 5]; 2] = generate_attack_maps();

/*
    1 0 0 0 0
    1 0 0 0 0
    1 0 0 0 0
    1 0 0 0 0
    1 0 0 0 0
    ---- Trailing ----
    0 0 0 0 0 0 0
*/
const FILE_A: u32 = 0x8421_0800;
/*
    0 0 0 0 1
    0 0 0 0 1
    0 0 0 0 1
    0 0 0 0 1
    0 0 0 0 1
    ---- Trailing ----
    0 0 0 0 0 0 0
*/
const FILE_E: u32 = 0x0842_1080;
/*
    1 1 0 0 0
    1 1 0 0 0
    1 1 0 0 0
    1 1 0 0 0
    1 1 0 0 0
    ---- Trailing ----
    0 0 0 0 0 0 0
*/
const FILE_AB: u32 = 0xC631_8C00;
/*
    0 0 0 1 1
    0 0 0 1 1
    0 0 0 1 1
    0 0 0 1 1
    0 0 0 1 1
    ---- Trailing ----
    0 0 0 0 0 0 0
*/
const FILE_DE: u32 = 0x18C6_3180;

/// Generates all attack maps for all players, cards and positions
const fn generate_attack_maps() -> [[[u32; 25]; 5]; 2] {
    let mut result = [[[0u32; 25]; 5]; 2];
    let mut player = 0;
    let mut card_idx = 0;

    // Need to use while to support const fn
    // for loops are not stabilized yet
    while player < 2 {
        while card_idx < ORIGINAL_CARDS.len() {
            let card = ORIGINAL_CARDS[card_idx];
            let mut positions = card.positions;
            if player == PlayerColor::Blue as usize {
                positions = card.mirror;
            }
            result[player][card.index] = generate_attack_maps_for_card(positions);
            card_idx += 1;
        }
        player += 1;
        card_idx = 0;
    }
    return result;
}

/// Attack map will be generated as following:
/// All the cards are defined that the figure stays in the center 'O':
/// . . . . .
/// . . X . .
/// X . O . X
/// . . . . .
/// . . . . .
/// Then the whole mask will be shifted to the left and right applying to the position
/// Then if the position is on the edges, it is going to be masked by files
/// in order to remove unnecessary bits
const fn generate_attack_maps_for_card(card: u32) -> [u32; 25] {
    // Create an array that will hold the mask for each position on the board
    let mut attack_map = [0u32; 25];
    // Set the center as the card positions
    attack_map[12] = card;

    // Start from 1, since center is already set
    let mut n = 1;
    // Need to use while, because for is not stabilized in const functions
    while n < 13 {
        // Make shifts
        // Use mask to get first 25 bits only
        let mut left = (card << n) & 0xFFFF_FF80;
        let mut right = (card >> n) & 0xFFFF_FF80;

        // remove everything on file E when position is on file B
        // and remove on file A when position is on file D
        match n % 5 {
            1 => {
                // remove everything on file E when position is on file B
                // and remove on file A when position is on file D
                left &= !FILE_E;
                right &= !FILE_A;
            }
            2 => {
                // remove everything on file D & E when position is on file A
                // and remove on file A & B when position is on file E
                left &= !FILE_DE;
                right &= !FILE_AB;
            }
            3 => {
                // remove everything on file A & B when position is on file E
                // and remove on file D & E when position is on file A
                left &= !FILE_AB;
                right &= !FILE_DE;
            }
            4 => {
                // remove everything on file A when position is on file D
                // and remove on file E when position is on file B
                left &= !FILE_A;
                right &= !FILE_E;
            }
            _ => (),
        };

        // Save the mask for the positions
        attack_map[12 - n] = left;
        attack_map[12 + n] = right;
        n += 1;
    }
    return attack_map;
}
