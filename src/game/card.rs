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
}

pub trait CardRepresentation {
    fn represent_card(&self) -> String;
}

impl CardRepresentation for u32 {
    fn represent_card(&self) -> String {
        let s = format!("{:032b}", &self);

        let mut result = String::new();
        for (i, ch) in s.chars().take(25).enumerate() {
            match i {
                5 | 10 | 15 | 20 | 25 => {
                    result += &format!(
                        "\n{}",
                        match ch {
                            '0' => ".",
                            '1' => "X",
                            _ => "E",
                        }
                    );
                }
                12 => result += "O",
                _ => match ch {
                    '0' => result += ".",
                    '1' => result += "X",
                    _ => (),
                },
            }
        }
        return result;
    }
}

pub const TIGER: Card = Card {
    // 0 0 1 0 0   |  . . X . .
    // 0 0 0 0 0   |  . . . . .
    // 0 0 1 0 0   |  . . O . .
    // 0 0 1 0 0   |  . . X . .
    // 0 0 0 0 0   |  . . . . .
    // ---- Trailing ----
    // 0 0 0 0 0 0 0
    // -------
    // 0010 0000 0000 1000 0100 0000 0000 0000
    positions: 0x2008_4000,
    // 0 0 0 0 0   |  . . . . .
    // 0 0 1 0 0   |  . . X . .
    // 0 0 1 0 0   |  . . O . .
    // 0 0 0 0 0   |  . . . . .
    // 0 0 1 0 0   |  . . X . .
    // ---- Trailing ----
    // 0 0 0 0 0 0 0
    // -------
    // 0000 0001 0000 1000 0000 0010 0000 0000
    mirror: 0x0108_0200,
    player_color: PlayerColor::Blue,
    name: "Tiger",
};

pub const DRAGON: Card = Card {
    // 0 0 0 0 0   |  . . . . .
    // 1 0 0 0 1   |  X . . . X
    // 0 0 1 0 0   |  . . O . .
    // 0 1 0 1 0   |  . X . X .
    // 0 0 0 0 0   |  . . . . .
    // ---- Trailing ----
    // 0 0 0 0 0 0 0
    // -------
    // 0000 0100 0100 1000 1010 0000 0000 0000
    positions: 0x0448_A000,
    // 0 0 0 0 0   |  . . . . .
    // 0 1 0 1 0   |  . X . X .
    // 0 0 1 0 0   |  . . O . .
    // 1 0 0 0 1   |  X . . . X
    // 0 0 0 0 0   |  . . . . .
    // ---- Trailing ----
    // 0 0 0 0 0 0 0
    // -------
    // 0000 0010 1000 1001 0001 0000 0000 0000
    mirror: 0x0289_1000,
    player_color: PlayerColor::Red,
    name: "Dragon",
};

pub const FROG: Card = Card {
    // 0 0 0 0 0  |  . . . . .
    // 0 1 0 0 0  |  . X . . .
    // 1 0 1 0 0  |  X . O . .
    // 0 0 0 1 0  |  . . . X .
    // 0 0 0 0 0  |  . . . . .
    // ---- Trailing ----
    // 0 0 0 0 0 0 0
    // -------
    // 0000 0010 0010 1000 0010 0000 0000 0000
    positions: 0x0228_2000,
    // 0 0 0 0 0  |  . . . . .
    // 0 1 0 0 0  |  . X . . .
    // 0 0 1 0 1  |  . . O . X
    // 0 0 0 1 0  |  . . . X .
    // 0 0 0 0 0  |  . . . . .
    // ---- Trailing ----
    // 0 0 0 0 0 0 0
    // -------
    // 0000 0010 0000 1010 0010 0000 0000 0000
    mirror: 0x020A_2000,
    player_color: PlayerColor::Red,
    name: "Frog",
};

pub const RABBIT: Card = Card {
    // 0 0 0 0 0  |  . . . . .
    // 0 0 0 1 0  |  . . . X .
    // 0 0 1 0 1  |  . . O . X
    // 0 1 0 0 0  |  . X . . .
    // 0 0 0 0 0  |  . . . . .
    // ---- Trailing ----
    // 0 0 0 0 0 0 0
    // -------
    // 0000 0000 1000 1010 1000 0000 0000 0000
    positions: 0x008A_8000,
    // 0 0 0 0 0  |  . . . . .
    // 0 0 0 1 0  |  . . . X .
    // 1 0 1 0 0  |  X . O . .
    // 0 1 0 0 0  |  . X . . .
    // 0 0 0 0 0  |  . . . . .
    // ---- Trailing ----
    // 0 0 0 0 0 0 0
    // -------
    // 0000 0000 1010 1000 1000 0000 0000 0000
    mirror: 0x00A8_8000,
    player_color: PlayerColor::Red,
    name: "Rabbit",
};

pub const CRAB: Card = Card {
    /*
      0 0 0 0 0  |  . . . . .
      0 0 1 0 0  |  . . X . .
      1 0 1 0 1  |  X . O . X
      0 0 0 0 0  |  . . . . .
      0 0 0 0 0  |  . . . . .
      ---- Trailing ----
      0 0 0 0 0 0 0
      -------
      0000 0001 0010 1010 0000 0000 0000 0000
    */
    positions: 0x012A_0000,
    /*
      0 0 0 0 0  |  . . . . .
      0 0 0 0 0  |  . . . . .
      1 0 1 0 1  |  X . O . X
      0 0 1 0 0  |  . . X . .
      0 0 0 0 0  |  . . . . .
      ---- Trailing ----
      0 0 0 0 0 0 0
      -------
      0000 0000 0010 1010 0100 0000 0000 0000
    */
    mirror: 0x002A_4000,
    player_color: PlayerColor::Blue,
    name: "Crab",
};

pub const ORIGINAL_CARDS: [Card; 5] = [TIGER, DRAGON, FROG, RABBIT, CRAB];
