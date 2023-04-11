use std::path::PathBuf;
use std::sync::mpsc::{sync_channel, Receiver};
use std::thread;
use std::time::Duration;

use eframe::{epaint::ahash::HashMap, App, CreationContext};
use egui::{
    Align, Button, CentralPanel, Color32, Context, Direction, FontData, FontDefinitions,
    FontFamily, Grid, Hyperlink, Label, Layout, RichText, ScrollArea, SidePanel, Ui,
};
use egui_extras::{Size, StripBuilder};
use egui_toast::{Toast, ToastOptions, Toasts};
use onitama_game::ai::human_gui::HumanGui;
use onitama_game::ai::mcts::Mcts;
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

use crate::game_setup::participants::{create_participant_setup, ParticipantSetup};
use crate::game_setup::setup_window::SetupWindow;
use crate::game_setup::tournament::TournamentSetup;
use crate::move_history::{MoveHistory, MoveInformation};
use crate::player::Participant;
use crate::player::{Player, PlayerType};
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
    board_panel_text: (String, Color32),
    card_panel_text: (String, Color32),
    deck: Deck,
    show_game_setup: bool,
    setup_selected_cards: [Option<Card>; 5],
    should_start_new_game: bool,
    selected_participants: [(Participant, Box<dyn ParticipantSetup>); 2],
    players: [Player; 2],
    mov_rx: Option<Receiver<(DoneMove, f64)>>,
    do_ai_move_generation: bool,
    evaluation_score: f64,
    move_history: MoveHistory,
    tournament_setup: TournamentSetup,
}

impl Onitama {
    pub fn new(cc: &CreationContext, debug: bool) -> Self {
        let deck = Deck::new([
            ORIGINAL_CARDS[DRAGON.index].clone(),
            ORIGINAL_CARDS[FROG.index].clone(),
            ORIGINAL_CARDS[TIGER.index].clone(),
            ORIGINAL_CARDS[RABBIT.index].clone(),
            ORIGINAL_CARDS[HORSE.index].clone(),
        ]);

        Self::configure_fonts(&cc.egui_ctx);

        let red_player = Player {
            typ: PlayerType::Human,
            agent: Box::new(HumanGui),
        };
        let blue_player = Player {
            typ: PlayerType::Human,
            agent: Box::new(Mcts::default()),
        };
        let players = [red_player, blue_player];
        let selected_participants = [
            (
                Participant::Human,
                create_participant_setup(&Participant::Human),
            ),
            (
                Participant::Mcts,
                create_participant_setup(&Participant::Mcts),
            ),
        ];

        Self {
            debug,
            game_state: GameState::with_deck(
                players[0].agent.clone(),
                players[1].agent.clone(),
                deck.clone(),
            ),
            players,
            deck,
            images: Self::load_images(),
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
            selected_participants,
            mov_rx: None,
            do_ai_move_generation: true,
            evaluation_score: 0.,
            move_history: MoveHistory::new(Participant::Human, Participant::Mcts),
            tournament_setup: TournamentSetup::default(),
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

    pub fn game_loop(&mut self, ctx: &Context) {
        match self.players[self.game_state.curr_agent_idx].typ {
            PlayerType::Human => {
                if let Some(done_move) = self.human_done_move {
                    // Need to save card before deck rotation for the history
                    let card = self
                        .game_state
                        .state
                        .deck
                        .get_card(done_move.used_card_idx)
                        .clone();

                    self.move_result = Some(self.game_state.progress(done_move));

                    self.move_history.push(MoveInformation {
                        state: self.game_state.state.clone(),
                        done_move: done_move.mov,
                        card,
                        evaluation: 0.,
                        // Get a previous player color
                        player_color: self.game_state.curr_player_color.enemy(),
                        ply: self.move_history.len() + 1,
                        move_result: self.move_result.expect("MoveResult cannot be empty here"),
                    });

                    self.human_done_move = None;
                }
            }
            PlayerType::Ai => {
                if self.do_ai_move_generation {
                    // Disable start move generation until a receiver says that it received a move
                    self.do_ai_move_generation = false;
                    let (mov_tx, mov_rx) = sync_channel(1);
                    self.mov_rx = Some(mov_rx);

                    let game_state = self.game_state.clone();

                    // TODO: need to close the thread when calculation happens
                    // but setup changes
                    thread::spawn(move || {
                        let mov = game_state.agent_generate_move();
                        if let Err(e) = mov_tx.send(mov) {
                            tracing::error!("Error sending a move: {}", e);
                        }
                    });
                }

                if let Some(rx) = &self.mov_rx {
                    if let Ok(mov) = rx.try_recv() {
                        let (mov, score) = mov;

                        self.evaluation_score = score;
                        self.last_played_move = Some(Move::convert_to_2d(mov.mov.to));

                        // Need to save card before deck rotation for the history
                        let card = self
                            .game_state
                            .state
                            .deck
                            .get_card(mov.used_card_idx)
                            .clone();

                        self.move_result = Some(self.game_state.progress(mov));

                        self.move_history.push(MoveInformation {
                            state: self.game_state.state.clone(),
                            done_move: mov.mov,
                            card,
                            evaluation: score,
                            // Get a previous player color
                            player_color: self.game_state.curr_player_color.enemy(),
                            ply: self.move_history.len() + 1,
                            move_result: self.move_result.expect("MoveResult cannot be empty here"),
                        });

                        self.mov_rx = None;

                        self.do_ai_move_generation = true;
                    }

                    ctx.request_repaint();
                }
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
                self.players[self.game_state.curr_agent_idx].agent.name()
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

    fn utility_panel(&mut self, ui: &mut Ui, ctx: &Context) {
        ui.with_layout(
            egui::Layout::top_down(egui::Align::LEFT).with_cross_justify(true),
            |ui| {
                self.utility_widget(ui);
                // ui.add(egui::Separator::default().grow(8.0));
                self.footer(ui);
                self.move_history_widget(ui, ctx);
            },
        );
    }

    fn utility_widget(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add(Label::new(
                RichText::new("Controls").text_style(egui::TextStyle::Heading),
            ));

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

            ui.add_space(PADDING);

            let game_setup = ui.add(Button::new(
                RichText::new("Make a new setup").text_style(egui::TextStyle::Body),
            ));

            if game_setup.clicked() {
                self.show_game_setup = true;
            }

            ui.add_space(PADDING);

            let start_tournament = ui.add(Button::new(
                RichText::new("About the game").text_style(egui::TextStyle::Body),
            ));

            if start_tournament.clicked() {
                tracing::warn!("TODO");
            }
        });

        ui.add_space(PADDING);
        ui.separator();

        ui.vertical_centered(|ui| {
            ui.add(Label::new(
                RichText::new("Evaluation").text_style(egui::TextStyle::Heading),
            ))
        });
        ui.add_space(PADDING);

        // Add labels for the evaluation
        let eval_color = if self.evaluation_score < 0. {
            Color32::BLUE
        } else if self.evaluation_score == 0. {
            Color32::BLACK
        } else {
            Color32::RED
        };
        ui.label(
            RichText::new(format!("Evaluation Score: {}", self.evaluation_score)).color(eval_color),
        );

        ui.add_space(PADDING);
    }

    fn move_history_widget(&self, ui: &mut Ui, ctx: &Context) {
        ui.vertical_centered_justified(|ui| {
            ui.add_space(PADDING);

            ui.add(Label::new(
                RichText::new("Move history").text_style(egui::TextStyle::Heading),
            ));

            ui.add_space(PADDING);

            // control buttons
            ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                let save_game = ui.add(Button::new(
                    RichText::new("Save the game").text_style(egui::TextStyle::Body),
                ));

                let mut toasts = Toasts::new()
                    .anchor((1800., 800.))
                    .direction(Direction::RightToLeft)
                    .align_to_end(true);

                if save_game.clicked() {
                    match self.move_history.save() {
                        Ok(_) => {
                            toasts.add(Toast {
                                kind: egui_toast::ToastKind::Success,
                                text: "The game was successfully saved".into(),
                                options: ToastOptions::with_duration(Duration::from_secs(5)),
                            });
                        }
                        Err(e) => tracing::error!("Error occurred while saving the game: {}", e),
                    };
                }

                toasts.show(ctx);

                let load_game = ui.add(Button::new(
                    RichText::new("Load the game").text_style(egui::TextStyle::Body),
                ));

                if load_game.clicked() {
                    tracing::warn!("TODO");
                }
            });

            ui.separator();

            // Vertical scroll enabled
            let scroll = ScrollArea::vertical().auto_shrink([false, true]);
            scroll.show(ui, |ui| {
                Grid::new("move_history_grid")
                    .num_columns(1)
                    .spacing([10., 10.])
                    .striped(true)
                    .show(ui, |ui| {
                        for mov_info in self.move_history.iter() {
                            let card_name = CARD_NAMES[mov_info.card.index];
                            let from = Move::convert_idx_to_notation(mov_info.done_move.from);
                            let to = Move::convert_idx_to_notation(mov_info.done_move.to);

                            let mut title = format!(
                                "▶ {}. {} {}-{} {}",
                                mov_info.ply,
                                card_name,
                                from,
                                to,
                                if mov_info.move_result == MoveResult::Capture {
                                    "❌"
                                } else {
                                    ""
                                }
                            );

                            if mov_info.move_result.is_win() {
                                title += &format!(" ({} won!)", mov_info.player_color.to_string());
                            }

                            let color = match mov_info.player_color {
                                PlayerColor::Red => Color32::RED,
                                PlayerColor::Blue => Color32::BLUE,
                            };
                            ui.colored_label(color, title);

                            ui.add_space(PADDING);
                            ui.end_row();
                        }
                    });
            });

            ui.add_space(PADDING);
        });
    }

    fn footer(&self, ui: &mut Ui) {
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
            ui.add(Hyperlink::new("https://github.com/cyoq/onitama-alphazero"));

            ui.add_space(10.);
        });
        ui.add_space(PADDING);
        ui.separator();
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

        let is_card_owner = deck.get_card_owner(card) == Some(self.game_state.curr_player_color);
        let is_human_player = self.players[self.game_state.curr_agent_idx].typ == PlayerType::Human;
        let enabled = is_card_owner && is_human_player && !self.end_game;

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

    fn clear_game(&mut self) {
        self.game_state.clear();
        self.selected_card = SelectedCard::default();
        self.selected_piece = None;
        self.allowed_moves = [[false; 5]; 5];
        self.human_done_move = None;
        self.last_played_move = None;
        self.move_result = None;
        self.end_game = false;
        self.evaluation_score = 0.;
        self.mov_rx = None;
        self.move_history.clear();
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

        if self.should_start_new_game {
            // Close game setup window
            self.show_game_setup = false;
            self.game_state = GameState::with_deck(
                self.players[0].agent.clone(),
                self.players[1].agent.clone(),
                self.deck.clone(),
            );
            self.clear_game();
            self.move_history.update_players(
                self.selected_participants[0].0.clone(),
                self.selected_participants[1].0.clone(),
            );
            // Do not make a new game
            self.should_start_new_game = false;
        }

        self.update_text();

        if self.move_result.is_none() || !self.move_result.unwrap().is_win() && !self.end_game {
            self.game_loop(ctx);
        }

        // TODO: this should not be in the update
        if let Some(result) = self.move_result {
            match result {
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
                MoveResult::Capture => (),
                MoveResult::InProgress => (),
            }
        }

        SetupWindow::new(
            &mut self.setup_selected_cards,
            &mut self.deck,
            &mut self.selected_participants,
            &mut self.players,
            &mut self.tournament_setup,
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
            .show(ctx, |ui| self.utility_panel(ui, ctx));

        CentralPanel::default().show(ctx, |ui| self.deck_panel(ui));
    }
}
