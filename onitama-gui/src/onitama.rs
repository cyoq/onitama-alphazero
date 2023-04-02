use std::path::PathBuf;

use eframe::{epaint::ahash::HashMap, App, CreationContext};
use egui::{
    Button, CentralPanel, Color32, Context, FontData, FontDefinitions, FontFamily, Hyperlink,
    Label, Layout, RichText, SidePanel, Ui,
};
use egui_extras::{Size, StripBuilder};
use onitama_game::game::piece::{Piece, PieceKind};
use onitama_game::game::r#move::Move;
use onitama_game::game::{
    card::{Card, CARD_NAMES, DRAGON, FROG, HORSE, ORIGINAL_CARDS, RABBIT, TIGER},
    deck::Deck,
    done_move::DoneMove,
    game_state::GameState,
    move_result::MoveResult,
    player_color::PlayerColor,
};

use crate::game_setup::setup_window::SetupWindow;
use crate::player::{Participant, Player, PlayerType};
use crate::selected_card::SelectedCard;
use crate::{game_board::GameBoard, image::Image, move_card::MoveCard};

const UTILITY_PANEL_WIDTH: f32 = 370.;
const BOARD_PANEL_WIDTH: f32 = 930.;
const PADDING: f32 = 15.;
const MOVE_CARD_CELL_SIZE: f32 = 32.; // to make 160 pixel total

pub struct Onitama {
    debug: bool,
    images: HashMap<Piece, Image>,
    game_state: GameState,
    selected_card: SelectedCard,
    /// Selected piece can be identified by (row, col)
    selected_piece: Option<(u32, u32)>,
    /// Needed to color the cell with the last played move
    last_played_move: Option<(u32, u32)>,
    allowed_moves: [[bool; 5]; 5],
    human_done_move: Option<DoneMove>,
    move_result: Option<MoveResult>,
    end_game: bool,
    // TODO: Later on the Application must own the player and not the Game
    players: [PlayerType; 2],
    player_names: [&'static str; 2],
    board_panel_text: (String, Color32),
    card_panel_text: (String, Color32),
    deck: Deck,
    show_game_setup: bool,
    setup_selected_cards: [Option<Card>; 5],
    should_start_new_game: bool,
    selected_players: [Participant; 2],
}

impl Onitama {
    pub fn new(cc: &CreationContext, debug: bool, red_player: Player, blue_player: Player) -> Self {
        let deck = Deck::new([
            ORIGINAL_CARDS[DRAGON.index].clone(),
            ORIGINAL_CARDS[FROG.index].clone(),
            ORIGINAL_CARDS[TIGER.index].clone(),
            ORIGINAL_CARDS[RABBIT.index].clone(),
            ORIGINAL_CARDS[HORSE.index].clone(),
        ]);

        Onitama::configure_fonts(&cc.egui_ctx);
        let images = Onitama::load_images();

        Self {
            debug,
            players: [red_player.typ, blue_player.typ],
            player_names: [red_player.agent.name(), blue_player.agent.name()],
            game_state: GameState::with_deck(red_player.agent, blue_player.agent, deck.clone()),
            deck,
            images,
            selected_card: SelectedCard::default(),
            selected_piece: None,
            last_played_move: None,
            allowed_moves: [[false; 5]; 5],
            human_done_move: None,
            move_result: None,
            end_game: false,
            board_panel_text: ("".to_string(), Color32::BLACK),
            card_panel_text: ("".to_string(), Color32::BLACK),
            show_game_setup: true,
            setup_selected_cards: [None, None, None, None, None],
            should_start_new_game: false,
            selected_players: [Participant::Human, Participant::Mcts],
        }
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

    fn load_images() -> HashMap<Piece, Image> {
        // Path comes from `target` folder
        let images = [
            (
                Piece::new(PieceKind::King, PlayerColor::Blue),
                "blue_king",
                "onitama-gui/assets/images/blue_king.svg",
            ),
            (
                Piece::new(PieceKind::Pawn, PlayerColor::Blue),
                "blue_pawn",
                "onitama-gui/assets/images/blue_pawn.svg",
            ),
            (
                Piece::new(PieceKind::King, PlayerColor::Red),
                "red_king",
                "onitama-gui/assets/images/red_king.svg",
            ),
            (
                Piece::new(PieceKind::Pawn, PlayerColor::Red),
                "red_pawn",
                "onitama-gui/assets/images/red_pawn.svg",
            ),
        ];

        images
            .iter()
            .map(|i| (i.0, Image::load_image(i.1.to_owned(), &PathBuf::from(i.2))))
            .collect::<HashMap<Piece, Image>>()
    }

    pub fn game_loop(&mut self) {
        match self.players[self.game_state.curr_agent_idx] {
            PlayerType::Human => {
                if let Some(done_move) = self.human_done_move {
                    self.move_result = Some(self.game_state.progress(done_move));
                    self.human_done_move = None;
                }
            }
            PlayerType::Ai => {
                let mov = self.game_state.agent_generate_move();
                self.last_played_move = Some(Move::convert_to_2d(mov.mov.to));
                self.move_result = Some(self.game_state.progress(mov));
            }
        }
    }

    pub fn update_text(&mut self) {
        let color = match self.game_state.curr_player_color {
            PlayerColor::Red => Color32::RED,
            PlayerColor::Blue => Color32::BLUE,
        };
        self.board_panel_text = (
            format!(
                "{} makes a move",
                self.player_names[self.game_state.curr_agent_idx]
            ),
            color,
        );

        if let Some(idx) = self.selected_card.card_idx {
            let card = self.game_state.state.deck.get_card(idx);
            let card_name = CARD_NAMES[card.index];
            self.card_panel_text = (format!("{} card was chosen", card_name), color);
        } else {
            self.card_panel_text = (
                format!(
                    "{} chooses a card",
                    self.game_state.curr_player_color.to_string()
                ),
                color,
            );
        }
    }

    fn board_panel(&mut self, ui: &mut Ui) {
        ui.add_space(PADDING);
        ui.vertical_centered(|ui| {
            let (text, color) = &self.board_panel_text;
            ui.label(
                RichText::new(text)
                    .text_style(egui::TextStyle::Heading)
                    .color(*color)
                    .size(20.),
            );
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
                        game_state: &mut self.game_state,
                        cell_size: 150.,
                        selected_card: &mut self.selected_card,
                        selected_piece: &mut self.selected_piece,
                        last_played_move: &mut self.last_played_move,
                        images: &self.images,
                        allowed_moves: &mut self.allowed_moves,
                        human_done_move: &mut self.human_done_move,
                        end_game: &self.end_game,
                    }
                    .show(ui);
                });
                strip.empty();
            });
    }

    fn deck_panel(&mut self, ui: &mut Ui) {
        let deck = self.game_state.state.deck.clone();

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
                        let (text, color) = &self.card_panel_text;
                        ui.label(
                            RichText::new(text)
                                .text_style(egui::TextStyle::Heading)
                                .color(*color)
                                .size(20.),
                        );
                    });
                });
                // strip builder that will separate row into two columns
                strip.strip(|builder| {
                    builder.sizes(Size::remainder(), 2).horizontal(|mut strip| {
                        for i in 0..2 {
                            strip.cell(|ui| {
                                ui.vertical_centered(|ui| {
                                    self.move_card_to_ui(ui, blue_player_cards[i])
                                });
                            });
                        }
                    });
                });
                // Middle row with 1 column
                strip.cell(|ui| {
                    ui.vertical_centered(|ui| self.move_card_to_ui(ui, neutral_card));
                });
                // Last row with 2 columns
                strip.strip(|builder| {
                    builder.sizes(Size::remainder(), 2).horizontal(|mut strip| {
                        for i in 0..2 {
                            strip.cell(|ui| {
                                ui.vertical_centered(|ui| {
                                    self.move_card_to_ui(ui, red_player_cards[i]);
                                });
                            });
                        }
                    });
                });
            });
    }

    fn utility_panel(&mut self, ui: &mut Ui) {
        ui.with_layout(
            egui::Layout::top_down(egui::Align::LEFT).with_cross_justify(true),
            |ui| {
                self.utility_widget(ui);
                ui.add(egui::Separator::default().grow(8.0));
                self.history_widget(ui);
            },
        );
    }

    fn utility_widget(&mut self, ui: &mut Ui) {
        // Add button on the left for starting a new game
        ui.with_layout(Layout::left_to_right(egui::Align::Min), |ui| {
            let start_game = ui.add(Button::new(
                RichText::new("Start a new game").text_style(egui::TextStyle::Body),
            ));

            if start_game.clicked() {
                tracing::debug!(
                    "Clicked to clear the game. Is end game: {:?}",
                    self.end_game
                );
                self.clear_game();
            }

            let game_setup = ui.add(Button::new(
                RichText::new("Make a new setup").text_style(egui::TextStyle::Body),
            ));

            if game_setup.clicked() {
                self.show_game_setup = true;
            }
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

    fn clear_game(&mut self) {
        self.game_state.clear();
        self.selected_card = SelectedCard::default();
        self.selected_piece = None;
        self.allowed_moves = [[false; 5]; 5];
        self.human_done_move = None;
        self.last_played_move = None;
        self.move_result = None;
        self.end_game = false;
    }

    fn history_widget(&self, ui: &mut Ui) {
        ui.vertical_centered_justified(|ui| {
            ui.add(Label::new(
                RichText::new("History").text_style(egui::TextStyle::Heading),
            ));
        });
    }

    fn move_card_to_ui(&mut self, ui: &mut Ui, card: &Card) {
        let deck = &self.game_state.state.deck;
        let mut stroke_fill = Color32::BLACK;
        if deck.get_card_idx(&card) == self.selected_card.card_idx {
            stroke_fill = match self.game_state.curr_player_color {
                PlayerColor::Red => Color32::RED,
                PlayerColor::Blue => Color32::BLUE,
            }
        }

        let enabled =
            deck.get_card_owner(card) == Some(self.game_state.curr_player_color) && !self.end_game;

        let response = ui.add(MoveCard {
            mirror: &deck.is_mirrored(card).unwrap_or(false),
            card: card,
            name: CARD_NAMES[card.index],
            cell_size: MOVE_CARD_CELL_SIZE,
            stroke_fill,
        });

        if response.clicked() && enabled {
            self.selected_card.update(card, deck);
            tracing::debug!("Selected card index: {:?}", self.selected_card);
        }

        if response.hovered() && enabled {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
    }
}

impl App for Onitama {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        if self.debug {
            ctx.set_debug_on_hover(true);
            egui::Window::new("Configuration").show(ctx, |ui| {
                ctx.inspection_ui(ui);
            });
        }

        self.update_text();

        if self.move_result.is_none() || !self.move_result.unwrap().is_win() && !self.end_game {
            self.game_loop();
        }

        // TODO: this should not be in the update
        if let Some(result) = self.move_result {
            match result {
                MoveResult::Capture => (),
                MoveResult::RedWin => {
                    self.end_game = true;
                    self.board_panel_text = ("Red won!".to_string(), Color32::RED);
                    self.card_panel_text = ("".to_string(), Color32::BLACK);
                }
                MoveResult::BlueWin => {
                    self.end_game = true;
                    self.board_panel_text = ("Blue won!".to_string(), Color32::BLUE);
                    self.card_panel_text = ("".to_string(), Color32::BLACK);
                }
                MoveResult::InProgress => (),
            }
        }

        if self.should_start_new_game {
            // Close game setup window
            self.show_game_setup = false;
            self.game_state = GameState::with_deck(
                self.game_state.agents[0].clone(),
                self.game_state.agents[1].clone(),
                self.deck.clone(),
            );
            self.clear_game();
            // Do not make a new game
            self.should_start_new_game = false;
        }

        SetupWindow::new(
            &mut self.setup_selected_cards,
            &mut self.deck,
            &mut self.selected_players,
        )
        .show_setup(
            ctx,
            &mut self.show_game_setup,
            &mut self.should_start_new_game,
        );

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
