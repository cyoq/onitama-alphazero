pub mod cell;
pub mod piece;

use eframe::epaint::ahash::HashMap;
use egui::*;
use onitama_game::{
    common::{from_2d_to_1d, get_bit},
    game::{done_move::DoneMove, game::Game, player_color::PlayerColor, r#move::Move},
};

use crate::{image::Image, onitama::Figure, selected_card::SelectedCard};

use self::piece::Piece;

pub const BG_FILL: Color32 = Color32::WHITE;
pub const BG_TEMPLE: Color32 = Color32::LIGHT_GRAY;
pub const BG_BLUE: Color32 = Color32::BLUE;
pub const BG_RED: Color32 = Color32::RED;
pub const BG_LAST_PLAYED_MOVE: Color32 = Color32::LIGHT_YELLOW;
pub const RED_TEMPLE: (u32, u32) = (0, 2);
pub const BLUE_TEMPLE: (u32, u32) = (4, 2);

pub fn drag_source(ui: &mut Ui, id: Id, body: impl FnOnce(&mut Ui)) -> Response {
    let is_being_dragged = ui.memory(|mem| mem.is_being_dragged(id));

    if !is_being_dragged {
        let response = ui.scope(body).response;

        // Check for drags:
        let response = ui.interact(response.rect, id, Sense::click_and_drag());
        if response.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::Grab);
        }
        response
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
        response
    }
}

pub fn drop_target<R>(
    ui: &mut Ui,
    rect: Rect,
    body: impl FnOnce(&mut Ui) -> R,
) -> InnerResponse<R> {
    let margin = Vec2::splat(16.0);

    // not needed since we have a fixed size of a rect
    // let outer_rect_bounds = ui.available_rect_before_wrap();
    let outer_rect_bounds = rect;
    let inner_rect = outer_rect_bounds.shrink2(margin);
    // let where_to_put_background = ui.painter().add(Shape::Noop);
    let mut content_ui = ui.child_ui(inner_rect, *ui.layout());
    let ret = body(&mut content_ui);
    // Changed from min_rect to max rect to get the full content coverage
    let outer_rect = Rect::from_min_max(outer_rect_bounds.min, content_ui.max_rect().max + margin);
    let response = ui.allocate_response(outer_rect.size(), Sense::hover());

    InnerResponse::new(ret, response)
}

/// A representation of a game board
pub struct GameBoard<'a> {
    /// State of the current game
    pub game_state: &'a mut Game,
    /// A size of the cell
    pub cell_size: f32,
    /// images to display
    pub images: &'a HashMap<Figure, Image>,
    /// Selected card idx
    pub selected_card: &'a mut SelectedCard,
    /// Selected piece identified by (row, col)
    pub selected_piece: &'a mut Option<(u32, u32)>,
    /// Hashmap for the cells that are possible moves
    pub allowed_moves: &'a mut [[bool; 5]; 5],
    pub human_done_move: &'a mut Option<DoneMove>,
    pub last_played_move: &'a mut Option<(u32, u32)>,
    pub end_game: &'a bool,
}

impl<'a> GameBoard<'a> {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        let mut source_row_col: Option<(u32, u32)> = None;
        let mut drop_row_col: Option<(u32, u32)> = None;
        let state = &self.game_state.state;

        egui::Grid::new("game_board")
            .min_col_width(0.)
            .min_row_height(0.)
            .spacing(egui::vec2(0., 0.))
            .show(ui, |ui| {
                for row in 0..5 {
                    for col in 0..5 {
                        let mut image = None;
                        // TODO: Need to use color and a piece type
                        let mut piece_color = None;
                        let coords = from_2d_to_1d((row, col)) as usize;
                        let red_pawn = get_bit(state.pawns[PlayerColor::Red as usize], coords);
                        let red_king = get_bit(state.kings[PlayerColor::Red as usize], coords);
                        let blue_pawn = get_bit(state.pawns[PlayerColor::Blue as usize], coords);
                        let blue_king = get_bit(state.kings[PlayerColor::Blue as usize], coords);

                        if red_pawn == 1 {
                            image = Some(self.images.get(&Figure::RedPawn).unwrap().image());
                            piece_color = Some(PlayerColor::Red);
                        } else if blue_pawn == 1 {
                            image = Some(self.images.get(&Figure::BluePawn).unwrap().image());
                            piece_color = Some(PlayerColor::Blue);
                        } else if red_king == 1 {
                            image = Some(self.images.get(&Figure::RedKing).unwrap().image());
                            piece_color = Some(PlayerColor::Red);
                        } else if blue_king == 1 {
                            image = Some(self.images.get(&Figure::BlueKing).unwrap().image());
                            piece_color = Some(PlayerColor::Blue);
                        }

                        let can_accept_what_is_being_dragged =
                            self.allowed_moves[row as usize][col as usize];

                        let bg_fill = if can_accept_what_is_being_dragged {
                            match self.game_state.curr_player_color {
                                PlayerColor::Red => Color32::LIGHT_RED,
                                PlayerColor::Blue => Color32::LIGHT_BLUE,
                            }
                        } else if Some((row, col)) == *self.last_played_move {
                            BG_LAST_PLAYED_MOVE
                        } else if (row, col) == BLUE_TEMPLE || (row, col) == RED_TEMPLE {
                            BG_TEMPLE
                        } else {
                            BG_FILL
                        };

                        // Clear allowed moves if selected card changes
                        if self.selected_card.changed {
                            *self.allowed_moves = [[false; 5]; 5];
                            self.selected_card.changed = false;
                        }

                        self::cell::Cell::new(row, col, bg_fill, self.cell_size).show(
                            ui,
                            |ui, rect| {
                                let response = drop_target(ui, rect, |ui| {
                                    let cell_id = Id::new("figure_dnd").with(col).with(row);

                                    if self.selected_card.card_idx.is_none()
                                        || Some(self.game_state.curr_player_color) != piece_color
                                        || *self.end_game
                                    {
                                        if image.is_some() {
                                            ui.add(Piece {
                                                outer_rect: &rect,
                                                image: image.unwrap(),
                                            });
                                        }
                                    } else {
                                        let drag_resp = drag_source(ui, cell_id, |ui| {
                                            if image.is_some() {
                                                ui.add(Piece {
                                                    outer_rect: &rect,
                                                    image: image.unwrap(),
                                                });
                                            }
                                        });

                                        if drag_resp.drag_started() {
                                            if let Some(idx) = self.selected_card.card_idx {
                                                // Clear set possible moves if other piece is selected:
                                                if *self.selected_piece != Some((row, col)) {
                                                    *self.allowed_moves = [[false; 5]; 5];
                                                }

                                                tracing::debug!("Clicked to show available moves");

                                                *self.selected_piece = Some((row, col));

                                                let available_moves = state
                                                    .generate_legal_moves_card_idx(
                                                        self.game_state.curr_player_color,
                                                        idx,
                                                        (row, col),
                                                    );

                                                tracing::debug!(
                                                    "Available moves from state: {:?}",
                                                    available_moves
                                                );

                                                for mov in available_moves.iter() {
                                                    let (row, col) = Move::convert_to_2d(mov.to);

                                                    tracing::info!("({}, {})", row, col);

                                                    self.allowed_moves[row as usize]
                                                        [col as usize] = true;
                                                }
                                            }
                                            tracing::debug!(
                                                "Allowed moves 2D array: {:?}",
                                                self.allowed_moves
                                            );
                                        }
                                    }

                                    if ui.memory(|mem| mem.is_being_dragged(cell_id)) {
                                        source_row_col = Some((row, col));
                                    }
                                })
                                .response;

                                let is_being_dragged =
                                    ui.memory(|mem| mem.is_anything_being_dragged());

                                if is_being_dragged
                                    && can_accept_what_is_being_dragged
                                    && response.hovered()
                                {
                                    drop_row_col = Some((row, col));
                                }
                            },
                        );
                    }
                    ui.end_row();
                }
            });

        // Do the dropping here
        if let Some(source_row_col) = source_row_col {
            if let Some(drop_row_col) = drop_row_col {
                if source_row_col == drop_row_col {
                    return;
                }

                if ui.input(|i| i.pointer.any_released()) {
                    *self.allowed_moves = [[false; 5]; 5];
                    *self.selected_piece = None;
                    *self.last_played_move = Some(drop_row_col);

                    tracing::info!("Dropping from {:?} to {:?}", source_row_col, drop_row_col);

                    let used_card_idx = match self.game_state.curr_player_color {
                        PlayerColor::Red => self
                            .selected_card
                            .card_idx
                            .expect("Card must be selected at this moment!"),
                        // Subtracting 2, because blue player cards are at 2 and 3 index,
                        // but card rotation happens with 0 and 1 indexes
                        PlayerColor::Blue => {
                            self.selected_card
                                .card_idx
                                .expect("Card must be selected at this moment!")
                                - 2
                        }
                    };

                    let figure = self
                        .game_state
                        .state
                        .get_piece_type_at_pos(source_row_col)
                        .expect("At source position a figure must be present!");

                    *self.human_done_move = Some(DoneMove {
                        mov: Move {
                            from: from_2d_to_1d(source_row_col),
                            to: from_2d_to_1d(drop_row_col),
                            figure,
                        },
                        used_card_idx,
                    });

                    self.selected_card.set(None);
                }
            }
        }
    }
}
