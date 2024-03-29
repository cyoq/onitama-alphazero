use onitama_game::ai::agent::Agent;
use onitama_game::ai::{human_console::HumanConsole, random::Random};
use onitama_game::game::{game_state::GameState, move_result::MoveResult};

fn main() {
    let mut progress = MoveResult::InProgress;

    let red_agent = HumanConsole;
    let blue_agent = Random;

    let agents: [Box<dyn Agent>; 2] = [Box::new(red_agent), Box::new(blue_agent)];
    let mut game = GameState::new();
    let mut max_plies = 200;

    while progress != MoveResult::BlueWin && progress != MoveResult::RedWin {
        println!("{}", game.state.deck.display());
        println!("{}", game.state.display());

        if max_plies < 0 {
            println!("Game has become infinite!");
            break;
        }

        let (mov, _evaluation) = agents[game.curr_agent_idx].generate_move(&game);

        progress = game.progress(mov);
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
