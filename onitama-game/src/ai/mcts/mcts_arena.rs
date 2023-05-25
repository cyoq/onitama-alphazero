use std::time::{Duration, Instant};

use rand::{rngs::SmallRng, Rng, SeedableRng};

use crate::game::{
    card::CARD_NAMES, deck::Deck, done_move::DoneMove, move_result::MoveResult,
    player_color::PlayerColor, r#move::Move, state::State,
};

#[derive(Clone)]
pub struct MctsState {
    pub state: State,
    pub player_color: PlayerColor,
}

pub struct MctsArena {
    pub game_state: MctsState,
    pub search_time: Duration,
    pub arena: Vec<MctsNode>,
    pub min_node_visits: u32,
    pub exploration_c: f32,
    pub playouts: u32,
    pub max_playouts: u32,
}

impl MctsArena {
    pub fn new(
        state: State,
        search_time: Duration,
        player_color: PlayerColor,
        min_node_visits: u32,
        exploration_c: f32,
        max_playouts: u32,
    ) -> Self {
        // root is always first in the arena
        let root = MctsNode::new(None, 0, None, player_color);
        let arena = vec![root];

        Self {
            game_state: MctsState {
                state,
                player_color,
            },
            search_time,
            arena,
            min_node_visits,
            exploration_c,
            playouts: 0,
            max_playouts,
        }
    }

    /// Seach the best move
    /// 1. Make the playouts until time is up
    /// 2. Select the best node which was visited the most
    pub fn search(&mut self) -> (DoneMove, f64) {
        let now = Instant::now();

        while now.elapsed() < self.search_time && self.playouts < self.max_playouts {
            self.playout();
            self.playouts += 1;
        }

        let children = &self.arena[0].children;

        // Did not see any major difference between number of visits
        // and the winrate
        let best_child_idx = children
            .iter()
            .max_by_key(|&c| self.arena[*c].visits)
            .expect("Must find the best child");

        // let best_child_idx = children
        //     .iter()
        //     .max_by(|&a, &b| self.arena[*a].winrate.total_cmp(&self.arena[*b].winrate))
        //     .expect("Must find the best child");

        (
            self.arena[*best_child_idx]
                .mov
                .expect("A child node must have a move"),
            self.arena[*best_child_idx].winrate as f64,
        )
    }

    /// Make a playout to find the best node
    pub fn playout(&mut self) {
        let mut game_state = self.game_state.clone();
        let mut node_idx = 0; // root node

        // 1. Select the best node and make a move with it
        while self.arena[node_idx].is_expanded && !self.arena[node_idx].is_terminal {
            node_idx = self.select(&self.arena[node_idx]);

            if let Some(mov) = self.arena[node_idx].mov {
                // during a move phase we must not see a root node
                // therefore unwrap is appropriate
                let parent = self.arena[node_idx].parent.unwrap();

                let move_result = game_state.state.make_move(
                    &mov.mov,
                    // a move makes a parent color
                    self.arena[parent].player_color,
                    mov.used_card_idx,
                );

                game_state.player_color.switch();

                if move_result.is_win() {
                    self.arena[node_idx].is_terminal = true;
                }
            }
        }

        // 2. Expand if node is capable of it
        if !self.arena[node_idx].is_expanded
            && !self.arena[node_idx].is_terminal
            && self.arena[node_idx].visits > self.min_node_visits
        {
            self.expand(node_idx, &game_state);
        }

        // 3. Simulate the game till the end
        // Reward color is dependent on the parent color, because
        // the child is the next move from the parent
        let parent = self.arena[node_idx].parent.unwrap_or(0);
        let reward = self.simulate(game_state, self.arena[parent].player_color);

        // 4. Back propagate the result back to the root
        self.back_propagate(node_idx, reward);
    }

    /// Selects the best child by UCT score
    /// 1. Get parent's children
    /// 2. Select the best child using UCT score
    /// 3. Return the best child's index
    fn select(&self, parent: &MctsNode) -> usize {
        let children = &parent.children;

        let uct = |child: &MctsNode| {
            child.winrate
                + self.exploration_c * ((parent.visits as f32).ln() / child.visits as f32).sqrt()
        };

        // Need to use max by, since it is not possible to compare floats
        // with max_by_key
        let best_child_idx = children
            .iter()
            .max_by(|&a, &b| {
                let child_a = &self.arena[*a];
                let child_b = &self.arena[*b];
                let uct_a = uct(child_a);
                let uct_b = uct(child_b);
                uct_a.total_cmp(&uct_b)
            })
            .expect("Must find the best child");

        *best_child_idx
    }

    /// Expands the node for the new children
    /// 1. Iterate over all allowed moves
    /// 2. Create a new child with one legal move
    /// 3. Add the child to the arena
    /// 4. Add the child reference to the parent
    /// 5. Set parent to be expanded node
    pub fn expand(&mut self, parent: usize, cloned_state: &MctsState) {
        // Player color is switched, because next layer will represent
        // the enemy made moves
        // let player_color = cloned_state.player_color;
        let player_color = self.arena[parent].player_color;

        let allowed_moves = cloned_state.state.generate_all_legal_moves(player_color);

        for mov in allowed_moves.iter() {
            let done_move = DoneMove {
                mov: mov.1,
                used_card_idx: mov.0,
            };
            let idx = self.size();
            let child = MctsNode::new(Some(parent), idx, Some(done_move), player_color.enemy());

            self.arena.push(child);
            self.arena[parent].children.push(idx);
        }
        self.arena[parent].is_expanded = true;
    }

    /// Simulate the game using random moves
    pub fn simulate(&self, mut mcts_state: MctsState, reward_color: PlayerColor) -> f32 {
        let mut move_result = mcts_state.state.current_state();

        if move_result.is_win() {
            return self.reward(move_result, reward_color);
        }

        let mut rng = SmallRng::from_entropy();

        // let mut capture_reward = 0.;

        while !move_result.is_win() {
            // generate legal moves for the enemy
            let moves = mcts_state
                .state
                .generate_all_legal_moves(mcts_state.player_color);

            // There is a possibility that there is no
            // legal moves at all.
            // For this case rules apply that we should choose
            // a card to swap with neutral and skip the turn
            if moves.len() == 0 {
                // If no move at all, skip the turn with random card
                let card_idx = match mcts_state.player_color {
                    PlayerColor::Red => rng.gen_range(0..2),
                    PlayerColor::Blue => rng.gen_range(2..4),
                };
                mcts_state.state.pass(card_idx);

                mcts_state.player_color.switch();
                continue;
            }

            let (used_card_idx, mov) = moves[rng.gen_range(0..moves.len())];

            move_result = mcts_state
                .state
                .make_move(&mov, mcts_state.player_color, used_card_idx);

            // Added additional heuristics which most likely helps the search
            // if move_result == MoveResult::Capture && mcts_state.player_color == reward_color {
            //     capture_reward -= 0.25;
            // } else if move_result == MoveResult::Capture && mcts_state.player_color != reward_color
            // {
            //     capture_reward += 0.25;
            // }

            mcts_state.player_color.switch();
        }

        self.reward(move_result, reward_color) // + capture_reward
    }

    pub fn reward(&self, move_result: MoveResult, reward_color: PlayerColor) -> f32 {
        match (reward_color, move_result) {
            (PlayerColor::Red, MoveResult::RedWin) => 1.,
            (PlayerColor::Red, MoveResult::BlueWin) => -1.,
            (PlayerColor::Blue, MoveResult::RedWin) => -1.,
            (PlayerColor::Blue, MoveResult::BlueWin) => 1.,
            _ => 0.,
        }
    }

    pub fn back_propagate(&mut self, node_idx: usize, mut reward: f32) {
        let mut node = &mut self.arena[node_idx];
        loop {
            node.update(reward);
            if let Some(parent) = node.parent {
                node = &mut self.arena[parent];
                reward = -reward;
            } else {
                break;
            }
        }
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.arena.len()
    }

    pub fn debug_tree(&self) -> String {
        let mut result = String::new();
        self.debug_tree_recursive(&self.arena[0], &mut result, 0);
        result
    }

    fn debug_tree_recursive(&self, node: &MctsNode, mut result: &mut String, indent: usize) {
        *result += &format!(
            "{}{}\n",
            "|  ".repeat(indent),
            node.to_string(&self.game_state.state.deck)
        );

        for child in node.children.iter() {
            self.debug_tree_recursive(&self.arena[*child], &mut result, indent + 1);
        }
    }
}

#[derive(Clone)]
pub struct MctsNode {
    /// reference to the parent in the arena
    pub parent: Option<usize>,
    /// references to the children in the arena
    pub children: Vec<usize>,
    /// The index of the node in the arena
    pub idx: usize,
    /// A move which the node represents
    pub mov: Option<DoneMove>,
    /// Number of visits for the MCTS node
    pub visits: u32,
    /// Number of wins in the specific node
    pub reward: f32,
    pub winrate: f32,
    pub is_terminal: bool,
    pub is_expanded: bool,
    /// Player color who has last moved the piece
    pub player_color: PlayerColor,
}

impl MctsNode {
    pub fn new(
        parent: Option<usize>,
        idx: usize,
        mov: Option<DoneMove>,
        player_color: PlayerColor,
    ) -> Self {
        Self {
            parent,
            children: vec![],
            idx,
            mov,
            visits: 0,
            reward: 0.,
            winrate: 0.,
            is_terminal: false,
            is_expanded: false,
            player_color,
        }
    }

    pub fn update(&mut self, reward: f32) {
        self.visits += 1;
        self.reward += reward;
        self.winrate = self.reward as f32 / self.visits as f32;
    }

    pub fn to_string(&self, deck: &Deck) -> String {
        let mut mov = "None".to_string();

        if let Some(dmov) = self.mov {
            let card = deck.get_card(dmov.used_card_idx);
            let card_name = CARD_NAMES[card.index];
            let from = Move::convert_idx_to_notation(dmov.mov.from);
            let to = Move::convert_idx_to_notation(dmov.mov.to);
            mov = format!("{} {}-{}", card_name, from, to);
        }

        format!(
            "[M: {}, C: {}, Q: {}, W: {}, N: {}, T: {}, E: {}]",
            mov,
            self.player_color.to_string(),
            self.winrate,
            self.reward,
            self.visits,
            self.is_terminal,
            self.is_expanded
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        common::from_2d_to_bitboard,
        game::{
            card::{
                DRAGON, FROG, GOOSE, HORSE, MANTIS, MONKEY, ORIGINAL_CARDS, OX, RABBIT, ROOSTER,
                TIGER,
            },
            piece::PieceKind,
        },
    };

    use super::*;
    fn deck() -> Deck {
        Deck::new([
            ORIGINAL_CARDS[DRAGON.index].clone(),
            ORIGINAL_CARDS[FROG.index].clone(),
            ORIGINAL_CARDS[TIGER.index].clone(),
            ORIGINAL_CARDS[RABBIT.index].clone(),
            ORIGINAL_CARDS[HORSE.index].clone(),
        ])
    }

    fn arena() -> MctsArena {
        let deck = deck();
        let search_time = Duration::from_secs(1);
        let state = State::with_deck(deck);
        MctsArena::new(state, search_time, PlayerColor::Red, 5, 2f32.sqrt(), 5000)
    }

    #[test]
    fn test_root_debug_print() {
        let arena = arena();

        let expected = "[M: None, C: Red, Q: 0, W: 0, N: 0, T: false, E: false]\n";
        let result = arena.debug_tree();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_first_expand_debug_print() {
        let mut arena = arena();
        let state = arena.game_state.clone();

        let expected = "
[M: None, C: Red, Q: 0, W: 0, N: 0, T: false, E: true]
|  [M: Dragon a1-c2, C: Blue, Q: 0, W: 0, N: 0, T: false, E: false]
|  [M: Dragon b1-d2, C: Blue, Q: 0, W: 0, N: 0, T: false, E: false]
|  [M: Dragon c1-a2, C: Blue, Q: 0, W: 0, N: 0, T: false, E: false]
|  [M: Dragon c1-e2, C: Blue, Q: 0, W: 0, N: 0, T: false, E: false]
|  [M: Dragon d1-b2, C: Blue, Q: 0, W: 0, N: 0, T: false, E: false]
|  [M: Dragon e1-c2, C: Blue, Q: 0, W: 0, N: 0, T: false, E: false]
|  [M: Frog b1-a2, C: Blue, Q: 0, W: 0, N: 0, T: false, E: false]
|  [M: Frog c1-b2, C: Blue, Q: 0, W: 0, N: 0, T: false, E: false]
|  [M: Frog d1-c2, C: Blue, Q: 0, W: 0, N: 0, T: false, E: false]
|  [M: Frog e1-d2, C: Blue, Q: 0, W: 0, N: 0, T: false, E: false]
";
        arena.expand(0, &state);
        let result = arena.debug_tree();
        assert_eq!("\n".to_string() + &result, expected);
    }

    #[test]
    fn test_root_child_expand_debug_print() {
        let mut arena = arena();
        let state = arena.game_state.clone();

        let expected = "
[M: None, C: Red, Q: 0, W: 0, N: 0, T: false, E: true]
|  [M: Dragon a1-c2, C: Blue, Q: 0, W: 0, N: 0, T: false, E: true]
|  |  [M: Tiger a5-a3, C: Red, Q: 0, W: 0, N: 0, T: false, E: false]
|  |  [M: Tiger b5-b3, C: Red, Q: 0, W: 0, N: 0, T: false, E: false]
|  |  [M: Tiger c5-c3, C: Red, Q: 0, W: 0, N: 0, T: false, E: false]
|  |  [M: Tiger d5-d3, C: Red, Q: 0, W: 0, N: 0, T: false, E: false]
|  |  [M: Tiger e5-e3, C: Red, Q: 0, W: 0, N: 0, T: false, E: false]
|  |  [M: Rabbit b5-a4, C: Red, Q: 0, W: 0, N: 0, T: false, E: false]
|  |  [M: Rabbit c5-b4, C: Red, Q: 0, W: 0, N: 0, T: false, E: false]
|  |  [M: Rabbit d5-c4, C: Red, Q: 0, W: 0, N: 0, T: false, E: false]
|  |  [M: Rabbit e5-d4, C: Red, Q: 0, W: 0, N: 0, T: false, E: false]
|  [M: Dragon b1-d2, C: Blue, Q: 0, W: 0, N: 0, T: false, E: false]
|  [M: Dragon c1-a2, C: Blue, Q: 0, W: 0, N: 0, T: false, E: false]
|  [M: Dragon c1-e2, C: Blue, Q: 0, W: 0, N: 0, T: false, E: false]
|  [M: Dragon d1-b2, C: Blue, Q: 0, W: 0, N: 0, T: false, E: false]
|  [M: Dragon e1-c2, C: Blue, Q: 0, W: 0, N: 0, T: false, E: false]
|  [M: Frog b1-a2, C: Blue, Q: 0, W: 0, N: 0, T: false, E: false]
|  [M: Frog c1-b2, C: Blue, Q: 0, W: 0, N: 0, T: false, E: false]
|  [M: Frog d1-c2, C: Blue, Q: 0, W: 0, N: 0, T: false, E: false]
|  [M: Frog e1-d2, C: Blue, Q: 0, W: 0, N: 0, T: false, E: false]
";
        arena.expand(0, &state);
        // Expanding first child of the root
        arena.expand(1, &state);
        let result = arena.debug_tree();
        assert_eq!("\n".to_string() + &result, expected);
    }

    #[test]
    fn test_best_move_win() {
        // Theoritically using the same time, we should get the same results
        // Blue must eat the Red king with the Dragon card
        let mut deck = deck();
        // swap rabbit and dragon
        deck.cards.swap(0, 3);
        let search_time = Duration::from_millis(1000);
        let mut state = State::with_deck(deck);
        state.kings[PlayerColor::Red as usize] = from_2d_to_bitboard((1, 3));

        let mut arena = MctsArena::new(state, search_time, PlayerColor::Blue, 5, 2f32.sqrt(), 5000);

        let mov = arena.search().0;
        println!("{}", arena.debug_tree());

        let expected = DoneMove {
            mov: Move {
                from: 1,
                to: 8,
                piece: PieceKind::Pawn,
            },
            used_card_idx: 3,
        };
        assert_eq!(mov, expected);
    }

    #[test]
    fn test_no_way_to_hide_for_blue() {
        // Theoritically using the same time, we should get the same results
        // Blue king is in the corner and he has only one move that does not lead
        // to the lost position - Rabbit e5-c5.
        // This test is for the MCTS sanity check that it does not choose losing move
        let deck = Deck::new([
            OX.clone(),
            MONKEY.clone(),
            RABBIT.clone(),
            HORSE.clone(),
            DRAGON.clone(),
        ]);
        let search_time = Duration::from_millis(1000);
        let mut state = State::with_deck(deck);
        state.kings[PlayerColor::Blue as usize] = from_2d_to_bitboard((0, 4));
        state.pawns[PlayerColor::Blue as usize] = 0;
        state.pawns[PlayerColor::Red as usize] =
            from_2d_to_bitboard((0, 3)) | from_2d_to_bitboard((1, 4));

        let mut arena = MctsArena::new(state, search_time, PlayerColor::Blue, 5, 2.0, 5000);

        let (mov, _eval) = arena.search();
        // println!("{}", arena.debug_tree());

        let expected = DoneMove {
            mov: Move {
                from: 4,
                to: 2,
                piece: PieceKind::King,
            },
            used_card_idx: 2,
        };
        assert_eq!(mov, expected);
    }

    #[test]
    fn test_worst_case_capture_blue() {
        // Theoritically using the same time, we should get the same results
        // One more sanity check for MCTS. It must not allow to capture Blue king.
        // This test function is more used for debugging purposes
        let deck = Deck::new([
            MONKEY.clone(),
            ROOSTER.clone(),
            GOOSE.clone(),
            MANTIS.clone(),
            HORSE.clone(),
        ]);
        let search_time = Duration::from_millis(1000);
        let mut state = State::with_deck(deck);
        state.kings[PlayerColor::Blue as usize] = from_2d_to_bitboard((1, 2));
        state.pawns[PlayerColor::Red as usize] =
            from_2d_to_bitboard((2, 3)) | from_2d_to_bitboard((3, 2));

        let mut arena = MctsArena::new(state, search_time, PlayerColor::Blue, 1, 1., 5000);

        let (mov, _eval) = arena.search();
        // println!("{}", arena.debug_tree());

        let expected = DoneMove {
            mov: Move {
                from: 7,
                to: 2,
                piece: PieceKind::King,
            },
            used_card_idx: 3,
        };
        assert_eq!(mov, expected);
    }
}
