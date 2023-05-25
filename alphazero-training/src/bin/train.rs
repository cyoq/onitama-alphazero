use std::time::Duration;

use alphazero_training::{
    alphazero_mcts::AlphaZeroMctsConfig,
    evaluator::EvaluatorConfig,
    train::{train, TrainConfig},
};

fn main() {
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
            search_time: Duration::from_millis(50),
            max_playouts: 100,
            ..Default::default()
        },
        iterations: 1,
        self_play_game_amnt: 10,
        save_checkpoint: 1,
        evaluation_checkpoint: 1,
        evaluator_config: EvaluatorConfig {
            game_amnt: 5,
            ..Default::default()
        },
        ..Default::default()
    };
    match train(train_config) {
        Ok(_) => println!("Success!"),
        Err(e) => println!("An error occured: {}", e),
    };
}
