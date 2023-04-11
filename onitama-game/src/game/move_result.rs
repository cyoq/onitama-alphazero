use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MoveResult {
    Capture,
    RedWin,
    BlueWin,
    InProgress,
}

impl MoveResult {
    #[inline]
    pub fn is_win(&self) -> bool {
        *self == MoveResult::RedWin || *self == MoveResult::BlueWin
    }
}
