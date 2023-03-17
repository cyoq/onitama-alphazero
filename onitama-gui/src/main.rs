use eframe::{run_native, NativeOptions};
use egui::{Vec2, Visuals};
use onitama::Onitama;

pub mod cell;
pub mod onitama;

fn main() -> Result<(), eframe::Error> {
    tracing_subscriber::fmt::init();

    run_native(
        "Onitama",
        NativeOptions {
            initial_window_size: Some(Vec2::new(1800.0, 840.0)),
            ..Default::default()
        },
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(Visuals::light());
            Box::new(Onitama::new(cc))
        }),
    )
}
