use super::r#move::Move;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DoneMove {
    pub mov: Move,
    pub used_card_idx: usize,
}
