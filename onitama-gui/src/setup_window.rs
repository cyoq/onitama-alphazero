use egui::*;
use egui_extras::{Size, StripBuilder};
use onitama_game::game::{
    card::{Card, CARD_NAMES, ORIGINAL_CARDS},
    player_color::PlayerColor,
};

use crate::move_card::MoveCard;

const MOVE_CARD_CELL_SIZE: f32 = 18.;
const SETUP_WINDOW_WIDTH: f32 = 900.;
const SETUP_WINDOW_HEIGHT: f32 = 550.;

pub struct SetupWindow;

impl SetupWindow {
    pub fn show_setup(ctx: &Context, open: &mut bool) {
        Window::new("Game Setup")
            .open(open)
            .resizable(false)
            .min_width(SETUP_WINDOW_WIDTH)
            .min_height(SETUP_WINDOW_HEIGHT)
            .show(ctx, |ui| {
                ui.separator();
                Self::show_deck_panel(ui);
                ui.separator();
                Self::show_bottom_panel(ui);
            });
    }

    fn show_top_panel(&self, ui: &mut Ui) {}

    fn show_deck_panel(ui: &mut Ui) {
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
                                    let r = Self::add_card_to_ui(ui, card);
                                    r.context_menu(Self::nested_menus);
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
                                    let r = Self::add_card_to_ui(ui, card);
                                    r.context_menu(Self::nested_menus);
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

    fn add_card_to_ui(ui: &mut Ui, card: &Card) -> Response {
        let mut stroke_fill = Color32::BLACK;
        // if deck.get_card_idx(&card) == self.selected_card.card_idx {
        //     stroke_fill = match PlayerColor::Red {
        //         PlayerColor::Red => Color32::RED,
        //         PlayerColor::Blue => Color32::BLUE,
        //     }
        // }

        let response = ui.add(MoveCard {
            mirror: &false,
            card: card,
            name: CARD_NAMES[card.index],
            cell_size: MOVE_CARD_CELL_SIZE,
            stroke_fill,
        });

        if response.clicked() {
            tracing::debug!("Selected card index");
        }

        if response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        response
    }

    fn nested_menus(ui: &mut Ui) {
        if ui.button("Red").clicked() {
            ui.close_menu();
        }
        if ui.button("Blue").clicked() {
            ui.close_menu();
        }
        if ui.button("Neutral").clicked() {
            ui.close_menu();
        }
    }
}
