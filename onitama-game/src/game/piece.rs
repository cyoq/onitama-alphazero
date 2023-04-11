use serde::{Deserialize, Serialize};

use super::player_color::PlayerColor;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PieceKind {
    Pawn,
    King,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Piece {
    pub color: PlayerColor,
    pub kind: PieceKind,
}

impl Piece {
    pub fn new(kind: PieceKind, color: PlayerColor) -> Self {
        Self { kind, color }
    }

    #[inline]
    pub fn red_king() -> Self {
        Self {
            kind: PieceKind::King,
            color: PlayerColor::Red,
        }
    }

    #[inline]
    pub fn red_pawn() -> Self {
        Self {
            kind: PieceKind::Pawn,
            color: PlayerColor::Red,
        }
    }

    #[inline]
    pub fn blue_king() -> Self {
        Self {
            kind: PieceKind::King,
            color: PlayerColor::Blue,
        }
    }

    #[inline]
    pub fn blue_pawn() -> Self {
        Self {
            kind: PieceKind::Pawn,
            color: PlayerColor::Blue,
        }
    }

    #[inline]
    pub const fn enemy(&self) -> PlayerColor {
        match self.color {
            PlayerColor::Red => PlayerColor::Blue,
            PlayerColor::Blue => PlayerColor::Red,
        }
    }
}
