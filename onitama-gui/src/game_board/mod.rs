pub mod cell;

use egui::{Color32, Widget};
use onitama_game::{
    common::{from_2d_to_1d, get_bit},
    game::{player_color::PlayerColor, state::State},
};

pub const BG_FILL: Color32 = Color32::WHITE;
pub const BG_TEMPLE: Color32 = Color32::GRAY;
pub const BG_BLUE: Color32 = Color32::BLUE;
pub const BG_RED: Color32 = Color32::RED;

/// A representation of a game board
pub struct GameBoard<'a> {
    /// State of the current game
    pub state: &'a State,
    /// A size of the cell
    pub cell_size: f32,
}

impl<'a> Widget for GameBoard<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let GameBoard { state, cell_size } = self;
        // Widget code can be broken up in four steps:
        //  1. Decide a size for the widget
        //  2. Allocate space for it
        //  3. Handle interactions with the widget (if any)
        //  4. Paint the widget

        // 1. Deciding widget size:
        // You can query the `ui` how much space is available,
        // but in this example we have a fixed size widget based on the height of a standard button:
        let desired_size = egui::vec2(cell_size * 5., cell_size * 5.);

        // 2. Allocating space:
        // This is where we get a region of the screen assigned.
        // We also tell the Ui to sense clicks in the allocated region.
        // let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
        let mut response = ui.allocate_response(egui::vec2(0., 0.), egui::Sense::click());

        // 3. Interact: Time to check for clicks!
        if response.clicked() {
            response.mark_changed(); // report back that the value changed
        }

        // Attach some meta-data to the response which can be used by screen readers:
        // response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, *on, ""));

        // 4. Paint!
        // Make sure we need to paint:
        // if !ui.is_rect_visible(rect) {
        //     return response;
        // }

        let mut bg_fill = BG_FILL;
        egui::Grid::new("game_board")
            .min_col_width(0.)
            .min_row_height(0.)
            .spacing(egui::vec2(0., 0.))
            .show(ui, |ui| {
                for row in 0..5 {
                    for col in 0..5 {
                        let coords = from_2d_to_1d((row, col)) as usize;
                        let red_pos = get_bit(state.pawns[PlayerColor::Red as usize], coords)
                            | get_bit(state.kings[PlayerColor::Red as usize], coords);
                        let blue_pos = get_bit(state.pawns[PlayerColor::Blue as usize], coords)
                            | get_bit(state.kings[PlayerColor::Blue as usize], coords);

                        if red_pos == 1 {
                            bg_fill = BG_RED;
                        } else if blue_pos == 1 {
                            bg_fill = BG_BLUE;
                        } else {
                            bg_fill = BG_FILL;
                        }

                        ui.add(self::cell::Cell::new(row, col, bg_fill, cell_size));
                    }
                    ui.end_row();
                }
            });

        // All done! Return the interaction response so the user can check what happened
        // (hovered, clicked, ...) and maybe show a tooltip:
        response
    }
}
