use alphazero_training::train::train;

fn main() {
    match train(50) {
        Ok(_) => println!("Success!"),
        Err(e) => println!("An error occured: {}", e),
    };
}
