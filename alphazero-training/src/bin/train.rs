use std::{fs::File, sync::Arc, time::Duration};

use alphazero_training::{
    alphazero_mcts::AlphaZeroMctsConfig,
    evaluator::EvaluatorConfig,
    train::{train, TrainConfig},
};
use chrono::Local;
use tracing_subscriber::{filter, prelude::*, util::SubscriberInitExt, Layer};

fn main() {
    // Tracing configuration taken from: https://stackoverflow.com/a/70042590
    let stdout_log = tracing_subscriber::fmt::layer().pretty();

    // A layer that logs events to a file.
    let file = File::create(format!(
        "logs/debug_{}.log",
        Local::now().format("%Y%m%d_%H%M%S")
    ));
    let file = match file {
        Ok(file) => file,
        Err(error) => panic!("Error: {:?}", error),
    };
    let debug_log = tracing_subscriber::fmt::layer().with_writer(Arc::new(file));
    tracing_subscriber::registry()
        .with(
            stdout_log
                // Add an `INFO` filter to the stdout logging layer
                .with_filter(filter::LevelFilter::INFO)
                // Combine the filtered `stdout_log` layer with the
                // `debug_log` layer, producing a new `Layered` layer.
                .and_then(debug_log),
        )
        .init();

    // TODO: playing with the stable deck at the moment
    // let deck = Some(Deck::new([
    //     ORIGINAL_CARDS[0].clone(),
    //     ORIGINAL_CARDS[1].clone(),
    //     ORIGINAL_CARDS[2].clone(),
    //     ORIGINAL_CARDS[3].clone(),
    //     ORIGINAL_CARDS[4].clone(),
    // ]));

    // let train_config = TrainConfig {
    //     iterations: 10,
    //     ..Default::default()
    // };
    let train_config = TrainConfig {
        mcts_config: AlphaZeroMctsConfig {
            search_time: Duration::from_millis(200),
            max_playouts: 400,
            ..Default::default()
        },
        iterations: 20,
        self_play_game_amnt: 100,
        save_checkpoint: 2,
        evaluation_checkpoint: 2,
        evaluator_config: EvaluatorConfig {
            game_amnt: 20,
            ..Default::default()
        },
        ..Default::default()
    };

    match train(train_config) {
        Ok(_) => println!("Success!"),
        Err(e) => println!("An error occured: {}", e),
    };
}
