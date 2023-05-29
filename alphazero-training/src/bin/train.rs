use std::{fs::File, sync::Arc, time::Duration};

use alphazero_training::{
    alphazero_mcts::AlphaZeroMctsConfig,
    evaluator::EvaluatorConfig,
    net::ConvResNetConfig,
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
            search_time: Duration::from_millis(20),
            max_playouts: 400,
            exploration_c: 5.,
            train: true,
        },
        model_config: ConvResNetConfig {
            hidden_channels: 64,
            // first model
            input_channels: 21,
            resnet_block_amnt: 5,
            // second model
            // input_channels: 10,
            // resnet_block_amnt: 5,
        },
        iterations: 10,
        self_play_game_amnt: 100,
        save_checkpoint: 2,
        evaluation_checkpoint: 5,
        thread_amnt: 12,
        learning_rate: 5e-3,
        evaluator_config: EvaluatorConfig {
            winrate_percent: 0.55,
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
