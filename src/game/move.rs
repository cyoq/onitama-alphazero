use crate::common::from_2d_to_bitboard;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Move {
    pub from: u32,
    pub to: u32,
}

impl From<[(u32, u32); 2]> for Move {
    fn from(value: [(u32, u32); 2]) -> Self {
        let from = value[0];
        let to = value[1];
        Self {
            from: from_2d_to_bitboard(from),
            to: from_2d_to_bitboard(to),
        }
    }
}
