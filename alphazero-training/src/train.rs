use std::{
    fs,
    time::{Duration, Instant},
};

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
use tracing::{error, info, warn};

use crate::{
    alphazero_mcts::{reward, AlphaZeroMctsConfig, TrainingAlphaZeroMcts},
    common::{create_tensor_from_state, Options},
    elo_rating::PlayerRating,
    evaluator::{Evaluator, EvaluatorConfig, PitStatistics},
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
                warn!("[@] Game has become infinite!");
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

    info!(
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
    pub deck: Option<Deck>,
    pub evaluator_config: EvaluatorConfig,
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
            deck: None,
            evaluator_config: EvaluatorConfig::default(),
        }
    }
}

// TODO: Need to properly split this function up
// Otherwise it becomes quite a mess
pub fn train(config: TrainConfig) -> anyhow::Result<()> {
    let date = Local::now().format("%Y%m%d_%H%M%S");
    let folder = format!("models/{}", date);
    fs::create_dir_all(folder.clone()).unwrap();

    // let device = Device::cuda_if_available();
    let device = Device::Cpu;
    info!("[*] Is CUDA available? {:?}", Device::is_cuda(device));
    let options = if Device::is_cuda(device) {
        Options::new(kind::FLOAT_CUDA)
    } else {
        Options::new(kind::FLOAT_CPU)
    };

    let vs = nn::VarStore::new(device);
    let training_model = ConvResNet::new(&vs.root(), config.model_config.clone(), options);
    let mut best_vs = nn::VarStore::new(device);
    if let Err(e) = best_vs.copy(&vs) {
        error!("[!] Was not able to copy varstore {}", e);
    }

    let mut opt = nn::Sgd {
        momentum: 0.9,
        ..Default::default()
    }
    .build(&vs, config.learning_rate)?;
    opt.set_weight_decay(config.l2_const);

    info!("[*] {} threads are going to be used", config.thread_amnt);

    let mut loss_stats = Stats::new();
    use crate::elo_rating::FightPlayer::*;
    // TODO: very very verbose solution,
    // Need a way to create a better architecture
    let mut ratings = [
        [
            PlayerRating::new(TrainingModel),
            PlayerRating::new(BestModel),
        ],
        [PlayerRating::new(TrainingModel), PlayerRating::new(Random)],
        [PlayerRating::new(TrainingModel), PlayerRating::new(Mcts)],
        [
            PlayerRating::new(TrainingModel),
            PlayerRating::new(AlphaBeta),
        ],
    ];

    info!("[*] Starting self play");

    let mut small_rng = SmallRng::from_entropy();

    for iter in 1..config.iterations + 1 {
        let mut data_buffer = Vec::with_capacity(config.buffer_size);

        let start = Instant::now();

        // Self play data gathering
        std::thread::scope(|s| {
            let mut handles = vec![];

            for _ in 0..config.thread_amnt {
                let mut best_vs_copy = nn::VarStore::new(device);
                if let Err(e) = best_vs_copy.copy(&best_vs) {
                    error!("[!] Was not able to copy best varstore: {}", e);
                }
                let best_model =
                    ConvResNet::new(&best_vs_copy.root(), config.model_config.clone(), options);

                let mcts = TrainingAlphaZeroMcts {
                    config: config.mcts_config.clone(),
                    model: best_model,
                    options,
                };

                let handle = s.spawn(|| self_play(mcts, options, config.deck.clone(), &config));
                handles.push(handle);
            }

            for handle in handles {
                let play_data = handle.join().unwrap();
                data_buffer.extend(play_data.into_iter());
            }
        });

        info!(
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

        for epoch in 1..config.training_epochs + 1 {
            if data_buffer.len() < config.train_batch_size {
                info!(
                    "[*] Not enough data for training. Data amount: {}, expected: {}",
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

                info!(
                    "[?] Iteration: {}, epoch: {}, loss: {:5.2}",
                    iter,
                    epoch,
                    f64::from(&loss)
                );
            }

            avg_epoch_loss += avg_loss / train_amnt as f64;
            avg_epoch_value_loss += avg_value_loss / train_amnt as f64;
            avg_epoch_policy_loss += avg_policy_loss / train_amnt as f64;
        }
        let epochs = config.training_epochs as f64;
        loss_stats.push(
            iter,
            avg_epoch_loss / epochs,
            avg_epoch_value_loss / epochs,
            avg_epoch_policy_loss / epochs,
        );

        info!(
            "[*] I: {}, Average loss: {:5.4}, avg value: {:5.4}, avg policy: {:5.4}",
            iter,
            avg_epoch_loss / epochs,
            avg_epoch_value_loss / epochs,
            avg_epoch_policy_loss / epochs,
        );

        // Evaluate the model against itself and other agents
        if iter % config.evaluation_checkpoint == 0 {
            info!("[*] Starting evaluation phase");
            let start = Instant::now();

            let mut evaluator = Evaluator::new(
                config.evaluator_config.clone(),
                &best_vs,
                &vs,
                &config.model_config,
                &ratings,
                options,
            );

            let (fight_statistics, should_change_best) = evaluator.pit();
            let end = start.elapsed();

            info!(
                "[*] Done evaluation in {:?}. New model winrate against the best: {}. New model rating: {}",
                end, fight_statistics.self_fight.winrate,
                fight_statistics.self_fight.rating_a
            );

            update_ratings(&mut ratings, &fight_statistics);
            loss_stats.push_fight(should_change_best, fight_statistics);

            if should_change_best {
                info!("[*] New model is better. Changing..");
                if let Err(e) = best_vs.copy(&vs) {
                    error!("[!] Was not able to copy best varstore: {}", e);
                }

                // Change best model rating to be the same as for the training model
                ratings[0][1].update_rating(*ratings[0][0].rating);

                info!("[*] Saving best model...");
                if let Err(err) = vs.save(format!(
                    "{}/best_model_{}.ot",
                    folder,
                    Local::now().format("%Y%m%d_%H%M%S")
                )) {
                    error!("[!] error while saving model: {}", err);
                }
            } else {
                info!("[*] New model is not the best one. Continue training..");
            }
        }

        // Checkpoint save
        if iter % config.save_checkpoint == 0 {
            let path = format!(
                "{}/model_{}.ot",
                folder,
                Local::now().format("%Y%m%d_%H%M%S")
            );

            if let Err(err) = vs.save(path) {
                error!("[!] error while saving model: {}", err);
            } else {
                info!("[*] Saved the model");
                if let Err(e) = loss_stats.save() {
                    error!("[!] Error while saving loss stats: {}", e);
                };
                info!("[*] Saved statistics!");
            }
        }
    }

    Ok(())
}

fn update_ratings(ratings: &mut [[PlayerRating; 2]; 4], statistics: &PitStatistics) {
    ratings[0][0].update_rating(statistics.self_fight.rating_a);
    ratings[0][1].update_rating(statistics.self_fight.rating_b);

    ratings[1][0].update_rating(statistics.random_fight.rating_a);
    ratings[1][1].update_rating(statistics.random_fight.rating_b);

    ratings[2][0].update_rating(statistics.mcts_fight.rating_a);
    ratings[2][1].update_rating(statistics.mcts_fight.rating_b);

    ratings[3][0].update_rating(statistics.alphabeta_fight.rating_a);
    ratings[3][1].update_rating(statistics.alphabeta_fight.rating_b);
}
