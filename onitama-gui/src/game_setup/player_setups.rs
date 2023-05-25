use std::time::Duration;

use alphazero_training::{
    alphazero_mcts::{AlphaZeroMctsConfig, TrainingAlphaZeroMcts},
    common::Options,
    net::ConvResNetConfig,
};
use egui::{Align, Layout, RichText, Slider, Ui};
use onitama_game::ai::{
    agent::Agent, alpha_beta::AlphaBeta, human_gui::HumanGui, mcts::Mcts, random::Random,
};
use tch::{kind, nn, Device};

use crate::player::PlayerType;

pub trait PlayerSetup: Send {
    fn show(&mut self, ui: &mut Ui);
    fn player_type(&self) -> PlayerType;
    fn create_agent(&self) -> Box<dyn Agent>;
}

pub fn create_player_setup(player_type: &PlayerType) -> Box<dyn PlayerSetup> {
    match *player_type {
        PlayerType::Human => Box::new(HumanSetup::default()),
        PlayerType::Random => Box::new(RandomSetup::default()),
        PlayerType::AlphaBeta => Box::new(AlphaBetaSetup::default()),
        PlayerType::Mcts => Box::new(MctsSetup::default()),
        PlayerType::AlphaZero => Box::new(AlphaZeroSetup::default()),
    }
}

#[derive(Default)]
pub struct HumanSetup;

impl PlayerSetup for HumanSetup {
    fn show(&mut self, ui: &mut Ui) {
        ui.add_space(20.);
        ui.label(RichText::new("Human parameters").text_style(egui::TextStyle::Heading));

        ui.label("Human does not have any parameters!");
    }

    fn create_agent(&self) -> Box<dyn Agent> {
        Box::new(HumanGui)
    }

    fn player_type(&self) -> PlayerType {
        PlayerType::Human
    }
}

#[derive(Default)]
pub struct RandomSetup;

impl PlayerSetup for RandomSetup {
    fn show(&mut self, ui: &mut Ui) {
        ui.add_space(20.);
        ui.label(RichText::new("Random parameters").text_style(egui::TextStyle::Heading));

        ui.label("Random does not have any parameters!");
    }

    fn create_agent(&self) -> Box<dyn Agent> {
        Box::new(Random)
    }

    fn player_type(&self) -> PlayerType {
        PlayerType::Random
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

impl PlayerSetup for AlphaBetaSetup {
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
            search_time: Duration::from_millis(self.search_time),
        })
    }

    fn player_type(&self) -> PlayerType {
        PlayerType::AlphaBeta
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

impl PlayerSetup for MctsSetup {
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
            ui.add(Slider::new(&mut self.max_playouts, 1..=1000000));
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

    fn player_type(&self) -> PlayerType {
        PlayerType::Mcts
    }
}

pub struct AlphaZeroSetup {
    pub search_time: u64,
    pub exploration_c: f64,
    pub max_playouts: u32,
    pub model_path: String,
}

impl Default for AlphaZeroSetup {
    fn default() -> Self {
        Self {
            search_time: 1000,
            exploration_c: 2f64.sqrt(),
            max_playouts: 5000,
            model_path: "".to_owned(),
        }
    }
}

impl PlayerSetup for AlphaZeroSetup {
    fn show(&mut self, ui: &mut Ui) {
        ui.add_space(20.);
        ui.label(RichText::new("AlphaZero MCTS parameters").text_style(egui::TextStyle::Heading));
        ui.with_layout(Layout::left_to_right(Align::Max), |ui| {
            ui.label("Exploration constant: ");
            ui.add(Slider::new(&mut self.exploration_c, 0.0..=10.0));

            ui.add_space(20.);

            ui.label("Search time(ms): ");
            ui.add(Slider::new(&mut self.search_time, 100..=15000));

            ui.add_space(20.);

            ui.label("Max playouts: ");
            ui.add(Slider::new(&mut self.max_playouts, 1..=15000));
        });
    }

    fn create_agent(&self) -> Box<dyn Agent> {
        // TODO: This thing must be receved from the application
        let mut vs = nn::VarStore::new(Device::cuda_if_available());
        let config = AlphaZeroMctsConfig {
            search_time: Duration::from_millis(self.search_time),
            exploration_c: self.exploration_c,
            max_playouts: self.max_playouts,
            train: false,
        };
        let net_config = ConvResNetConfig::default();
        let options = Options::new(kind::FLOAT_CUDA);
        Box::new(TrainingAlphaZeroMcts::from_model_file(
            &mut vs,
            "./models/best_model_20230425_163514.ot",
            config,
            net_config,
            options,
        ))
    }

    fn player_type(&self) -> PlayerType {
        PlayerType::Mcts
    }
}
