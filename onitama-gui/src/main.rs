use eframe::{run_native, NativeOptions};
use egui::{Vec2, Visuals};
use onitama::Onitama;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

pub mod game_board;
pub mod game_config;
pub mod game_setup;
pub mod image;
pub mod move_card;
pub mod onitama;
pub mod player;
pub mod selected_card;

fn main() -> Result<(), eframe::Error> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    run_native(
        "Onitama",
        NativeOptions {
            initial_window_size: Some(Vec2::new(1800.0, 840.0)),
            ..Default::default()
        },
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(Visuals::light());
            let mut onitama = Onitama::new(cc, false);
            onitama.start();
            Box::new(onitama)
        }),
    )
}
