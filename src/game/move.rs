use std::fmt::Display;

use crate::common::from_2d_to_1d;

use super::figure::Figure;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Move {
    pub from: u32,
    pub to: u32,
    pub figure: Figure,
}

pub fn convert_idx_to_human_readable(idx: u32) -> String {
    let x = 5 - idx / 5;
    let y = idx % 5;
    let letter = match y {
        0 => 'a',
        1 => 'b',
        2 => 'c',
        3 => 'd',
        4 => 'e',
        _ => 'X',
    };
    format!("{}{}", letter, x)
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Move [from: {}, to: {}, with: {:?}]",
            convert_idx_to_human_readable(self.from),
            convert_idx_to_human_readable(self.to),
            self.figure
        ))
    }
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
