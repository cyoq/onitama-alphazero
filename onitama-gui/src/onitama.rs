use std::path::PathBuf;

use eframe::{
    epaint::ahash::{HashMap, HashMapExt},
    App, CreationContext,
};
use egui::{
    Button, CentralPanel, Context, FontData, FontDefinitions, FontFamily, Hyperlink, Label, Layout,
    RichText, SidePanel, Ui,
};
use egui_extras::{Size, StripBuilder};
use onitama_game::game::{
    card::{Card, CARD_NAMES, DRAGON, FROG, HORSE, ORIGINAL_CARDS, RABBIT, TIGER},
    deck::Deck,
    player_color::PlayerColor,
    state::State,
};

use crate::{game_board::GameBoard, image::Image, move_card::MoveCard};

const UTILITY_PANEL_WIDTH: f32 = 370.;
const BOARD_PANEL_WIDTH: f32 = 930.;
const PADDING: f32 = 15.;
const MOVE_CARD_CELL_SIZE: f32 = 32.; // to make 160 pixel total
                                      // const UTILITY_PANEL_HEIGHT: f32 = 500.;
                                      // const HISTORY_PANEL_HEIGHT: f32 = 340.;

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub enum Figure {
    BlueKing,
    BluePawn,
    RedKing,
    RedPawn,
}

pub struct Onitama {
    images: HashMap<Figure, Image>,
}

impl Default for Onitama {
    fn default() -> Self {
        Self {
            images: HashMap::new(),
        }
    }
}

impl Onitama {
    pub fn new(cc: &CreationContext) -> Self {
        Onitama::configure_fonts(&cc.egui_ctx);
        let images = Onitama::load_images();

        Self { images }
    }

    fn configure_fonts(ctx: &Context) {
        let mut font_def = FontDefinitions::default();

        // load the font
        font_def.font_data.insert(
            "MesloLGS".to_string(),
            FontData::from_static(include_bytes!("../assets/fonts/MesloLGS_NF_Regular.ttf")),
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

    fn load_images() -> HashMap<Figure, Image> {
        // Path comes from `target` folder
        let images = [
            (
                Figure::BlueKing,
                "blue_king",
                "onitama-gui/assets/images/blue_king.svg",
            ),
            (
                Figure::BluePawn,
                "blue_pawn",
                "onitama-gui/assets/images/blue_pawn.svg",
            ),
            (
                Figure::RedKing,
                "red_king",
                "onitama-gui/assets/images/red_king.svg",
            ),
            (
                Figure::RedPawn,
                "red_pawn",
                "onitama-gui/assets/images/red_pawn.svg",
            ),
        ];

        images
            .iter()
            .map(|i| (i.0, Image::load_image(i.1.to_owned(), &PathBuf::from(i.2))))
            .collect::<HashMap<Figure, Image>>()
    }

    fn board_panel(&self, ui: &mut Ui) {
        ui.add_space(PADDING);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("Game information").text_style(egui::TextStyle::Heading));
        });

        ui.add_space(5.);

        // StripBuilder is for centering the board
        StripBuilder::new(ui)
            .size(Size::exact(90.))
            .size(Size::remainder())
            .size(Size::exact(90.))
            .horizontal(|mut strip| {
                strip.empty();
                strip.cell(|ui| {
                    GameBoard {
                        state: &State::new(),
                        cell_size: 150.,
                        images: &self.images,
                    }
                    .show(ui);
                });
                strip.empty();
            });
    }

    fn deck_panel(&self, ui: &mut Ui) {
        let deck = Deck::new([
            ORIGINAL_CARDS[DRAGON.index].clone(),
            ORIGINAL_CARDS[FROG.index].clone(),
            ORIGINAL_CARDS[TIGER.index].clone(),
            ORIGINAL_CARDS[RABBIT.index].clone(),
            ORIGINAL_CARDS[HORSE.index].clone(),
        ]);

        let red_player_cards = deck.get_player_cards(PlayerColor::Red);
        let blue_player_cards = deck.get_player_cards(PlayerColor::Blue);
        let neutral_card = deck.neutral_card();

        StripBuilder::new(ui)
            // size for the top row of textual information
            .size(Size::exact(30.))
            // Sizes for the card rows
            .size(Size::relative(1. / 3.))
            .size(Size::relative(1. / 3.))
            .size(Size::relative(1. / 3.))
            // Signal that strips will represent rows
            .vertical(|mut strip| {
                // Textual information strip
                strip.cell(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new("Text information").text_style(egui::TextStyle::Heading),
                        );
                    });
                });
                // strip builder that will separate row into two columns
                strip.strip(|builder| {
                    builder.sizes(Size::remainder(), 2).horizontal(|mut strip| {
                        for i in 0..2 {
                            strip.cell(|ui| {
                                ui.vertical_centered(|ui| {
                                    move_card_to_ui(ui, blue_player_cards[i], &deck)
                                });
                            });
                        }
                    });
                });
                // Middle row with 1 column
                strip.cell(|ui| {
                    ui.vertical_centered(|ui| move_card_to_ui(ui, neutral_card, &deck));
                });
                // Last row with 2 columns
                strip.strip(|builder| {
                    builder.sizes(Size::remainder(), 2).horizontal(|mut strip| {
                        for i in 0..2 {
                            strip.cell(|ui| {
                                ui.vertical_centered(|ui| {
                                    move_card_to_ui(ui, red_player_cards[i], &deck);
                                });
                            });
                        }
                    });
                });
            });
    }

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

fn move_card_to_ui(ui: &mut Ui, card: &Card, deck: &Deck) {
    ui.add(MoveCard {
        mirror: &deck.is_mirrored(card).unwrap_or(false),
        card: card,
        name: CARD_NAMES[card.index],
        cell_size: MOVE_CARD_CELL_SIZE,
    });
}

impl App for Onitama {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        ctx.set_debug_on_hover(true);
        SidePanel::new(egui::panel::Side::Left, "board_panel")
            .max_width(BOARD_PANEL_WIDTH)
            .min_width(BOARD_PANEL_WIDTH)
            .resizable(false)
            .show(ctx, |ui| self.board_panel(ui));

        SidePanel::new(egui::panel::Side::Right, "right_panel")
            .max_width(UTILITY_PANEL_WIDTH)
            .min_width(UTILITY_PANEL_WIDTH)
            .resizable(false)
            .show(ctx, |ui| self.utility_panel(ui));

        CentralPanel::default().show(ctx, |ui| self.deck_panel(ui));
    }
}
