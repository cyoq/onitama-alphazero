use eframe::{run_native, NativeOptions};
use egui::{Vec2, Visuals};
use onitama::{Onitama, PlayerType};
use onitama_game::ai::{human_gui::HumanGui, random::Random};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

pub mod game_board;
pub mod image;
pub mod move_card;
pub mod onitama;

fn main() -> Result<(), eframe::Error> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let red_agent = HumanGui;
    let blue_agent = Random;

    run_native(
        "Onitama",
        NativeOptions {
            initial_window_size: Some(Vec2::new(1800.0, 840.0)),
            ..Default::default()
        },
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(Visuals::light());
            Box::new(Onitama::new(
                cc,
                false,
                Box::new(red_agent),
                Box::new(blue_agent),
                [PlayerType::Human, PlayerType::Ai],
            ))
        }),
    )
}
