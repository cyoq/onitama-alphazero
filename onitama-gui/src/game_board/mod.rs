pub mod cell;

use eframe::epaint::ahash::HashMap;
use egui::Color32;
use onitama_game::{
    common::{from_2d_to_1d, get_bit},
    game::{player_color::PlayerColor, state::State},
};

use crate::{image::Image, onitama::Figure};

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
    /// images to display
    pub images: &'a HashMap<Figure, Image>,
}

impl<'a> GameBoard<'a> {
    pub fn show(&self, ui: &mut egui::Ui) {
        let GameBoard {
            state,
            cell_size,
            images,
        } = self;

        let bg_fill = BG_FILL;

        egui::Grid::new("game_board")
            .min_col_width(0.)
            .min_row_height(0.)
            .spacing(egui::vec2(0., 0.))
            .show(ui, |ui| {
                for row in 0..5 {
                    for col in 0..5 {
                        let mut image = None;
                        let coords = from_2d_to_1d((row, col)) as usize;
                        let red_pawn = get_bit(state.pawns[PlayerColor::Red as usize], coords);
                        let red_king = get_bit(state.kings[PlayerColor::Red as usize], coords);
                        let blue_pawn = get_bit(state.pawns[PlayerColor::Blue as usize], coords);
                        let blue_king = get_bit(state.kings[PlayerColor::Blue as usize], coords);

                        if red_pawn == 1 {
                            image = Some(&images.get(&Figure::RedPawn).unwrap().image);
                        } else if blue_pawn == 1 {
                            image = Some(&images.get(&Figure::BluePawn).unwrap().image);
                        } else if red_king == 1 {
                            image = Some(&images.get(&Figure::RedKing).unwrap().image);
                        } else if blue_king == 1 {
                            image = Some(&images.get(&Figure::BlueKing).unwrap().image);
                        }

                        ui.add(self::cell::Cell::new(row, col, bg_fill, *cell_size, image));
                    }
                    ui.end_row();
                }
            });
    }
}
