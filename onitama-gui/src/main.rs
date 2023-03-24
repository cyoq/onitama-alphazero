use eframe::{run_native, NativeOptions};
use egui::{Vec2, Visuals};
use onitama::Onitama;
use onitama_game::ai::{alpha_beta::AlphaBeta, human_gui::HumanGui, random::Random};
use player::{Player, PlayerType};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

pub mod game_board;
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

    let red_player = Player {
        typ: PlayerType::Human,
        agent: Box::new(HumanGui),
    };

    // let blue_player = Player {
    //     typ: PlayerType::Human,
    //     agent: Box::new(HumanGui),
    // };

    let blue_player = Player {
        typ: PlayerType::Ai,
        // agent: Box::new(AlphaBeta { max_depth: 6 }),
        agent: Box::new(Random),
    };

    run_native(
        "Onitama",
        NativeOptions {
            initial_window_size: Some(Vec2::new(1800.0, 840.0)),
            ..Default::default()
        },
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(Visuals::light());
            Box::new(Onitama::new(cc, false, red_player, blue_player))
        }),
    )
}
