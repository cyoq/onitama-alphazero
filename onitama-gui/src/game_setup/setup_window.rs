use egui::*;
use egui_extras::{Size, StripBuilder};
use onitama_game::game::{
    card::{Card, CARD_NAMES, ORIGINAL_CARDS},
    deck::Deck,
};
use rand::{thread_rng, Rng};

use crate::{
    move_card::MoveCard,
    player::{Participant, Player},
};

use super::{
    participants::{create_participant_setup, ParticipantSetup},
    tournament::Tournament,
};

const MOVE_CARD_CELL_SIZE: f32 = 18.;
const SETUP_WINDOW_WIDTH: f32 = 900.;
const SETUP_WINDOW_HEIGHT: f32 = 550.;
const COLOR_CHOICE_ORDER: [CardColor; 5] = [
    CardColor::Red,
    CardColor::Red,
    CardColor::Blue,
    CardColor::Blue,
    CardColor::Yellow,
];

enum CardColor {
    Red,
    Blue,
    // For neutral card
    Yellow,
}

impl CardColor {
    pub fn color(&self) -> Color32 {
        match self {
            CardColor::Red => Color32::RED,
            CardColor::Blue => Color32::BLUE,
            CardColor::Yellow => Color32::GOLD,
        }
    }
}

pub struct SetupWindow<'a> {
    selected_cards: &'a mut [Option<Card>; 5],
    deck: &'a mut Deck,
    selected_participants: &'a mut [(Participant, Box<dyn ParticipantSetup>); 2],
    players: &'a mut [Player; 2],
    tournament: &'a mut Tournament,
}

impl<'a> SetupWindow<'a> {
    pub fn new(
        selected_cards: &'a mut [Option<Card>; 5],
        deck: &'a mut Deck,
        selected_participants: &'a mut [(Participant, Box<dyn ParticipantSetup>); 2],
        players: &'a mut [Player; 2],
        tournament: &'a mut Tournament,
    ) -> Self {
        Self {
            selected_cards,
            deck,
            selected_participants,
            players,
            tournament,
        }
    }

    pub fn show_setup(
        &mut self,
        ctx: &Context,
        is_active: &'a mut bool,
        should_start_new_game: &'a mut bool,
    ) {
        Window::new("Game Setup")
            .open(is_active)
            .resizable(false)
            .min_width(SETUP_WINDOW_WIDTH)
            .min_height(SETUP_WINDOW_HEIGHT)
            .show(ctx, |ui| {
                self.show_top_panel(ui);
                ui.separator();

                self.deck_helper(ui);
                ui.separator();

                self.show_deck_panel(ui);
                ui.separator();

                self.show_bottom_panel(ui, should_start_new_game);
            });
    }

    fn show_top_panel(&mut self, ui: &mut Ui) {
        // Create 1 row
        StripBuilder::new(ui)
            // height of the row
            .size(Size::exact(200.))
            .vertical(|mut strip| {
                strip.strip(|builder| {
                    // create two columns
                    builder
                        .size(Size::exact(200.))
                        .size(Size::remainder())
                        .horizontal(|mut strip| {
                            // First column is the content with combo-boxes
                            strip.cell(|ui| {
                                self.top_panel_combobox(ui);
                            });

                            // Second column is separated into two rows with settings for each combo box
                            strip.strip(|builder| {
                                builder.sizes(Size::remainder(), 2).vertical(|mut strip| {
                                    for idx in 0..self.selected_participants.len() {
                                        strip.cell(|ui| {
                                            ui.vertical_centered(|ui| {
                                                // update view if participant was updated too
                                                let (participant, setup) =
                                                    &mut self.selected_participants[idx];
                                                if setup.participant() != *participant {
                                                    *setup =
                                                        create_participant_setup(&*participant);
                                                }
                                                setup.show(ui);
                                                ui.separator();
                                            });
                                        });
                                    }
                                });
                            });
                        });
                });
            });
    }

    fn top_panel_combobox(&mut self, ui: &mut Ui) {
        ui.add_space(15.);
        ui.add(Label::new(
            RichText::new("🟥 Red player: ").color(Color32::RED),
        ));
        self.create_combo_box(ui, "red_player_combo_box", 0);

        ui.add_space(40.);

        ui.vertical_centered(|ui| {
            ui.label("VS");
        });
        ui.add_space(35.);

        ui.add(Label::new(
            RichText::new("🟦 Blue player: ").color(Color32::BLUE),
        ));
        self.create_combo_box(ui, "blue_player_combo_box", 1);
    }

    fn create_combo_box(&mut self, ui: &mut Ui, id: &str, idx: usize) {
        assert!(idx < 2);
        let (participant, _setup) = &mut self.selected_participants[idx];
        egui::ComboBox::from_id_source(id)
            .selected_text(participant.to_string())
            .show_ui(ui, |ui| {
                ui.selectable_value(participant, Participant::Human, "Human");
                ui.selectable_value(participant, Participant::Random, "Random");
                ui.selectable_value(participant, Participant::AlphaBeta, "AlphaBeta");
                ui.selectable_value(participant, Participant::Mcts, "MCTS");
            });
    }

    fn deck_helper(&mut self, ui: &mut Ui) {
        ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
            ui.label("Right click on a card to select player color for it. \nSelect up to 5 cards. \nIf less than 5 cards selected, the rest will be chosen randomly when starting the game.");
        });

        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
            let clear = ui.button("Clear selection");
            if clear.clicked() {
                self.clear_selected_cards();
            }
            clear.on_hover_text("Clear all selected cards");
            ui.add_space(15.);

            let random_btn = ui.button("Random cards!");
            if random_btn.clicked() {
                self.fill_random();
            }
            random_btn.on_hover_text("Take random cards in addition to already chosen cards");
            ui.add_space(15.);
        });
    }

    fn show_deck_panel(&mut self, ui: &mut Ui) {
        let deck = &ORIGINAL_CARDS;

        StripBuilder::new(ui)
            // Sizes for the card rows
            .size(Size::exact(130.))
            .size(Size::exact(130.))
            // Signal that strips will represent rows
            .vertical(|mut strip| {
                // strip builder that will separate row into 8 columns
                strip.strip(|builder| {
                    builder.sizes(Size::remainder(), 8).horizontal(|mut strip| {
                        for card in deck.iter().take(8) {
                            strip.cell(|ui| {
                                ui.vertical_centered(|ui| {
                                    let r = self.add_card_to_ui(ui, card);
                                    r.context_menu(|ui| self.nested_menus(ui, card));
                                });
                            });
                        }
                    });
                });
                // Last row with 8 columns
                strip.strip(|builder| {
                    builder.sizes(Size::remainder(), 8).horizontal(|mut strip| {
                        for card in deck.iter().skip(8) {
                            strip.cell(|ui| {
                                ui.vertical_centered(|ui| {
                                    let r = self.add_card_to_ui(ui, card);
                                    r.context_menu(|ui| self.nested_menus(ui, card));
                                });
                            });
                        }
                    });
                });
            });
    }

    fn show_bottom_panel(&mut self, ui: &mut Ui, should_start_new_game: &'a mut bool) {
        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
            let start_game_btn = ui.button("Start a game");
            if start_game_btn.clicked() {
                self.create_deck();
                self.assign_players();
                *should_start_new_game = true;
            }
            ui.add_space(10.);
            ui.checkbox(&mut true, "Save my choice");
        });

        ui.separator();

        self.show_tournament_setup(ui, should_start_new_game);
    }

    fn show_tournament_setup(&mut self, ui: &mut Ui, should_start_new_game: &'a mut bool) {
        let r = ui.label(RichText::new("Tournament setup").text_style(egui::TextStyle::Heading));
        r.on_hover_text("Setup a tournament between agents");

        ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
            ui.label("Round amount");
            ui.add(Slider::new(&mut self.tournament.round_amnt, 1..=1000));

            ui.add_space(20.);
        });

        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
            let start_tournament_btn = ui.button("Start a tournament");
            ui.add_space(20.);

            if start_tournament_btn.clicked() {
                self.tournament.is_tournament_on = true;
                self.create_deck();
                self.assign_players();
                self.tournament.deck = self.deck.clone();
                *should_start_new_game = true;
            }

            ui.checkbox(&mut self.tournament.save_games, "Save tournament games?");
            ui.add_space(20.);

            ui.checkbox(&mut self.tournament.do_player_swap, "Do players swap?");
            ui.add_space(20.);

            ui.checkbox(
                &mut self.tournament.random_deck_each_turn,
                "Use random deck each turn?",
            );
            ui.add_space(20.);
        });
    }

    fn fill_random(&mut self) {
        let mut rng = thread_rng();
        for card_idx in 0..self.selected_cards.len() {
            if let None = self.selected_cards[card_idx] {
                loop {
                    let idx = rng.gen_range(0..ORIGINAL_CARDS.len());
                    let card = ORIGINAL_CARDS[idx];
                    if !self.selected_cards.contains(&Some(card)) {
                        self.selected_cards[card_idx] = Some(card);
                        break;
                    }
                }
            }
        }
    }

    fn create_deck(&mut self) {
        self.fill_random();

        let cards = self
            .selected_cards
            .iter()
            .map(|card| card.expect("Must be a valid card"))
            .collect::<Vec<_>>();

        *self.deck = Deck::new(cards.try_into().expect("Must be 5 cards"));
    }

    fn assign_players(&mut self) {
        let player = |(p, s): &(Participant, Box<dyn ParticipantSetup>)| {
            let agent = s.create_agent();
            let typ = p.to_player_type();
            Player { typ, agent }
        };

        self.players[0] = player(&self.selected_participants[0]);
        self.players[1] = player(&self.selected_participants[1]);
    }

    fn clear_selected_cards(&mut self) {
        for card_idx in 0..self.selected_cards.len() {
            self.selected_cards[card_idx] = None;
        }
    }

    fn add_card_to_ui(&mut self, ui: &mut Ui, card: &Card) -> Response {
        let pos = self.selected_cards.iter().position(|&c| c == Some(*card));
        let stroke_fill = match pos {
            Some(idx) => COLOR_CHOICE_ORDER[idx].color(),
            None => Color32::BLACK,
        };

        let response = ui.add(MoveCard {
            mirror: &false,
            card,
            name: CARD_NAMES[card.index],
            cell_size: MOVE_CARD_CELL_SIZE,
            stroke_fill,
        });

        response
    }

    fn select_card(&mut self, card: &'a Card, color: CardColor) {
        let pos = self.selected_cards.iter().position(|&c| c == Some(*card));

        if let Some(idx) = pos {
            self.selected_cards[idx] = None;
        }

        match color {
            CardColor::Red => match (self.selected_cards[0], self.selected_cards[1]) {
                (Some(_), None) => self.selected_cards[1] = Some(*card),
                (Some(_), Some(_)) => {
                    self.selected_cards[0] = self.selected_cards[1];
                    self.selected_cards[1] = Some(*card);
                }
                (_, _) => self.selected_cards[0] = Some(*card),
            },
            CardColor::Blue => match (self.selected_cards[2], self.selected_cards[3]) {
                (Some(_), None) => self.selected_cards[3] = Some(*card),
                (Some(_), Some(_)) => {
                    self.selected_cards[2] = self.selected_cards[3];
                    self.selected_cards[3] = Some(*card);
                }
                (_, _) => self.selected_cards[2] = Some(*card),
            },
            CardColor::Yellow => self.selected_cards[4] = Some(*card),
        }
    }

    fn deselect_card(&mut self, card: &Card) {
        let pos = self.selected_cards.iter().position(|&c| c == Some(*card));
        match pos {
            Some(idx) => self.selected_cards[idx] = None,
            None => (),
        }
    }

    fn nested_menus(&mut self, ui: &mut Ui, card: &'a Card) {
        if ui.button("Red").clicked() {
            self.select_card(card, CardColor::Red);
            ui.close_menu();
        }

        if ui.button("Blue").clicked() {
            self.select_card(card, CardColor::Blue);
            ui.close_menu();
        }

        if ui.button("Neutral").clicked() {
            self.select_card(card, CardColor::Yellow);
            ui.close_menu();
        }

        let is_selected = self.selected_cards.contains(&Some(*card));
        if is_selected {
            if ui.button("Deselect").clicked() {
                self.deselect_card(card);
                ui.close_menu();
            }
        }
    }
}
