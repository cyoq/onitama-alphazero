use egui::*;
use egui_extras::{Size, StripBuilder};

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
                ui.label("text");
                ui.separator();
                Self::show_deck_panel(ui);
                ui.separator();
                Self::show_bottom_panel(ui);
            });
    }

    fn show_top_panel(&self, ui: &mut Ui) {}

    fn show_deck_panel(ui: &mut Ui) {
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
                        for i in 0..8 {
                            strip.cell(|ui| {
                                ui.vertical_centered(|ui| {
                                    ui.painter().rect_filled(
                                        ui.available_rect_before_wrap(),
                                        0.0,
                                        Color32::YELLOW,
                                    );
                                    ui.label(format!("Cell {}", i));
                                });
                            });
                        }
                    });
                });
                // Last row with 8 columns
                strip.strip(|builder| {
                    builder.sizes(Size::remainder(), 8).horizontal(|mut strip| {
                        for i in 0..8 {
                            strip.cell(|ui| {
                                ui.vertical_centered(|ui| {
                                    ui.painter().rect_filled(
                                        ui.available_rect_before_wrap(),
                                        0.0,
                                        Color32::BLUE,
                                    );
                                    ui.label(format!("Cell {}", i));
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
}
