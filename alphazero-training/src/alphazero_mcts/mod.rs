pub mod mcts_arena;

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use onitama_game::{
    ai::agent::Agent,
    game::{
        done_move::DoneMove, game_state::GameState, move_result::MoveResult,
        player_color::PlayerColor, state::State,
    },
};

use serde::{ser::SerializeStruct, Deserialize, Serialize};
use tch::{nn, Tensor};

use crate::{
    common::Options,
    net::{ConvResNet, ConvResNetConfig},
};

use self::mcts_arena::MctsArena;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlphaZeroMctsConfig {
    pub search_time: Duration,
    pub exploration_c: f64,
    pub max_playouts: u32,
    pub train: bool,
}

impl Default for AlphaZeroMctsConfig {
    fn default() -> Self {
        Self {
            search_time: Duration::from_millis(400),
            exploration_c: 2f64.sqrt(),
            max_playouts: 5000,
            train: false,
        }
    }
}

pub fn reward(move_result: MoveResult, reward_color: PlayerColor) -> f64 {
    match (reward_color, move_result) {
        (PlayerColor::Red, MoveResult::RedWin) => 1.,
        (PlayerColor::Red, MoveResult::BlueWin) => -1.,
        (PlayerColor::Blue, MoveResult::RedWin) => -1.,
        (PlayerColor::Blue, MoveResult::BlueWin) => 1.,
        _ => 0.,
    }
}

#[derive(Debug)]
pub struct TrainingAlphaZeroMcts {
    pub config: AlphaZeroMctsConfig,
    pub model: ConvResNet,
    pub options: Options,
}

impl TrainingAlphaZeroMcts {
    pub fn generate_move_tensor(
        &self,
        state: &State,
        curr_player_color: PlayerColor,
    ) -> (DoneMove, Tensor) {
        let mut arena = MctsArena::new(
            state.clone(),
            curr_player_color,
            self.config.clone(),
            &self.model,
            self.options,
            reward,
        );

        arena.search()
    }
}

#[derive(Debug, Clone)]
pub struct AlphaZeroMcts {
    pub config: AlphaZeroMctsConfig,
    pub model: Arc<Mutex<ConvResNet>>,
    pub options: Options,
}

impl AlphaZeroMcts {
    pub fn from_model_file(
        vs: &mut nn::VarStore,
        model_path: &str,
        config: AlphaZeroMctsConfig,
        net_config: ConvResNetConfig,
        options: Options,
    ) -> Self {
        let model = Arc::new(Mutex::new(ConvResNet::new(&vs.root(), net_config, options)));
        if let Err(e) = vs.load(model_path) {
            eprintln!("An error occurred while loading the file: {}", e);
        }
        Self {
            config,
            model,
            options,
        }
    }
}

impl Serialize for AlphaZeroMcts {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("AlphaZeroMcts", 3)?;
        state.serialize_field("config", &self.config)?;
        state.serialize_field("model", &self.model.lock().unwrap().id)?;
        state.serialize_field("options", "TODO: most likely FLOAT_CUDA")?;

        state.end()
    }
}

impl Agent for AlphaZeroMcts {
    fn generate_move(&self, game_state: &GameState) -> (DoneMove, f64) {
        let model = self.model.lock().unwrap();
        let mut arena = MctsArena::new(
            game_state.state.clone(),
            game_state.curr_player_color,
            self.config.clone(),
            &model,
            self.options,
            reward,
        );

        let (mov, _priors) = arena.search();
        // priors.reshape(&[2, 5, 5]).print();

        let res = arena.evaluate_state(&game_state.state, game_state.curr_player_color);
        // res.policy.squeeze_dim(0).reshape(&[2, 5, 5]).print();
        // res.value.print();

        let value = f64::from(res.value.squeeze_dim(0));

        (mov, value)
    }

    fn name(&self) -> &'static str {
        "AlphaZero MCTS AI"
    }

    fn clone_dyn(&self) -> Box<dyn Agent> {
        Box::new(self.clone())
    }

    fn id(&self) -> u64 {
        self.config.search_time.as_nanos() as u64
            + self.config.exploration_c as u64
            + self.config.max_playouts as u64
            + self.config.train as u64
            + self.model.lock().unwrap().id.parse::<u64>().unwrap()
    }
}
