pub mod cell;

use eframe::epaint::ahash::HashMap;
use egui::*;
use onitama_game::{
    common::{from_2d_to_1d, get_bit},
    game::{player_color::PlayerColor, r#move::Move, state::State},
};

use crate::{image::Image, onitama::Figure};

pub const BG_FILL: Color32 = Color32::WHITE;
pub const BG_TEMPLE: Color32 = Color32::GRAY;
pub const BG_BLUE: Color32 = Color32::BLUE;
pub const BG_RED: Color32 = Color32::RED;

pub fn drag_source(ui: &mut Ui, id: Id, body: impl FnOnce(&mut Ui)) {
    let is_being_dragged = ui.memory(|mem| mem.is_being_dragged(id));

    if !is_being_dragged {
        let response = ui.scope(body).response;

        // Check for drags:
        let response = ui.interact(response.rect, id, Sense::drag());
        if response.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::Grab);
        }
    } else {
        ui.ctx().set_cursor_icon(CursorIcon::Grabbing);

        // Paint the body to a new layer:
        let layer_id = LayerId::new(Order::Tooltip, id);
        let response = ui.with_layer_id(layer_id, body).response;

        // Now we move the visuals of the body to where the mouse is.
        // Normally you need to decide a location for a widget first,
        // because otherwise that widget cannot interact with the mouse.
        // However, a dragged component cannot be interacted with anyway
        // (anything with `Order::Tooltip` always gets an empty [`Response`])
        // So this is fine!

        if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
            let delta = pointer_pos - response.rect.center();
            ui.ctx().translate_layer(layer_id, delta);
        }
    }
}

pub fn drop_target<R>(
    ui: &mut Ui,
    can_accept_what_is_being_dragged: bool,
    body: impl FnOnce(&mut Ui) -> R,
) -> InnerResponse<R> {
    let is_being_dragged = ui.memory(|mem| mem.is_anything_being_dragged());

    let margin = Vec2::splat(4.0);

    let outer_rect_bounds = ui.available_rect_before_wrap();
    let inner_rect = outer_rect_bounds.shrink2(margin);
    let where_to_put_background = ui.painter().add(Shape::Noop);
    let mut content_ui = ui.child_ui(inner_rect, *ui.layout());
    let ret = body(&mut content_ui);
    let outer_rect = Rect::from_min_max(outer_rect_bounds.min, content_ui.min_rect().max + margin);
    let (rect, response) = ui.allocate_at_least(outer_rect.size(), Sense::hover());

    let style = if is_being_dragged && can_accept_what_is_being_dragged && response.hovered() {
        ui.visuals().widgets.active
    } else {
        ui.visuals().widgets.inactive
    };

    let mut fill = style.bg_fill;
    let mut stroke = style.bg_stroke;
    if is_being_dragged && !can_accept_what_is_being_dragged {
        fill = ui.visuals().gray_out(fill);
        stroke.color = ui.visuals().gray_out(stroke.color);
    }

    ui.painter().set(
        where_to_put_background,
        epaint::RectShape {
            rounding: style.rounding,
            fill,
            stroke,
            rect,
        },
    );

    InnerResponse::new(ret, response)
}

/// A representation of a game board
pub struct GameBoard<'a> {
    /// State of the current game
    pub state: &'a mut State,
    /// A size of the cell
    pub cell_size: f32,
    /// images to display
    pub images: &'a HashMap<Figure, Image>,
}

impl<'a> GameBoard<'a> {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        let bg_fill = BG_FILL;

        let mut source_col_row: Option<(u32, u32)> = None;
        let mut drop_col_row: Option<(u32, u32)> = None;

        egui::Grid::new("game_board")
            .min_col_width(0.)
            .min_row_height(0.)
            .spacing(egui::vec2(0., 0.))
            .show(ui, |ui| {
                let can_accept_what_is_being_dragged = true;
                for row in 0..5 {
                    for col in 0..5 {
                        let mut image = None;
                        let coords = from_2d_to_1d((row, col)) as usize;
                        let red_pawn = get_bit(self.state.pawns[PlayerColor::Red as usize], coords);
                        let red_king = get_bit(self.state.kings[PlayerColor::Red as usize], coords);
                        let blue_pawn =
                            get_bit(self.state.pawns[PlayerColor::Blue as usize], coords);
                        let blue_king =
                            get_bit(self.state.kings[PlayerColor::Blue as usize], coords);

                        if red_pawn == 1 {
                            image = Some(&self.images.get(&Figure::RedPawn).unwrap().image);
                        } else if blue_pawn == 1 {
                            image = Some(&self.images.get(&Figure::BluePawn).unwrap().image);
                        } else if red_king == 1 {
                            image = Some(&self.images.get(&Figure::RedKing).unwrap().image);
                        } else if blue_king == 1 {
                            image = Some(&self.images.get(&Figure::BlueKing).unwrap().image);
                        }

                        let response = drop_target(ui, can_accept_what_is_being_dragged, |ui| {
                            let cell_id = Id::new("figure_dnd").with(col).with(row);
                            drag_source(ui, cell_id, |ui| {
                                let response = ui.add(self::cell::Cell::new(
                                    row,
                                    col,
                                    bg_fill,
                                    self.cell_size,
                                    image,
                                ));
                            });

                            if ui.memory(|mem| mem.is_being_dragged(cell_id)) {
                                source_col_row = Some((col, row));
                            }
                        })
                        .response;

                        let is_being_dragged = ui.memory(|mem| mem.is_anything_being_dragged());
                        if is_being_dragged
                            && can_accept_what_is_being_dragged
                            && response.hovered()
                        {
                            drop_col_row = Some((col, row));
                        }
                    }
                    ui.end_row();
                }
            });

        if let Some(source_col_row) = source_col_row {
            if let Some(drop_col_row) = drop_col_row {
                if ui.input(|i| i.pointer.any_released()) {
                    // do the drop:
                    self.state.make_move(
                        &Move {
                            from: from_2d_to_1d(source_col_row),
                            to: from_2d_to_1d(drop_col_row),
                            figure: onitama_game::game::figure::Figure::Pawn,
                        },
                        PlayerColor::Red,
                        0,
                    );
                }
            }
        }
    }
}
