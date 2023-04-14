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

pub struct Tournament {
    pub round_amnt: u32,
    pub curr_round: u32,
    // A flag to save games
    pub save_games: bool,
    pub do_player_swap: bool,
    pub is_tournament_on: bool,
    pub result: TournamentResult,
}

impl Default for Tournament {
    fn default() -> Self {
        Self {
            round_amnt: 10,
            curr_round: 1,
            save_games: false,
            is_tournament_on: false,
            do_player_swap: true,
            result: TournamentResult::default(),
        }
    }
}

impl Tournament {
    pub fn clear(&mut self) {
        self.is_tournament_on = false;
        self.curr_round = 1;
        self.result = TournamentResult::default();
    }
}
