use std::{
    sync::{Arc, Mutex},
    thread::JoinHandle,
    time::Duration,
};

use onitama_game::{
    ai::{agent::Agent, alpha_beta::AlphaBeta, mcts::Mcts, random::Random},
    game::{deck::Deck, game_state::GameState, move_result::MoveResult, player_color::PlayerColor},
};
use serde::{Deserialize, Serialize};
use tch::nn;
use tracing::error;

use crate::{
    alphazero_mcts::{AlphaZeroMcts, AlphaZeroMctsConfig},
    common::Options,
    elo_rating::{EloRating, PlayerRating},
    net::{ConvResNet, ConvResNetConfig},
};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct WinLoseDraws {
    pub wins: u64,
    pub loses: u64,
    pub draws: u64,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RatingChange {
    pub before_a: f64,
    pub after_a: f64,
    pub before_b: f64,
    pub after_b: f64,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FightStatistics {
    pub general: WinLoseDraws,
    pub winrate: f64,
    // win, lose, draws with the specific color, their sum must equal to general
    pub color: [WinLoseDraws; 2],
    pub color_winrate: [f64; 2],
    pub rating_a: f64,
    pub rating_b: f64,
    pub rating_change_history: Vec<RatingChange>,
}

impl FightStatistics {
    pub fn new(rating_a: f64, rating_b: f64) -> Self {
        Self {
            rating_a,
            rating_b,
            ..Default::default()
        }
    }

    pub fn update(&mut self, move_result: MoveResult, player_color: PlayerColor) {
        match (move_result, player_color) {
            (MoveResult::BlueWin, PlayerColor::Red) | (MoveResult::RedWin, PlayerColor::Blue) => {
                let (ra, rb) = EloRating::elo_change(self.rating_a, self.rating_b, false);
                self.rating_change_history.push(RatingChange {
                    before_a: self.rating_a,
                    after_a: ra,
                    before_b: self.rating_b,
                    after_b: rb,
                });
                self.rating_a = ra;
                self.rating_b = rb;

                self.general.loses += 1;
                self.color[player_color as usize].loses += 1;
            }
            (MoveResult::BlueWin, PlayerColor::Blue) | (MoveResult::RedWin, PlayerColor::Red) => {
                let (ra, rb) = EloRating::elo_change(self.rating_a, self.rating_b, true);
                self.rating_change_history.push(RatingChange {
                    before_a: self.rating_a,
                    after_a: ra,
                    before_b: self.rating_b,
                    after_b: rb,
                });
                self.rating_a = ra;
                self.rating_b = rb;

                self.general.wins += 1;
                self.color[player_color as usize].wins += 1;
            }
            _ => {
                self.rating_change_history.push(RatingChange {
                    before_a: self.rating_a,
                    after_a: self.rating_a,
                    before_b: self.rating_b,
                    after_b: self.rating_b,
                });
                self.general.draws += 1;
                self.color[player_color as usize].draws += 1;
            }
        }
        self.update_winrate();
    }

    pub fn update_winrate(&mut self) {
        self.winrate = self.general.wins as f64
            / (self.general.loses + self.general.draws + self.general.wins) as f64;
        self.color_winrate[0] = self.color[0].wins as f64
            / (self.color[0].loses + self.color[0].draws + self.color[0].wins) as f64;
        self.color_winrate[1] = self.color[1].wins as f64
            / (self.color[1].loses + self.color[1].draws + self.color[1].wins) as f64;
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PitStatistics {
    pub self_fight: FightStatistics,
    pub random_fight: FightStatistics,
    pub alphabeta_fight: FightStatistics,
    pub mcts_fight: FightStatistics,
}

#[derive(Clone, Debug)]
pub struct EvaluatorConfig {
    pub winrate_percent: f64,
    pub game_amnt: u64,
    pub deck: Option<Deck>,
    pub max_plies: i64,
}

impl Default for EvaluatorConfig {
    fn default() -> Self {
        Self {
            winrate_percent: 0.55,
            game_amnt: 20,
            deck: None,
            max_plies: 150,
        }
    }
}

pub struct Evaluator<'a> {
    pub config: EvaluatorConfig,
    pub best_nn_vs: &'a nn::VarStore,
    pub new_nn_vs: &'a nn::VarStore,
    pub net_config: &'a ConvResNetConfig,
    pub ratings: &'a [[PlayerRating; 2]; 4],
    pub options: Options,
}

impl<'a> Evaluator<'a> {
    // Returns if new neural net is better than the old one
    pub fn new(
        config: EvaluatorConfig,
        best_nn_vs: &'a nn::VarStore,
        new_nn_vs: &'a nn::VarStore,
        net_config: &'a ConvResNetConfig,
        ratings: &'a [[PlayerRating; 2]; 4],
        options: Options,
    ) -> Self {
        Self {
            config,
            best_nn_vs,
            new_nn_vs,
            net_config,
            ratings,
            options,
        }
    }

    /// Returns boolean which tells if the new model is better than the best one
    pub fn pit(&mut self) -> (PitStatistics, bool) {
        // Apply handles to start threads working in parallel
        let self_fight_handle = self.fight_against_best(&self.ratings[0]);
        let random_fight_handle = self.fight_against_random(&self.ratings[1]);
        let mcts_fight_handle = self.fight_against_mcts(&self.ratings[2]);
        let alphabeta_fight_handle = self.fight_against_alphabeta(&self.ratings[3]);

        let self_fight = self_fight_handle.join().unwrap();
        let random_fight = random_fight_handle.join().unwrap();
        let mcts_fight = mcts_fight_handle.join().unwrap();
        let alphabeta_fight = alphabeta_fight_handle.join().unwrap();

        let mut is_best = false;
        if self_fight.winrate > self.config.winrate_percent {
            is_best = true;
        }
        let statistics = PitStatistics {
            self_fight,
            random_fight,
            mcts_fight,
            alphabeta_fight,
        };

        (statistics, is_best)
    }

    pub fn fight_against_best(
        &self,
        ratings: &'a [PlayerRating; 2],
    ) -> JoinHandle<FightStatistics> {
        let mcts_config = AlphaZeroMctsConfig {
            search_time: Duration::from_millis(400),
            max_playouts: 400,
            train: false,
            ..Default::default()
        };
        let mut train_vs = nn::VarStore::new(self.options.device);
        if let Err(e) = train_vs.copy(self.new_nn_vs) {
            error!("Was not able to copy VarStore: {}", e);
        }
        let training_model = Arc::new(Mutex::new(ConvResNet::new(
            &train_vs.root(),
            self.net_config.clone(),
            self.options,
        )));
        let train_mcts = AlphaZeroMcts {
            config: mcts_config.clone(),
            model: training_model,
            options: self.options,
        };

        let mut best_vs = nn::VarStore::new(self.options.device);
        if let Err(e) = best_vs.copy(self.best_nn_vs) {
            error!("Was not able to copy VarStore: {}", e);
        }
        let best_model = Arc::new(Mutex::new(ConvResNet::new(
            &best_vs.root(),
            self.net_config.clone(),
            self.options,
        )));

        let best_mcts = AlphaZeroMcts {
            config: mcts_config.clone(),
            model: best_model,
            options: self.options,
        };

        let config = self.config.clone();
        let ra = *ratings[0].rating;
        let rb = *ratings[1].rating;
        std::thread::spawn(move || fight(config, Box::new(train_mcts), Box::new(best_mcts), ra, rb))
    }

    pub fn fight_against_random(
        &self,
        ratings: &'a [PlayerRating; 2],
    ) -> JoinHandle<FightStatistics> {
        let mcts_config = AlphaZeroMctsConfig {
            search_time: Duration::from_millis(400),
            max_playouts: 400,
            train: false,
            ..Default::default()
        };

        let mut train_vs = nn::VarStore::new(self.options.device);
        if let Err(e) = train_vs.copy(self.new_nn_vs) {
            error!("Was not able to copy VarStore: {}", e);
        }
        let model = Arc::new(Mutex::new(ConvResNet::new(
            &train_vs.root(),
            self.net_config.clone(),
            self.options,
        )));
        let mcts = AlphaZeroMcts {
            config: mcts_config.clone(),
            model,
            options: self.options,
        };

        let random = Random;

        let config = self.config.clone();
        let ra = *ratings[0].rating;
        let rb = *ratings[1].rating;
        std::thread::spawn(move || fight(config, Box::new(mcts), Box::new(random), ra, rb))
    }

    pub fn fight_against_alphabeta(
        &self,
        ratings: &'a [PlayerRating; 2],
    ) -> JoinHandle<FightStatistics> {
        let mcts_config = AlphaZeroMctsConfig {
            search_time: Duration::from_millis(400),
            max_playouts: 400,
            train: false,
            ..Default::default()
        };

        let mut train_vs = nn::VarStore::new(self.options.device);
        if let Err(e) = train_vs.copy(self.new_nn_vs) {
            error!("Was not able to copy VarStore: {}", e);
        }

        let model = Arc::new(Mutex::new(ConvResNet::new(
            &train_vs.root(),
            self.net_config.clone(),
            self.options,
        )));
        let mcts = AlphaZeroMcts {
            config: mcts_config.clone(),
            model,
            options: self.options,
        };

        let alphabeta = AlphaBeta {
            max_depth: 4,
            search_time: Duration::from_millis(400),
        };

        let config = self.config.clone();
        let ra = *ratings[0].rating;
        let rb = *ratings[1].rating;
        std::thread::spawn(move || fight(config, Box::new(mcts), Box::new(alphabeta), ra, rb))
    }

    pub fn fight_against_mcts(
        &self,
        ratings: &'a [PlayerRating; 2],
    ) -> JoinHandle<FightStatistics> {
        let mcts_config = AlphaZeroMctsConfig {
            search_time: Duration::from_millis(400),
            max_playouts: 400,
            train: false,
            ..Default::default()
        };

        let mut train_vs = nn::VarStore::new(self.options.device);
        if let Err(e) = train_vs.copy(self.new_nn_vs) {
            error!("Was not able to copy VarStore: {}", e);
        }

        let model = Arc::new(Mutex::new(ConvResNet::new(
            &train_vs.root(),
            self.net_config.clone(),
            self.options,
        )));
        let a0_mcts = AlphaZeroMcts {
            config: mcts_config.clone(),
            model,
            options: self.options,
        };

        let mcts = Mcts {
            search_time: Duration::from_millis(400),
            min_node_visits: 5,
            exploration_c: 1.41,
            max_playouts: 400,
        };

        let config = self.config.clone();
        let ra = *ratings[0].rating;
        let rb = *ratings[1].rating;
        std::thread::spawn(move || fight(config, Box::new(a0_mcts), Box::new(mcts), ra, rb))
    }
}

pub fn fight(
    config: EvaluatorConfig,
    agent: Box<dyn Agent>,
    opponent: Box<dyn Agent>,
    agent_rating: f64,
    opponent_rating: f64,
) -> FightStatistics {
    let mut agents = [agent, opponent];
    let mut agent_color = PlayerColor::Red;
    let mut statistics = FightStatistics::new(agent_rating, opponent_rating);

    for _ in 0..config.game_amnt {
        let deck: Deck;
        if let None = config.deck {
            deck = Deck::default();
        } else {
            deck = config.deck.clone().unwrap_or_default();
        }
        let mut state = GameState::with_deck(deck);
        let mut progress = MoveResult::InProgress;

        let mut max_plies = config.max_plies;

        while !progress.is_win() {
            let (done_move, _) = agents[state.curr_agent_idx].generate_move(&state);

            progress = state.progress(done_move);

            if max_plies < 0 {
                println!("Game has become infinite!");
                break;
            }

            max_plies -= 1;
        }

        // Gather statistics
        statistics.update(progress, agent_color);

        agent_color.switch();
        agents.swap(0, 1);
    }

    statistics
}
