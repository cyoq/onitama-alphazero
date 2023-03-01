#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveResult {
    Capture,
    RedWin,
    BlueWin,
    InProgress,
}
