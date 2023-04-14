use onitama_game::game::deck::Deck;
use serde::Serialize;

#[derive(Serialize)]
pub struct TournamentResult {
    // First and a second player winrate
    // Their sum must be the same as round amount
    pub win_1: u32,
    pub win_2: u32,
}

impl Default for TournamentResult {
    fn default() -> Self {
        Self { win_1: 0, win_2: 0 }
    }
}

#[derive(Serialize)]
pub struct RoundResult {
    pub round: u32,
    pub move_amnt: u32,
}

#[derive(Serialize)]
pub struct Tournament {
    pub round_amnt: u32,
    pub curr_round: u32,
    pub save_games: bool,
    pub do_player_swap: bool,
    pub is_tournament_on: bool,
    pub random_deck_each_turn: bool,
    pub deck: Deck,
    pub result: TournamentResult,
    pub deck_history: Vec<Deck>,
    pub round_result_history: Vec<RoundResult>,
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
            deck,
            result: TournamentResult::default(),
            deck_history,
            round_result_history: Vec::new(),
        }
    }
}

impl Tournament {
    pub fn progress(&mut self) {
        if self.random_deck_each_turn {
            self.deck = Deck::default();
            self.deck_history.push(self.deck.clone());
        }
        self.curr_round += 1;
    }

    pub fn clear(&mut self) {
        self.is_tournament_on = false;
        self.curr_round = 1;
        self.result = TournamentResult::default();
    }
}
