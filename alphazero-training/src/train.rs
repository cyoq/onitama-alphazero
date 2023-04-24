use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use chrono::Local;
use onitama_game::game::{
    card::ORIGINAL_CARDS, deck::Deck, move_result::MoveResult, player_color::PlayerColor,
    state::State,
};
use rand::{seq::IteratorRandom, thread_rng};
use serde::{Deserialize, Serialize};
use tch::{
    kind,
    nn::{self, OptimizerConfig},
    Device, Tensor,
};

use crate::{
    alphazero_mcts::{AlphaZeroMcts, AlphaZeroMctsConfig},
    common::{create_tensor_from_state, Options},
    evaluator::{Evaluator, EvaluatorConfig, PitStatistics},
    net::{ConvResNet, ConvResNetConfig},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct LossStats {
    pub epoch: Vec<usize>,
    pub loss: Vec<f64>,
    pub policy_loss: Vec<f64>,
    pub value_loss: Vec<f64>,
    pub was_best_change: Vec<bool>,
    pub fight_statistics: Vec<PitStatistics>,
}

impl LossStats {
    pub fn new() -> Self {
        Self {
            epoch: vec![],
            loss: vec![],
            policy_loss: vec![],
            value_loss: vec![],
            was_best_change: vec![],
            fight_statistics: vec![],
        }
    }

    pub fn push(&mut self, epoch: usize, loss: f64, value_loss: f64, policy_loss: f64) {
        self.epoch.push(epoch);
        self.loss.push(loss);
        self.policy_loss.push(policy_loss);
        self.value_loss.push(value_loss);
    }

    pub fn push_fight(&mut self, was_best_change: bool, fight_statistics: PitStatistics) {
        self.was_best_change.push(was_best_change);
        self.fight_statistics.push(fight_statistics);
    }

    pub fn get_filename(&self) -> String {
        let now = chrono::offset::Local::now();
        let datetime = now.format("%Y%m%y_%H%M%S");
        format!("loss_stats_{}.json", datetime)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let content = serde_json::to_string_pretty(&self)
            .expect("Serde must serialize loss stats with no problem");
        let dir = PathBuf::from("./loss_stats");
        let filename = self.get_filename();
        let path = dir.join(filename);
        fs::create_dir_all(dir)?;
        fs::write(path, content)?;

        Ok(())
    }
}

pub struct SelfPlayData {
    pub pi: Tensor,
    pub z: Tensor,
    pub state: Tensor,
    // Depending on the player color, the reward will be determined
    pub player_color: PlayerColor,
}

pub fn self_play(
    mcts: Arc<AlphaZeroMcts>,
    options: Options,
    deck: Option<Deck>,
    data_buffer: Arc<Mutex<Vec<SelfPlayData>>>,
) {
    let mut state = if let Some(deck) = deck {
        State::with_deck(deck)
    } else {
        State::new()
    };
    let mut player_color = state.deck.neutral_card().player_color;
    let mut progress = MoveResult::InProgress;

    let mut max_plies = 150;

    let mut play_data = vec![];
    while !progress.is_win() {
        let (mov, priors) = mcts.generate_move_tensor(&state, player_color);

        let state_tensor = create_tensor_from_state(&state, player_color, options.to_tuple());

        progress = state.make_move(&mov.mov, player_color, mov.used_card_idx);

        play_data.push(SelfPlayData {
            // Priors are the size of [25]
            pi: priors,
            // Value is one number size [1]
            z: Tensor::from(0.),
            // Size is [L, 5, 5]. Later stack will create [B, L, 5, 5] where B is a batch size and L block layer size
            state: state_tensor,
            player_color,
        });

        player_color.switch();

        if max_plies < 0 {
            println!("Game has become infinite!");
            break;
        }

        max_plies -= 1;
    }

    // assign rewards to the positions depending on a player color
    play_data
        .iter_mut()
        .for_each(|s| s.z = Tensor::from(AlphaZeroMcts::reward(progress, s.player_color)));

    // Append play data to the data_buffer
    data_buffer.lock().unwrap().extend(play_data.into_iter());
}

pub fn train(epochs: usize) -> anyhow::Result<()> {
    let device = Device::cuda_if_available();
    // let device = Device::Cpu;
    println!("[*] Is CUDA available? {:?}", Device::is_cuda(device));

    let options = Options::new(kind::FLOAT_CUDA);

    let net_config = ConvResNetConfig {
        hidden_channels: 32,
        input_channels: 21,
        resnet_block_amnt: 3,
    };
    let vs = nn::VarStore::new(device);
    let model = Arc::new(Mutex::new(ConvResNet::new(
        &vs.root(),
        net_config.clone(),
        options,
    )));
    let mut best_vs = nn::VarStore::new(device);
    if let Err(e) = best_vs.copy(&vs) {
        eprintln!("Was not able to copy varstore {}", e);
    }
    let mcts = Arc::new(AlphaZeroMcts {
        config: AlphaZeroMctsConfig {
            max_playouts: 400,
            search_time: Duration::from_millis(400),
            train: true,
            ..Default::default()
        },
        model: model.clone(),
        options,
    });

    // TODO: playing with the stable deck at the moment
    let deck = Some(Deck::new([
        ORIGINAL_CARDS[0].clone(),
        ORIGINAL_CARDS[1].clone(),
        ORIGINAL_CARDS[2].clone(),
        ORIGINAL_CARDS[3].clone(),
        ORIGINAL_CARDS[4].clone(),
    ]));
    let evaluator_config = EvaluatorConfig {
        deck: deck.clone(),
        ..Default::default()
    };

    let learning_rate = 1e-2;
    let save_checkpoint = 5;
    let evaluation_checkpoint = 10;

    let mut opt = nn::Sgd {
        momentum: 0.9,
        ..Default::default()
    }
    .build(&vs, learning_rate)?;
    opt.set_weight_decay(1e-4);

    let self_play_batch_size = 1;
    let train_batch_size = 256;
    let thread_amnt = std::thread::available_parallelism().unwrap().get() * 2;
    println!("[*] {} threads are going to be used", thread_amnt);

    let mut loss_stats = LossStats::new();

    println!("[*] Starting self play");

    // Epochs or iterations
    for epoch in 1..epochs + 1 {
        let data_buffer: Arc<Mutex<Vec<SelfPlayData>>> = Arc::new(Mutex::new(Vec::new()));
        let start = Instant::now();

        // Self play data gathering
        for i in 0..self_play_batch_size {
            let mut handles = vec![];

            for _ in 0..thread_amnt {
                let db = Arc::clone(&data_buffer);
                let mcts = Arc::clone(&mcts);
                let deck = deck.clone();
                let handle = std::thread::spawn(move || {
                    self_play(mcts, options, deck, db);
                });
                handles.push(handle);
            }

            for handle in handles {
                handle.join().unwrap();
            }

            println!(
                "[*] Self-play batch: {:?}, Data buffer size: {}",
                i,
                data_buffer.lock().unwrap().len(),
            );
        }

        let end = start.elapsed();

        println!("[*] Epoch: {}, Self-play time: {:?}", epoch, end);

        // Train
        let data_buffer_lock = data_buffer.lock().unwrap();
        let mut avg_loss: f64 = 0.;
        let mut avg_value_loss: f64 = 0.;
        let mut avg_policy_loss: f64 = 0.;

        if data_buffer_lock.len() > train_batch_size {
            let train_amnt = data_buffer_lock.len() / train_batch_size;
            for _ in 0..train_amnt {
                let batch = data_buffer_lock
                    .iter()
                    .choose_multiple(&mut thread_rng(), train_batch_size);

                // Creating a batch of size [B, L, 5, 5] where L is a block layer size
                let state_batch = Tensor::stack(
                    &batch
                        .iter()
                        .map(|s| s.state.shallow_clone())
                        .collect::<Vec<Tensor>>(),
                    0,
                );

                // Z batch is the size of [B, 1]
                let z_batch = Tensor::stack(
                    &batch
                        .iter()
                        .map(|s| s.z.shallow_clone())
                        .collect::<Vec<Tensor>>(),
                    0,
                );

                // PI batch is the size of [B, 50]
                let pi_batch = Tensor::stack(
                    &batch
                        .iter()
                        .map(|s| s.pi.shallow_clone())
                        .collect::<Vec<Tensor>>(),
                    0,
                );

                let model_lock = model.lock().unwrap();
                let y = model_lock.forward(&state_batch, true);

                let (value, policy) =
                    model_lock.alphaloss(&y.value, &y.policy, &pi_batch, &z_batch);

                let loss = &value + &policy;

                let mean = loss.mean(options.kind);

                avg_loss += f64::from(&mean);
                avg_value_loss += f64::from(&value);
                avg_policy_loss += f64::from(&policy.mean(options.kind));

                opt.backward_step(&mean);

                println!("epoch: {:4} loss: {:5.2}", epoch, mean);
            }

            avg_loss /= train_amnt as f64;
            avg_value_loss /= train_amnt as f64;
            avg_policy_loss /= train_amnt as f64;
            loss_stats.push(epoch, avg_loss, avg_value_loss, avg_policy_loss);
        }

        // Evaluate the model against itself and other agents
        if epoch % evaluation_checkpoint == 0 {
            println!("[*] Starting evaluation phase");
            let start = Instant::now();
            let mut evaluator = Evaluator::new(
                evaluator_config.clone(),
                &best_vs,
                &vs,
                &net_config,
                options,
            );

            let (fight_statistics, should_change_best) = evaluator.pit();
            let end = start.elapsed();

            println!(
                "[*] Done evaluation in {:?}. New model winrate against the best: {}",
                end, fight_statistics.self_fight.winrate
            );
            loss_stats.push_fight(should_change_best, fight_statistics);

            if should_change_best {
                println!("[*] New model is better. Changing..");
                if let Err(e) = best_vs.copy(&vs) {
                    eprintln!("Was not able to copy best varstore {}", e);
                }

                println!("[*] Saving best model...");
                if let Err(err) = vs.save(format!(
                    "models/best_model_{}.ot",
                    Local::now().format("%Y%m%d_%H%M%S")
                )) {
                    eprintln!("error while saving model: {}", err);
                }
            } else {
                println!("[*] New model is not the best one. Continue training..");
            }
            println!("{:?}", loss_stats);
        }

        // Checkpoint save
        if epoch % save_checkpoint == 0 {
            if let Err(err) = vs.save(format!(
                "models/model_{}.ot",
                Local::now().format("%Y%m%d_%H%M%S")
            )) {
                eprintln!("error while saving model: {}", err);
            } else {
                println!("Saved the model");
                println!("Loss stats: {:?}", loss_stats);
                if let Err(e) = loss_stats.save() {
                    eprintln!("Error while saving loss stats: {}", e);
                };
            }
        }
    }

    Ok(())
}
