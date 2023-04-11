use std::{error::Error, fmt::Display};

use serde::{Deserialize, Serialize};

use crate::common::from_2d_to_1d;

use super::piece::PieceKind;

#[derive(Debug, Clone)]
pub struct NotationError(String);

impl Error for NotationError {}

impl Display for NotationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Deserialize)]
pub struct Move {
    pub from: u32,
    pub to: u32,
    pub piece: PieceKind,
}

impl Move {
    pub fn convert_to_2d(value: u32) -> (u32, u32) {
        let col = value % 5;
        let row = value / 5;
        (row, col)
    }

    /// Converts index from 0 to 24 to the algebraic notation
    pub fn convert_idx_to_notation(idx: u32) -> String {
        let x = 5 - idx / 5;
        let y = idx % 5;
        let letter = match y {
            0 => 'a',
            1 => 'b',
            2 => 'c',
            3 => 'd',
            4 => 'e',
            _ => panic!("Incorrect letter was produced from {}", y),
        };
        format!("{}{}", letter, x)
    }

    /// Converts algebraic notation to the index(0..24) of the board
    pub fn convert_notation_to_idx(notation: &str) -> Result<u32, NotationError> {
        let col = Move::get_col_from_notation(notation)?;
        let row = Move::get_row_from_notation(notation)?;

        Ok(from_2d_to_1d((row, col)))
    }

    fn get_col_from_notation(notation: &str) -> Result<u32, NotationError> {
        if let Some(letter) = notation.chars().nth(0) {
            match letter {
                'a' => Ok(0),
                'b' => Ok(1),
                'c' => Ok(2),
                'd' => Ok(3),
                'e' => Ok(4),
                _ => Err(NotationError(String::from(
                    "First character is not a letter a, b, c, d or e!",
                ))),
            }
        } else {
            Err(NotationError(String::from(
                "Character at index 0 was not found!",
            )))
        }
    }

    fn get_row_from_notation(notation: &str) -> Result<u32, NotationError> {
        if let Some(num) = notation.chars().nth(1) {
            match num.to_digit(10) {
                Some(x) => match x {
                    // subtracting five to get the numbering starting from the bottom of the board
                    1..=5 => Ok(5 - x),
                    _ => Err(NotationError(String::from(
                        "A second character must be in range 1..5!",
                    ))),
                },
                None => Err(NotationError(format!(
                    "It was not possible to parse this value: {}",
                    num
                ))),
            }
        } else {
            Err(NotationError(String::from(
                "Character at index 1 was not found!",
            )))
        }
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Move [from: {}, to: {}, with: {:?}]",
            Move::convert_idx_to_notation(self.from),
            Move::convert_idx_to_notation(self.to),
            self.piece
        ))
    }
}

impl From<([(u32, u32); 2], PieceKind)> for Move {
    fn from(value: ([(u32, u32); 2], PieceKind)) -> Self {
        let mov = value.0;
        let figure = value.1;
        let from = mov[0];
        let to = mov[1];
        Self {
            from: from_2d_to_1d(from),
            to: from_2d_to_1d(to),
            piece: figure,
        }
    }
}
