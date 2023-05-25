use std::time::{Duration, Instant};

use chrono::Local;
use onitama_game::game::{
    deck::Deck, move_result::MoveResult, player_color::PlayerColor, state::State,
};
use rand::{rngs::SmallRng, seq::IteratorRandom, SeedableRng};
use tch::{
    kind,
    nn::{self, OptimizerConfig},
    Device, Tensor,
};

use crate::{
    alphazero_mcts::{reward, AlphaZeroMctsConfig, TrainingAlphaZeroMcts},
    common::{create_tensor_from_state, Options},
    evaluator::{Evaluator, EvaluatorConfig},
    net::{ConvResNet, ConvResNetConfig},
    stats::Stats,
};

pub struct SelfPlayData {
    pub pi: Tensor,
    pub z: Tensor,
    pub state: Tensor,
    // Depending on the player color, the reward will be determined
    pub player_color: PlayerColor,
}

pub fn self_play(
    mcts: TrainingAlphaZeroMcts,
    options: Options,
    deck: Option<Deck>,
    config: &TrainConfig,
) -> Vec<SelfPlayData> {
    let mut play_buffer = vec![];

    for _ in 0..config.self_play_game_amnt {
        let mut state = if let Some(deck) = deck.clone() {
            State::with_deck(deck)
        } else {
            State::new()
        };
        let mut player_color = state.deck.neutral_card().player_color;
        let mut progress = MoveResult::InProgress;

        let mut max_plies = config.max_plies;
        let mut play_history = vec![];

        while !progress.is_win() {
            let (mov, priors) = mcts.generate_move_tensor(&state, player_color);

            let state_tensor = create_tensor_from_state(&state, player_color, options.to_tuple());

            play_history.push(SelfPlayData {
                // Priors are the size of [25]
                pi: priors,
                // Value is one number size [1]
                z: Tensor::from(0.),
                // Size is [L, 5, 5]. Later stack will create [B, L, 5, 5] where B is a batch size and L block layer size
                state: state_tensor,
                player_color,
            });

            progress = state.make_move(&mov.mov, player_color, mov.used_card_idx);

            player_color.switch();

            if max_plies < 0 {
                println!("Game has become infinite!");
                break;
            }

            max_plies -= 1;
        }

        // assign rewards to the positions depending on a player color
        play_history
            .iter_mut()
            .for_each(|s| s.z = Tensor::from(reward(progress, s.player_color)));

        // Append play data to the data_buffer
        play_buffer.extend(play_history.into_iter());
    }

    println!(
        "[*] Self-played {} games, Play data size: {}",
        config.self_play_game_amnt,
        play_buffer.len(),
    );

    play_buffer
}

pub struct TrainConfig {
    // size for the game buffer before resizing
    pub buffer_size: usize,
    pub model_config: ConvResNetConfig,
    pub mcts_config: AlphaZeroMctsConfig,
    pub iterations: usize,
    pub training_epochs: usize,
    pub train_batch_size: usize,
    pub self_play_game_amnt: usize,
    pub l2_const: f64,
    pub learning_rate: f64,
    pub save_checkpoint: usize,
    pub evaluation_checkpoint: usize,
    pub thread_amnt: usize,
    // Maximum plies in the self play game to call it a draw
    pub max_plies: isize,
}

impl Default for TrainConfig {
    fn default() -> Self {
        let thread_amnt = std::thread::available_parallelism().unwrap().get() / 2;

        Self {
            // Depends on the average game length
            // Let's say that is 18 positions, then for around 10k games in one iter = 180 000
            // It is just needed so that Vector does not resize
            buffer_size: 180_000,
            model_config: ConvResNetConfig {
                hidden_channels: 64,
                input_channels: 21,
                resnet_block_amnt: 5,
            },
            mcts_config: AlphaZeroMctsConfig {
                search_time: Duration::from_millis(200),
                exploration_c: 2.,
                max_playouts: 400,
                train: true,
            },
            iterations: 10,
            training_epochs: 10,
            train_batch_size: 512,
            self_play_game_amnt: 100,
            l2_const: 1e-4,
            learning_rate: 1e-2,
            save_checkpoint: 5,
            evaluation_checkpoint: 3,
            thread_amnt,
            max_plies: 150,
        }
    }
}

// TODO: Need to properly split this function up
// Otherwise it becomes quite a mess
pub fn train(config: TrainConfig) -> anyhow::Result<()> {
    // let device = Device::cuda_if_available();
    let device = Device::Cpu;
    println!("[*] Is CUDA available? {:?}", Device::is_cuda(device));
    let options = if Device::is_cuda(device) {
        Options::new(kind::FLOAT_CUDA)
    } else {
        Options::new(kind::FLOAT_CPU)
    };

    let vs = nn::VarStore::new(device);
    let training_model = ConvResNet::new(&vs.root(), config.model_config.clone(), options);
    let mut best_vs = nn::VarStore::new(device);
    if let Err(e) = best_vs.copy(&vs) {
        eprintln!("Was not able to copy varstore {}", e);
    }

    // TODO: playing with the stable deck at the moment
    // let deck = Some(Deck::new([
    //     ORIGINAL_CARDS[0].clone(),
    //     ORIGINAL_CARDS[1].clone(),
    //     ORIGINAL_CARDS[2].clone(),
    //     ORIGINAL_CARDS[3].clone(),
    //     ORIGINAL_CARDS[4].clone(),
    // ]));
    let evaluator_config = EvaluatorConfig {
        // deck: deck.clone(),
        deck: None,
        ..Default::default()
    };

    let mut opt = nn::Sgd {
        momentum: 0.9,
        ..Default::default()
    }
    .build(&vs, config.learning_rate)?;
    opt.set_weight_decay(config.l2_const);

    println!("[*] {} threads are going to be used", config.thread_amnt);

    let mut loss_stats = Stats::new();

    println!("[*] Starting self play");

    let mut small_rng = SmallRng::from_entropy();

    for iter in 1..config.iterations + 1 {
        let mut data_buffer = Vec::with_capacity(config.buffer_size);

        let start = Instant::now();

        // Self play data gathering
        std::thread::scope(|s| {
            let mut handles = vec![];

            for _ in 0..config.thread_amnt {
                let best_model =
                    ConvResNet::new(&best_vs.root(), config.model_config.clone(), options);
                let mcts = TrainingAlphaZeroMcts {
                    config: config.mcts_config.clone(),
                    model: best_model,
                    options,
                };
                let handle = s.spawn(|| self_play(mcts, options, None, &config));
                handles.push(handle);
            }

            for handle in handles {
                let play_data = handle.join().unwrap();
                data_buffer.extend(play_data.into_iter());
            }
        });

        println!(
            "[*] Iteration: {}, Self-play time: {:?}, Data Buffer size: {}",
            iter,
            start.elapsed(),
            data_buffer.len()
        );

        loss_stats.push_games_played(
            config.self_play_game_amnt * config.thread_amnt,
            data_buffer.len(),
        );

        // Train
        let mut avg_epoch_loss: f64 = 0.;
        let mut avg_epoch_value_loss: f64 = 0.;
        let mut avg_epoch_policy_loss: f64 = 0.;

        for _epoch in 1..config.training_epochs + 1 {
            if data_buffer.len() < config.train_batch_size {
                println!(
                    "Not enough data for training. Data amount: {}, expected: {}",
                    data_buffer.len(),
                    config.train_batch_size
                );
                break;
            }

            let train_amnt = data_buffer.len() / config.train_batch_size;
            let mut avg_loss: f64 = 0.;
            let mut avg_value_loss: f64 = 0.;
            let mut avg_policy_loss: f64 = 0.;

            for _ in 0..train_amnt {
                let batch = data_buffer
                    .iter()
                    .choose_multiple(&mut small_rng, config.train_batch_size);

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

                let y = training_model.forward(&state_batch, true);

                let (value, policy) =
                    training_model.alphaloss(&y.value, &y.policy, &pi_batch, &z_batch);

                let loss = &value + &policy;
                value.print();
                policy.print();

                avg_loss += f64::from(&loss);
                avg_value_loss += f64::from(&value);
                avg_policy_loss += f64::from(&policy);

                opt.backward_step(&loss);

                println!("Iteration: {:4} loss: {:5.2}", iter, f64::from(&loss));
            }

            avg_epoch_loss = avg_loss / train_amnt as f64;
            avg_epoch_value_loss = avg_value_loss / train_amnt as f64;
            avg_epoch_policy_loss = avg_policy_loss / train_amnt as f64;
        }
        loss_stats.push(
            iter,
            avg_epoch_loss,
            avg_epoch_value_loss,
            avg_epoch_policy_loss,
        );

        // Evaluate the model against itself and other agents
        if iter % config.evaluation_checkpoint == 0 {
            println!("[*] Starting evaluation phase");
            let start = Instant::now();

            let mut evaluator = Evaluator::new(
                evaluator_config.clone(),
                &best_vs,
                &vs,
                &config.model_config,
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
        }

        // Checkpoint save
        if iter % config.save_checkpoint == 0 {
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
