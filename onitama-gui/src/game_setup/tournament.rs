use egui::{Align, Layout, RichText, Slider, Ui};

pub struct TournamentSetup {
    pub round_amnt: u32,
    // First and a second player winrate
    // Their sum must be the same as round amount
    pub win_1: u32,
    pub win_2: u32,
    // A flag to save games
    pub save_games: bool,
}

impl Default for TournamentSetup {
    fn default() -> Self {
        Self {
            round_amnt: 10,
            win_1: 0,
            win_2: 0,
            save_games: false,
        }
    }
}

impl TournamentSetup {
    pub fn show(&mut self, ui: &mut Ui) {
        // ui.add_space(20.);
        let r = ui.label(RichText::new("Tournament setup").text_style(egui::TextStyle::Heading));
        r.on_hover_text("Setup a tournament between agents");

        // ui.add_space(20.);

        ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
            ui.label("Round amount");
            ui.add(Slider::new(&mut self.round_amnt, 1..=1000));

            ui.add_space(20.);
        });

        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
            let start_tournament_btn = ui.button("Start a tournament");

            if start_tournament_btn.clicked() {
                tracing::warn!("TODO");
            }

            ui.checkbox(&mut self.save_games, "Save tournament games");
            ui.add_space(20.);
        });
    }
}
