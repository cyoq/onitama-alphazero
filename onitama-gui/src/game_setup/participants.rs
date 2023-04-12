use std::time::Duration;

use egui::{Align, Layout, RichText, Slider, Ui};
use onitama_game::ai::{
    agent::Agent, alpha_beta::AlphaBeta, human_gui::HumanGui, mcts::Mcts, random::Random,
};

use crate::player::Participant;

pub trait ParticipantSetup: Send {
    fn show(&mut self, ui: &mut Ui);
    fn participant(&self) -> Participant;
    fn create_agent(&self) -> Box<dyn Agent>;
}

pub fn create_participant_setup(participant: &Participant) -> Box<dyn ParticipantSetup> {
    match *participant {
        Participant::Human => Box::new(HumanSetup::default()),
        Participant::Random => Box::new(RandomSetup::default()),
        Participant::AlphaBeta => Box::new(AlphaBetaSetup::default()),
        Participant::Mcts => Box::new(MctsSetup::default()),
    }
}

#[derive(Default)]
pub struct HumanSetup;

impl ParticipantSetup for HumanSetup {
    fn show(&mut self, ui: &mut Ui) {
        ui.add_space(20.);
        ui.label(RichText::new("Human parameters").text_style(egui::TextStyle::Heading));

        ui.label("Human does not have any parameters!");
    }

    fn create_agent(&self) -> Box<dyn Agent> {
        Box::new(HumanGui)
    }

    fn participant(&self) -> Participant {
        Participant::Human
    }
}

#[derive(Default)]
pub struct RandomSetup;

impl ParticipantSetup for RandomSetup {
    fn show(&mut self, ui: &mut Ui) {
        ui.add_space(20.);
        ui.label(RichText::new("Random parameters").text_style(egui::TextStyle::Heading));

        ui.label("Random does not have any parameters!");
    }

    fn create_agent(&self) -> Box<dyn Agent> {
        Box::new(Random)
    }

    fn participant(&self) -> Participant {
        Participant::Random
    }
}

pub struct AlphaBetaSetup {
    pub max_depth: u8,
    pub search_time: u64,
}

impl Default for AlphaBetaSetup {
    fn default() -> Self {
        Self {
            max_depth: 6,
            search_time: 1000,
        }
    }
}

impl ParticipantSetup for AlphaBetaSetup {
    fn show(&mut self, ui: &mut Ui) {
        ui.add_space(20.);
        ui.label(RichText::new("AlphaBeta parameters").text_style(egui::TextStyle::Heading));
        ui.with_layout(Layout::left_to_right(Align::Max), |ui| {
            ui.label("Max search depth: ");
            ui.add(Slider::new(&mut self.max_depth, 1..=15));

            ui.add_space(20.);

            ui.label("Search time(ms): ");
            ui.add(Slider::new(&mut self.search_time, 100..=15000));
        });
    }

    fn create_agent(&self) -> Box<dyn Agent> {
        Box::new(AlphaBeta {
            max_depth: self.max_depth,
        })
    }

    fn participant(&self) -> Participant {
        Participant::AlphaBeta
    }
}

pub struct MctsSetup {
    pub search_time: u64,
    pub min_node_visits: u32,
    pub exploration_c: f32,
    pub max_playouts: u32,
}

impl Default for MctsSetup {
    fn default() -> Self {
        Self {
            search_time: 1000,
            min_node_visits: 5,
            exploration_c: 2f32.sqrt(),
            max_playouts: 5000,
        }
    }
}

impl ParticipantSetup for MctsSetup {
    fn show(&mut self, ui: &mut Ui) {
        ui.add_space(20.);
        ui.label(RichText::new("MCTS parameters").text_style(egui::TextStyle::Heading));
        ui.with_layout(Layout::left_to_right(Align::Max), |ui| {
            ui.label("Exploration constant: ");
            ui.add(Slider::new(&mut self.exploration_c, 0.0..=10.0));

            ui.add_space(20.);

            ui.label("Minimal node visits: ");
            ui.add(Slider::new(&mut self.min_node_visits, 1..=10));

            ui.add_space(20.);

            ui.label("Search time(ms): ");
            ui.add(Slider::new(&mut self.search_time, 100..=15000));

            ui.add_space(20.);

            ui.label("Max playouts: ");
            ui.add(Slider::new(&mut self.max_playouts, 1..=15000));
        });
    }

    fn create_agent(&self) -> Box<dyn Agent> {
        Box::new(Mcts {
            search_time: Duration::from_millis(self.search_time),
            min_node_visits: self.min_node_visits,
            exploration_c: self.exploration_c,
            max_playouts: self.max_playouts,
        })
    }

    fn participant(&self) -> Participant {
        Participant::Mcts
    }
}
