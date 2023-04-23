use std::{fs, io, path::PathBuf};

use onitama_game::game::deck::Deck;
use serde::Serialize;

use crate::player::PlayerType;

#[derive(Serialize)]
pub struct TournamentResult {
    // First and a second player winrate
    // Their sum must be the same as round amount
    pub wins: [u32; 2],
}

impl Default for TournamentResult {
    fn default() -> Self {
        Self { wins: [0; 2] }
    }
}

#[derive(Serialize)]
pub struct RoundResult {
    pub round: u32,
    pub move_amnt: usize,
    pub winning_player: PlayerType,
}

#[derive(Serialize)]
pub struct Tournament {
    pub round_amnt: u32,
    pub curr_round: u32,
    pub save_games: bool,
    pub do_player_swap: bool,
    pub is_tournament_on: bool,
    pub random_deck_each_turn: bool,
    // TODO: identify player by hash
    pub players: [PlayerType; 2],
    pub result: TournamentResult,
    pub round_result_history: Vec<RoundResult>,
    pub deck: Deck,
    pub deck_history: Vec<Deck>,
}

impl Default for Tournament {
    fn default() -> Self {
        let deck = Deck::default();
        let deck_history = vec![deck.clone()];
        Self {
            round_amnt: 10,
            curr_round: 1,
            save_games: false,
            is_tournament_on: false,
            do_player_swap: true,
            random_deck_each_turn: true,
            players: [PlayerType::Human, PlayerType::Mcts],
            deck,
            result: TournamentResult::default(),
            deck_history,
            round_result_history: Vec::new(),
        }
    }
}

impl Tournament {
    pub fn progress(&mut self, move_amnt: usize, winning_player: PlayerType) {
        if self.random_deck_each_turn {
            self.deck = Deck::default();
            self.deck_history.push(self.deck.clone());
        }
        self.round_result_history.push(RoundResult {
            round: self.curr_round,
            move_amnt,
            winning_player,
        });
        let index = self
            .players
            .iter()
            .position(|&p| p == winning_player)
            .unwrap();
        self.result.wins[index] += 1;
        self.curr_round += 1;
    }

    pub fn get_filename(&self) -> String {
        let now = chrono::offset::Local::now();
        let datetime = now.format("%Y%m%y_%H%M%S");
        format!(
            "tour_{}_vs_{}_{}.json",
            self.players[0].to_string().to_lowercase().replace(" ", "_"),
            self.players[1].to_string().to_lowercase().replace(" ", "_"),
            datetime
        )
    }

    pub fn save_to_folder(&self, folder: &String) -> io::Result<()> {
        let path = PathBuf::from(format!("{}/{}", folder, self.get_filename()));
        fs::write(
            path,
            serde_json::to_string_pretty(&self)
                .expect("Serde must serialize move history with no problem"),
        )?;
        Ok(())
    }

    pub fn clear(&mut self) {
        self.curr_round = 1;
        self.result = TournamentResult::default();
        self.deck_history = vec![];
        self.round_result_history = vec![];
    }
}
