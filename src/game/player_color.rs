#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum PlayerColor {
    Red,
    Blue,
}

impl PlayerColor {
    #[inline]
    pub fn switch(&mut self) {
        *self = match self {
            PlayerColor::Red => PlayerColor::Blue,
            PlayerColor::Blue => PlayerColor::Red,
        };
    }
}
