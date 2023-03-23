#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
