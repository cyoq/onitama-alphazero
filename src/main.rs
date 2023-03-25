use onitama_game::ai::{human_console::HumanConsole, random::Random};
use onitama_game::game::{game_state::GameState, move_result::MoveResult};

fn main() {
    let mut progress = MoveResult::InProgress;

    let red_agent = HumanConsole;
    let blue_agent = Random;

    let mut game = GameState::new(Box::new(red_agent), Box::new(blue_agent));
    let mut max_plies = 200;

    while progress != MoveResult::BlueWin && progress != MoveResult::RedWin {
        println!("{}", game.state.deck.display());
        println!("{}", game.state.display());

        if max_plies < 0 {
            println!("Game has become infinite!");
            break;
        }

        progress = game.next_turn();
        max_plies -= 1;
    }

    println!("{}", game.state.display());
    if max_plies != 0 {
        match progress {
            MoveResult::Capture => (),
            MoveResult::RedWin => println!("Red won!"),
            MoveResult::BlueWin => println!("Blue won!"),
            MoveResult::InProgress => (),
        }
    }
}
