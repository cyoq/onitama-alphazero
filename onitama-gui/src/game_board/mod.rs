pub mod cell;

use egui::Color32;
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

impl<'a> GameBoard<'a> {
    pub fn show(&self, ui: &mut egui::Ui) {
        let GameBoard { state, cell_size } = self;

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

                        ui.add(self::cell::Cell::new(row, col, bg_fill, *cell_size, None));
                    }
                    ui.end_row();
                }
            });
    }
}
