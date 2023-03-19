use egui::{Color32, Widget};
use onitama_game::{
    common::{from_2d_to_1d, get_bit},
    game::card::Card,
};

pub const BG_FILL: Color32 = Color32::WHITE;
pub const BG_CENTER: Color32 = Color32::GRAY;

/// A representation of a move card
pub struct MoveCard<'a> {
    /// Determine if moves should be painted as mirrored
    pub mirror: &'a bool,
    /// A representing card
    pub card: &'a Card,
    /// A name of the card to display
    pub name: &'a str,
    /// A size of the cell
    pub cell_size: f32,
}

impl<'a> MoveCard<'a> {
    pub fn paint_cell(
        x: i32,
        y: i32,
        bg_fill: Color32,
        cell_size: f32,
        painter: &egui::Painter,
        rect: &egui::Rect,
    ) {
        let stroke: egui::Stroke = (0.5, Color32::BLACK).into();
        // Get physical center of a rectangle
        let center = rect.center();
        // Subtract center cell coords(2, 2) from (x, y) to get the (x, y) offset relative to the center
        let offset: egui::Pos2 = ((x - 2) as f32, (y - 2) as f32).into();
        // Get the absolute coords based on the cell size
        // Adding half a cell to the y, because additional row was added for the text
        let coords =
            center - egui::Pos2::new(offset.x * cell_size, offset.y * cell_size + cell_size / 2.);
        let cell = egui::Rect::from_center_size(coords.to_pos2(), egui::vec2(cell_size, cell_size));
        painter.rect(cell, 0., bg_fill, stroke);
    }
}

impl<'a> Widget for MoveCard<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let MoveCard {
            mirror,
            card,
            name,
            cell_size,
        } = self;
        // Widget code can be broken up in four steps:
        //  1. Decide a size for the widget
        //  2. Allocate space for it
        //  3. Handle interactions with the widget (if any)
        //  4. Paint the widget

        // 1. Deciding widget size:
        // You can query the `ui` how much space is available,
        // but in this example we have a fixed size widget based on the size of a cell
        // Adding additional cell row for the text - card name
        let desired_size = egui::vec2(cell_size * 5., cell_size * 6.);

        // 2. Allocating space:
        // This is where we get a region of the screen assigned.
        // We also tell the Ui to sense clicks in the allocated region.
        let (mut response, painter) = ui.allocate_painter(desired_size, egui::Sense::click());

        // 3. Interact: Time to check for clicks!
        if response.clicked() {
            tracing::info!("Clicked on a move card {}", name);
            response.mark_changed(); // report back that the value changed
        }

        if response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        // Attach some meta-data to the response which can be used by screen readers:
        response.widget_info(|| egui::WidgetInfo::new(egui::WidgetType::Other));

        // 4. Paint!
        // Make sure we need to paint:
        if !ui.is_rect_visible(response.rect) {
            return response;
        }

        let mut bg_fill = BG_FILL;
        egui::Grid::new(format!("card_board_{}", name))
            .min_col_width(0.)
            .min_row_height(0.)
            .spacing(egui::vec2(0., 0.))
            .show(ui, |ui| {
                for row in 0..5 {
                    for col in 0..5 {
                        let coords = from_2d_to_1d((row, col));
                        let pos = get_bit(
                            if !*mirror {
                                card.positions
                            } else {
                                card.mirror
                            },
                            coords as usize,
                        );

                        if pos == 1 {
                            bg_fill = Color32::LIGHT_GREEN;
                        } else {
                            bg_fill = BG_FILL;
                        }

                        if (row, col) == (2, 2) {
                            bg_fill = BG_CENTER;
                        }

                        MoveCard::paint_cell(
                            col as i32,
                            4 - row as i32, // inverse row from bitmap
                            bg_fill,
                            cell_size,
                            &painter,
                            &response.rect,
                        );
                    }
                    ui.end_row();
                }
            });
        let text_coords =
            response.rect.center() + egui::Vec2::new(0., 2. * cell_size + cell_size / 2.);
        painter.text(
            text_coords,
            egui::Align2::CENTER_CENTER,
            name,
            egui::FontId {
                size: 14.,
                family: egui::FontFamily::Proportional,
            },
            Color32::BLACK,
        );

        // All done! Return the interaction response so the user can check what happened
        // (hovered, clicked, ...) and maybe show a tooltip:
        response
    }
}
