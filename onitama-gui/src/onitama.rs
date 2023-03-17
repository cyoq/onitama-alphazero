use eframe::{App, CreationContext};
use egui::{
    Button, CentralPanel, Color32, Context, Direction, FontData, FontDefinitions, FontFamily,
    Hyperlink, Label, Layout, Pos2, Rect, RichText, SidePanel, Ui,
};
use egui_extras::{Size, StripBuilder};
use onitama_game::game::card::{Card, DRAGON, FROG, HORSE, ORIGINAL_CARDS};

use crate::move_card::MoveCard;

const UTILITY_PANEL_WIDTH: f32 = 370.;
const BOARD_PANEL_WIDTH: f32 = 930.;
const PADDING: f32 = 15.;
// const UTILITY_PANEL_HEIGHT: f32 = 500.;
// const HISTORY_PANEL_HEIGHT: f32 = 340.;

#[derive(Debug)]
pub struct Onitama {
    on: bool,
}

impl Default for Onitama {
    fn default() -> Self {
        Self { on: false }
    }
}

impl Onitama {
    pub fn new(cc: &CreationContext) -> Self {
        Onitama::configure_fonts(&cc.egui_ctx);

        Self { on: false }
    }

    fn configure_fonts(ctx: &Context) {
        let mut font_def = FontDefinitions::default();

        // load the font
        font_def.font_data.insert(
            "MesloLGS".to_string(),
            FontData::from_static(include_bytes!("../MesloLGS_NF_Regular.ttf")),
        );

        // set the font to be in the first priority
        font_def
            .families
            .get_mut(&FontFamily::Proportional)
            .expect("Proportional key must be in the 'families' property")
            .insert(0, "MesloLGS".to_string());

        // load the fonts to the context
        ctx.set_fonts(font_def);
    }

    fn board_panel(&self, ui: &mut Ui) {
        let faded_color = ui.visuals().window_fill();
        let faded_color = |color: Color32| -> Color32 {
            use egui::Rgba;
            egui::lerp(Rgba::from(color)..=Rgba::from(faded_color), 0.5).into()
        };

        ui.horizontal(|ui| {
            let background = Rect::from_points(&[
                Pos2::new(0., 0.),
                Pos2::new(750., 0.),
                Pos2::new(0., 750.),
                Pos2::new(750., 750.),
            ]);
            ui.painter().rect_filled(background, 0.0, Color32::BLACK);
            StripBuilder::new(ui)
                .size(Size::exact(150.0))
                .vertical(|mut strip| {
                    // Create a builder
                    strip.strip(|builder| {
                        // Create rows
                        builder.sizes(Size::exact(150.), 5).vertical(|mut strip| {
                            for _row in 0..5 {
                                // Create column builder
                                strip.strip(|builder| {
                                    // Create columns
                                    builder.sizes(Size::exact(150.), 5).horizontal(|mut strip| {
                                        for _col in 0..5 {
                                            strip.cell(|ui| {
                                                let rect = ui.available_rect_before_wrap();
                                                ui.painter().rect_filled(
                                                    rect,
                                                    0.0,
                                                    faded_color(Color32::RED),
                                                );
                                                // ui.add(Button::new(format!("{}, {}", _row, _col)));
                                                ui.label("width: 50%\nheight: remaining");
                                            });
                                        }
                                    });
                                });
                            }
                        });
                    });
                });
        });
    }

    fn deck_panel(ui: &mut Ui) {}

    fn utility_panel(&self, ui: &mut Ui) {
        ui.with_layout(
            egui::Layout::top_down(egui::Align::LEFT).with_cross_justify(true),
            |ui| {
                self.utility_widget(ui);
                ui.add(egui::Separator::default().grow(8.0));
                self.history_widget(ui);
            },
        );
    }

    fn utility_widget(&self, ui: &mut Ui) {
        // Add button on the left for starting a new game
        ui.with_layout(Layout::left_to_right(egui::Align::Min), |ui| {
            ui.add(Button::new(
                RichText::new("Start a new game").text_style(egui::TextStyle::Body),
            ));

            ui.add(Button::new(
                RichText::new("Make a new setup").text_style(egui::TextStyle::Body),
            ));
        });
        ui.add_space(PADDING);

        // Add button on the right to open a window with a new game setup
        // ui.with_layout(Layout::right_to_left(egui::Align::Max), |ui| {
        //     ui.add(Button::new(
        //         RichText::new("Make a new setup").text_style(egui::TextStyle::Body),
        //     ));
        // });

        // ui.add_space(PADDING);

        // // Add Statistics heading
        ui.vertical_centered(|ui| {
            ui.add(Label::new(
                RichText::new("Statistics").text_style(egui::TextStyle::Heading),
            ))
        });
        ui.add_space(PADDING);

        // Add labels for the statistics
        ui.add(Label::new("Evaluation Score: OVER9000"));
        ui.add_space(PADDING);

        // Add footer with links
        ui.separator();
        ui.vertical_centered(|ui| {
            ui.add_space(PADDING);
            // add a link to the egui framework
            ui.add(Hyperlink::from_label_and_url(
                RichText::new("Made with egui").text_style(egui::TextStyle::Monospace),
                "https://github.com/emilk/egui",
            ));
            // then we'll put github link the the headlines source code
            ui.add(Hyperlink::new("https://github.com/"));

            ui.add_space(10.);
        });
        ui.add_space(PADDING);
    }

    fn history_widget(&self, ui: &mut Ui) {
        ui.vertical_centered_justified(|ui| {
            ui.add(Label::new(
                RichText::new("History").text_style(egui::TextStyle::Heading),
            ));
        });
    }
}

impl App for Onitama {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        ctx.set_debug_on_hover(true);
        SidePanel::new(egui::panel::Side::Left, "board_panel")
            .max_width(BOARD_PANEL_WIDTH)
            .min_width(BOARD_PANEL_WIDTH)
            .resizable(false)
            .show(ctx, |ui| {
                ui.with_layout(Layout::centered_and_justified(Direction::TopDown), |ui| {
                    self.board_panel(ui);
                });
            });

        CentralPanel::default().show(ctx, |ui| {
            ui.add(MoveCard {
                mirror: &mut false,
                card: &ORIGINAL_CARDS[DRAGON.index],
                name: "Dragon",
                cell_size: 32.,
            });

            ui.add(MoveCard {
                mirror: &mut false,
                card: &ORIGINAL_CARDS[FROG.index],
                name: "Frog",
                cell_size: 32.,
            });

            ui.add(MoveCard {
                mirror: &mut true,
                card: &ORIGINAL_CARDS[HORSE.index],
                name: "Horse",
                cell_size: 32.,
            });
        });

        SidePanel::new(egui::panel::Side::Right, "right_panel")
            .max_width(UTILITY_PANEL_WIDTH)
            .min_width(UTILITY_PANEL_WIDTH)
            .resizable(false)
            .show(ctx, |ui| self.utility_panel(ui));
    }
}
