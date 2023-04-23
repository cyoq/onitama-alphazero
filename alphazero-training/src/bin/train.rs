use alphazero_training::train::train;

fn main() {
    match train(10) {
        Ok(_) => println!("Success!"),
        Err(e) => println!("An error occured: {}", e),
    };
}
