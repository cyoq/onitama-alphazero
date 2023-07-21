use std::time::{Duration, Instant};

use onitama_game::{
    ai::{agent::Agent, mcts::Mcts},
    game::{deck::Deck, game_state::GameState, move_result::MoveResult, player_color::PlayerColor},
};

pub fn play(agent: Box<dyn Agent>, opponent: Box<dyn Agent>, game_amnt: u32) -> u32 {
    let mut agents = [agent, opponent];
    let mut agent_color = PlayerColor::Red;
    let mut wins = 0;
    let mut game = 0;

    while game < game_amnt {
        let deck = Deck::default();
        let mut state = GameState::with_deck(deck);
        let mut progress = MoveResult::InProgress;

        let mut max_plies = 150;

        while !progress.is_win() {
            let (done_move, _) = agents[state.curr_agent_idx].generate_move(&state);

            progress = state.progress(done_move);

            if max_plies < 0 {
                println!("Game has become infinite!");
                game -= 1;
                break;
            }

            max_plies -= 1;
        }

        // Gather statistics
        wins += match (progress, agent_color) {
            (MoveResult::BlueWin, PlayerColor::Blue) | (MoveResult::RedWin, PlayerColor::Red) => 1,
            _ => 0,
        };

        agent_color.switch();
        agents.swap(0, 1);
        game += 1;
    }

    wins
}

pub fn play_with_c_value() {
    let game_amnt = 100;
    let c_values = [0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0];

    for i in 0..c_values.len() {
        let mcts1: Box<dyn Agent> = Box::new(Mcts {
            search_time: Duration::from_millis(400),
            min_node_visits: 1,
            exploration_c: c_values[i],
            max_playouts: 1600,
        });

        let mut handles = vec![];
        for k in (i + 1)..c_values.len() {
            let mcts2: Box<dyn Agent> = Box::new(Mcts {
                search_time: Duration::from_millis(400),
                min_node_visits: 1,
                exploration_c: c_values[k],
                max_playouts: 1600,
            });

            let clone = mcts1.clone();
            let handle = std::thread::spawn(move || play(clone, mcts2, game_amnt));
            handles.push(handle);
        }

        let now = Instant::now();
        for (j, handle) in handles.into_iter().enumerate() {
            let wins = handle.join().unwrap();
            println!(
                "MCTS with c = {:3.2} vs MCTS with c = {:3.2} -> winrate: {:3.2}",
                c_values[i],
                c_values[i + j + 1],
                (wins as f64 / game_amnt as f64),
            );
        }
        println!("Elapsed: {:?}\n", now.elapsed());
    }
}

pub fn play_with_n_value() {
    let game_amnt = 100;
    let n_values = [0, 1, 2, 3, 4, 5, 6, 7, 8];

    for i in 0..n_values.len() {
        let mcts1: Box<dyn Agent> = Box::new(Mcts {
            search_time: Duration::from_millis(400),
            min_node_visits: n_values[i],
            exploration_c: 1.,
            max_playouts: 1600,
        });

        let mut handles = vec![];
        for k in (i + 1)..n_values.len() {
            let mcts2: Box<dyn Agent> = Box::new(Mcts {
                search_time: Duration::from_millis(400),
                min_node_visits: n_values[k],
                exploration_c: 1.,
                max_playouts: 1600,
            });

            let clone = mcts1.clone();
            let handle = std::thread::spawn(move || play(clone, mcts2, game_amnt));
            handles.push(handle);
        }

        let now = Instant::now();
        for (j, handle) in handles.into_iter().enumerate() {
            let wins = handle.join().unwrap();
            println!(
                "MCTS with n = {} vs MCTS with n = {} -> winrate: {:3.2}",
                n_values[i],
                n_values[i + j + 1],
                (wins as f64 / game_amnt as f64),
            );
        }
        println!("Elapsed: {:?}\n", now.elapsed());
    }
}

fn main() {
    // play_with_c_value();
    play_with_n_value();
}
