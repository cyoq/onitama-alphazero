use std::ops::Deref;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum FightPlayer {
    TrainingModel,
    BestModel,
    Random,
    AlphaBeta,
    Mcts,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Rating(f64);

impl Default for Rating {
    fn default() -> Self {
        Self(800.)
    }
}

impl Deref for Rating {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerRating {
    pub player: FightPlayer,
    pub rating: Rating,
}

impl PlayerRating {
    pub fn new(player: FightPlayer) -> Self {
        Self {
            player,
            rating: Default::default(),
        }
    }

    pub fn update_rating(&mut self, new_rating: f64) {
        self.rating.0 = new_rating;
    }
}

pub struct EloRating;

// Maximum value for the change of Elo after one game
const K: f64 = 32.;
// 1 / 400
const C_ELO: f64 = 2.5e-3;

impl EloRating {
    pub fn elo_change(ra: f64, rb: f64, is_a_win: bool) -> (f64, f64) {
        // Expectations based on ratings
        let ea = 1. / (1. + 10.0f64.powf(C_ELO * (rb - ra)));
        let eb = 1. / (1. + 10.0f64.powf(C_ELO * (ra - rb)));

        // Game scores
        let sa = if is_a_win { 1. } else { 0. };
        let sb = 1. - sa;

        let new_ra = ra + K * (sa - ea);
        let new_rb = rb + K * (sb - eb);
        (new_ra, new_rb)
    }
}
