#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum PlayerColor {
    Red = 0,
    Blue = 1,
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
