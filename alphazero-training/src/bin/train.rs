use alphazero_training::train::{train, TrainConfig};

fn main() {
    let train_config = TrainConfig {
        iterations: 10,
        ..Default::default()
    };
    match train(train_config) {
        Ok(_) => println!("Success!"),
        Err(e) => println!("An error occured: {}", e),
    };
}
