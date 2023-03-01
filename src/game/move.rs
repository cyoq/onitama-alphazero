use crate::common::from_2d_to_1d;

use super::figure::Figure;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Move {
    pub from: u32,
    pub to: u32,
    pub figure: Figure,
}

impl From<([(u32, u32); 2], Figure)> for Move {
    fn from(value: ([(u32, u32); 2], Figure)) -> Self {
        let mov = value.0;
        let figure = value.1;
        let from = mov[0];
        let to = mov[1];
        Self {
            from: from_2d_to_1d(from),
            to: from_2d_to_1d(to),
            figure,
        }
    }
}
