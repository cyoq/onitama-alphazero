use eframe::{run_native, NativeOptions};
use egui::{Vec2, Visuals};
use onitama::Onitama;
use onitama_game::ai::human_gui::HumanGui;

pub mod game_board;
pub mod image;
pub mod move_card;
pub mod onitama;

fn main() -> Result<(), eframe::Error> {
    tracing_subscriber::fmt::init();

    let red_agent = HumanGui;
    let blue_agent = HumanGui;

    run_native(
        "Onitama",
        NativeOptions {
            initial_window_size: Some(Vec2::new(1800.0, 840.0)),
            ..Default::default()
        },
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(Visuals::light());
            Box::new(Onitama::new(cc, Box::new(red_agent), Box::new(blue_agent)))
        }),
    )
}
