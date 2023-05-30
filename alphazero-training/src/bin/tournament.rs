use std::time::{Duration, Instant};

use alphazero_training::{
    alphazero_mcts::{AlphaZeroMcts, AlphaZeroMctsConfig},
    common::Options,
    elo_rating::EloRating,
    net::ConvResNetConfig,
};
use onitama_game::{
    ai::{agent::Agent, alpha_beta::AlphaBeta, mcts::Mcts, random::Random},
    game::{
        card::CARD_NAMES, deck::Deck, game_state::GameState, move_result::MoveResult,
        player_color::PlayerColor,
    },
};
use tch::{kind, nn::VarStore, Device};

fn play(
    agent: &Box<dyn Agent>,
    opponent: &Box<dyn Agent>,
    ra: &mut f64,
    rb: &mut f64,
    game_amnt: usize,
    decks: &Vec<Deck>,
) {
    let mut agents = [agent, opponent];
    let mut agent_color = PlayerColor::Red;
    let mut wins = 0;
    let mut game = 0;

    let now = Instant::now();
    while game < game_amnt {
        let deck = decks[game].clone();
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
        match (progress, agent_color) {
            (MoveResult::BlueWin, PlayerColor::Blue) | (MoveResult::RedWin, PlayerColor::Red) => {
                wins += 1;
                (*ra, *rb) = EloRating::elo_change(*ra, *rb, true);
            }
            (MoveResult::BlueWin, PlayerColor::Red) | (MoveResult::RedWin, PlayerColor::Blue) => {
                (*ra, *rb) = EloRating::elo_change(*ra, *rb, false);
            }
            _ => {}
        };

        agent_color.switch();
        agents.swap(0, 1);
        game += 1;
    }

    println!(
        "{} ({:5.2}) vs {} ({:5.2}) -> winrate: {:3.2}",
        agent.name(),
        ra,
        opponent.name(),
        rb,
        (wins as f64 / game_amnt as f64),
    );
    println!("Elapsed: {:?}\n", now.elapsed());
}

const GAME_AMNT: usize = 100;

pub fn pit() {
    let mut alphabeta_rating = 800.;
    let mut mcts_rating = 800.;
    let mut alphazero_rating = 800.;
    let mut random_rating = 800.;

    let mut decks = Vec::with_capacity(GAME_AMNT);
    for _ in 0..GAME_AMNT {
        decks.push(Deck::default());
    }
    println!("Using decks: ");
    for i in 0..decks.len() {
        println!(
            "{}. Deck: {:?}, Color: {:?}",
            i + 1,
            decks[i]
                .cards
                .iter()
                .map(|c| CARD_NAMES[c.index])
                .collect::<Vec<_>>(),
            decks[i].neutral_card().player_color
        );
    }

    let alphabeta: Box<dyn Agent> = Box::new(AlphaBeta {
        max_depth: 8,
        search_time: Duration::from_secs(1),
    });

    let mcts: Box<dyn Agent> = Box::new(Mcts {
        search_time: Duration::from_secs(1),
        min_node_visits: 0,
        exploration_c: 1.0,
        max_playouts: 5000,
    });

    let mut vs = VarStore::new(Device::Cpu);
    let net_config = ConvResNetConfig::default();
    let alphazero: Box<dyn Agent> = Box::new(AlphaZeroMcts::from_model_file(
        &mut vs,
        "../models/model.ot",
        AlphaZeroMctsConfig {
            search_time: Duration::from_secs(1),
            exploration_c: 1.0,
            max_playouts: 5000,
            train: false,
        },
        net_config,
        Options::new(kind::FLOAT_CPU),
    ));

    let random: Box<dyn Agent> = Box::new(Random);

    play(
        &alphabeta,
        &mcts,
        &mut alphabeta_rating,
        &mut mcts_rating,
        GAME_AMNT,
        &decks,
    );

    play(
        &alphabeta,
        &alphazero,
        &mut alphabeta_rating,
        &mut alphazero_rating,
        GAME_AMNT,
        &decks,
    );

    play(
        &alphabeta,
        &random,
        &mut alphabeta_rating,
        &mut random_rating,
        GAME_AMNT,
        &decks,
    );

    play(
        &mcts,
        &alphazero,
        &mut mcts_rating,
        &mut alphazero_rating,
        GAME_AMNT,
        &decks,
    );

    play(
        &mcts,
        &random,
        &mut mcts_rating,
        &mut random_rating,
        GAME_AMNT,
        &decks,
    );

    play(
        &alphazero,
        &random,
        &mut alphazero_rating,
        &mut random_rating,
        GAME_AMNT,
        &decks,
    );
}

fn main() {
    pit();
}
