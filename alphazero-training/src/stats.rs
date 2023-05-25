use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::evaluator::PitStatistics;

#[derive(Debug, Serialize, Deserialize)]
pub struct GamesPlayed {
    pub games_amnt: usize,
    pub positions_retrieved: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Stats {
    pub iteration: Vec<usize>,
    pub loss: Vec<f64>,
    pub policy_loss: Vec<f64>,
    pub value_loss: Vec<f64>,
    pub was_best_change: Vec<bool>,
    pub fight_statistics: Vec<PitStatistics>,
    pub games_played: Vec<GamesPlayed>,
}

impl Stats {
    pub fn new() -> Self {
        Self {
            iteration: vec![],
            loss: vec![],
            policy_loss: vec![],
            value_loss: vec![],
            was_best_change: vec![],
            fight_statistics: vec![],
            games_played: vec![],
        }
    }

    pub fn push_games_played(&mut self, games_amnt: usize, positions_amnt: usize) {
        self.games_played.push(GamesPlayed {
            games_amnt,
            positions_retrieved: positions_amnt,
        });
    }

    pub fn push(&mut self, epoch: usize, loss: f64, value_loss: f64, policy_loss: f64) {
        self.iteration.push(epoch);
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
