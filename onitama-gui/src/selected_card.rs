use onitama_game::game::{card::Card, deck::Deck};

#[derive(Debug)]
pub struct SelectedCard {
    pub card_idx: Option<usize>,
    pub changed: bool,
}

impl Default for SelectedCard {
    fn default() -> Self {
        Self {
            card_idx: None,
            changed: false,
        }
    }
}

impl SelectedCard {
    pub fn set(&mut self, idx: Option<usize>) {
        self.card_idx = idx;
        self.changed = true;
    }

    pub fn update(&mut self, selected_card: &Card, deck: &Deck) {
        let idx = deck.get_card_idx(selected_card);
        match idx {
            Some(idx) => {
                self.card_idx = match self.card_idx {
                    Some(cidx) => {
                        if cidx != idx {
                            self.changed = true;
                            Some(idx)
                        } else {
                            self.changed = false;
                            Some(cidx)
                        }
                    }
                    None => {
                        self.changed = true;
                        Some(idx)
                    }
                }
            }
            None => (),
        }
    }
}
