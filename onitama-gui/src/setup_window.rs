use egui::*;
use egui_extras::{Size, StripBuilder};
use onitama_game::game::card::{Card, CARD_NAMES, ORIGINAL_CARDS};

use crate::move_card::MoveCard;

const MOVE_CARD_CELL_SIZE: f32 = 18.;
const SETUP_WINDOW_WIDTH: f32 = 900.;
const SETUP_WINDOW_HEIGHT: f32 = 550.;
const COLOR_CHOICE_ORDER: [CardColor; 5] = [
    CardColor::Red,
    CardColor::Red,
    CardColor::Blue,
    CardColor::Blue,
    CardColor::Yellow,
];

enum CardColor {
    Red,
    Blue,
    // For neutral card
    Yellow,
}

impl CardColor {
    pub fn color(&self) -> Color32 {
        match self {
            CardColor::Red => Color32::RED,
            CardColor::Blue => Color32::BLUE,
            CardColor::Yellow => Color32::LIGHT_YELLOW,
        }
    }
}

pub struct SetupWindow<'a> {
    selected_cards: &'a mut [Option<Card>; 5],
}

impl<'a> SetupWindow<'a> {
    pub fn new(selected_cards: &'a mut [Option<Card>; 5]) -> Self {
        Self { selected_cards }
    }

    pub fn show_setup(&mut self, ctx: &Context, open: &mut bool) {
        Window::new("Game Setup")
            .open(open)
            .resizable(false)
            .min_width(SETUP_WINDOW_WIDTH)
            .min_height(SETUP_WINDOW_HEIGHT)
            .show(ctx, |ui| {
                self.show_top_panel(ui);
                ui.separator();
                self.show_deck_panel(ui);
                ui.separator();
                Self::show_bottom_panel(ui);
            });
    }

    fn show_top_panel(&self, ui: &mut Ui) {
        ui.label("Top panel");
    }

    fn show_deck_panel(&mut self, ui: &mut Ui) {
        let deck = &ORIGINAL_CARDS;

        StripBuilder::new(ui)
            // Sizes for the card rows
            .size(Size::exact(130.))
            .size(Size::exact(130.))
            // Signal that strips will represent rows
            .vertical(|mut strip| {
                // Textual information strip
                // strip.cell(|ui| {
                //     ui.vertical_centered(|ui| {});
                // });
                // strip builder that will separate row into 8 columns
                strip.strip(|builder| {
                    builder.sizes(Size::remainder(), 8).horizontal(|mut strip| {
                        for card in deck.iter().take(8) {
                            strip.cell(|ui| {
                                ui.vertical_centered(|ui| {
                                    let r = self.add_card_to_ui(ui, card);
                                    r.context_menu(|ui| self.nested_menus(ui, card));
                                });
                            });
                        }
                    });
                });
                // Last row with 8 columns
                strip.strip(|builder| {
                    builder.sizes(Size::remainder(), 8).horizontal(|mut strip| {
                        for card in deck.iter().skip(8) {
                            strip.cell(|ui| {
                                ui.vertical_centered(|ui| {
                                    let r = self.add_card_to_ui(ui, card);
                                    r.context_menu(|ui| self.nested_menus(ui, card));
                                });
                            });
                        }
                    });
                });
            });
    }

    fn show_bottom_panel(ui: &mut Ui) {
        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
            ui.button("Start a game");
            ui.checkbox(&mut true, "Save my choice");
        });
    }

    fn add_card_to_ui(&mut self, ui: &mut Ui, card: &Card) -> Response {
        let pos = self.selected_cards.iter().position(|&c| c == Some(*card));
        let stroke_fill = match pos {
            Some(idx) => COLOR_CHOICE_ORDER[idx].color(),
            None => Color32::BLACK,
        };

        let response = ui.add(MoveCard {
            mirror: &false,
            card,
            name: CARD_NAMES[card.index],
            cell_size: MOVE_CARD_CELL_SIZE,
            stroke_fill,
        });

        response
    }

    fn select_card(&mut self, card: &'a Card, color: CardColor) {
        let pos = self.selected_cards.iter().position(|&c| c == Some(*card));

        if let Some(idx) = pos {
            self.selected_cards[idx] = None;
        }

        match color {
            CardColor::Red => match (self.selected_cards[0], self.selected_cards[1]) {
                (Some(_), None) => self.selected_cards[1] = Some(*card),
                (_, _) => self.selected_cards[0] = Some(*card),
            },
            CardColor::Blue => match (self.selected_cards[2], self.selected_cards[3]) {
                (Some(_), None) => self.selected_cards[3] = Some(*card),
                (_, _) => self.selected_cards[2] = Some(*card),
            },
            CardColor::Yellow => self.selected_cards[4] = Some(*card),
        }
    }

    fn deselect_card(&mut self, card: &Card) {
        let pos = self.selected_cards.iter().position(|&c| c == Some(*card));
        match pos {
            Some(idx) => self.selected_cards[idx] = None,
            None => (),
        }
    }

    fn nested_menus(&mut self, ui: &mut Ui, card: &'a Card) {
        if ui.button("Red").clicked() {
            self.select_card(card, CardColor::Red);
            ui.close_menu();
        }

        if ui.button("Blue").clicked() {
            self.select_card(card, CardColor::Blue);
            ui.close_menu();
        }

        if ui.button("Neutral").clicked() {
            self.select_card(card, CardColor::Yellow);
            ui.close_menu();
        }

        let is_selected = self.selected_cards.contains(&Some(*card));
        if is_selected {
            if ui.button("Deselect").clicked() {
                self.deselect_card(card);
                ui.close_menu();
            }
        }
    }
}
