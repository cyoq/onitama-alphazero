// TODO: find a better way to index the array with enum.
// Probably create separate arrays and match them with enum
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

    #[inline]
    pub fn enemy(&self) -> PlayerColor {
        match self {
            PlayerColor::Red => PlayerColor::Blue,
            PlayerColor::Blue => PlayerColor::Red,
        }
    }
}
