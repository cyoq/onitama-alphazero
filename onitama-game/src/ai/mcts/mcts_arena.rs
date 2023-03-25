use std::time::{Duration, Instant};

use rand::{thread_rng, Rng};

use crate::game::{
    card::{self, CARD_NAMES},
    deck::Deck,
    done_move::DoneMove,
    move_result::MoveResult,
    player_color::PlayerColor,
    r#move::Move,
    state::State,
};

pub struct MctsArena {
    pub state: State,
    pub search_time: Duration,
    pub arena: Vec<MctsNode>,
    pub min_node_visits: u32,
    pub exploration_c: f32,
    pub playouts: u32,
}

impl MctsArena {
    pub fn new(
        state: State,
        search_time: Duration,
        player_color: PlayerColor,
        min_node_visits: u32,
        exploration_c: f32,
    ) -> Self {
        // root is always first in the arena
        let root = MctsNode::new(None, 0, None, player_color);
        let arena = vec![root];

        Self {
            state,
            search_time,
            arena,
            min_node_visits,
            exploration_c,
            playouts: 0,
        }
    }

    /// Seach the best move
    /// 1. Make the playouts until time is up
    /// 2. Select the best node which was visited the most
    pub fn search(&mut self) -> DoneMove {
        let now = Instant::now();

        while now.elapsed() < self.search_time {
            self.playout();
            self.playouts += 1;
        }

        let children = &self.arena[0].children;

        let best_child_idx = children
            .iter()
            .max_by_key(|&c| self.arena[*c].visits)
            .expect("Must find the best child");

        self.arena[*best_child_idx]
            .mov
            .expect("A child node must have a move")
    }

    /// Make a playout to find the best node
    pub fn playout(&mut self) {
        let mut state = self.state.clone();
        let mut node_idx = 0; // root node

        // 1. Select the best node and make a move of it
        while self.arena[node_idx].is_expanded && !self.arena[node_idx].is_terminal {
            node_idx = self.select(&self.arena[node_idx]);

            if let Some(mov) = self.arena[node_idx].mov {
                let move_result = state.make_move(
                    &mov.mov,
                    self.arena[node_idx].player_color,
                    mov.used_card_idx,
                );
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
            self.expand(node_idx, &state);
        }

        // 3. Simulate the game till the end
        let reward = self.simulate(state, node_idx);

        // 4. Back propagate the result back to the root
        self.back_propagate(node_idx, reward);
    }

    /// Selects the best or random child
    /// 1. Get children and visits count
    /// 2. Check if parent was visited enough times to be able to give the best child
    ///     It is also necessary, since to calculate UCT, we need to gather some data
    /// 3. Select the best child using UCT score by returning index of it
    fn select(&self, parent: &MctsNode) -> usize {
        let children = &parent.children;

        if parent.visits < self.min_node_visits {
            let mut rng = thread_rng();
            let child_idx = rng.gen_range(0..children.len());
            return child_idx;
        }

        let uct = |child: &MctsNode| {
            child.winrate
                + self.exploration_c * (parent.visits as f32 / child.visits as f32).ln().sqrt()
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
    /// 1. Iterate over cards
    /// 2. Get allowed moves per card
    /// 3. Create a new child with the move
    /// 4. Add child to the arena
    /// 5. Add child references to the parent
    /// 6. Set parent to be expanded node
    pub fn expand(&mut self, parent: usize, cloned_state: &State) {
        let player_color = self.arena[parent].player_color;
        let cards = cloned_state.deck.get_player_cards_idx(player_color);

        for card_idx in cards {
            let allowed_moves = cloned_state.generate_legal_moves_card_idx(player_color, card_idx);

            for mov in allowed_moves.iter() {
                let done_move = DoneMove {
                    mov: *mov,
                    used_card_idx: card_idx,
                };
                let idx = self.size();
                let child = MctsNode::new(Some(parent), idx, Some(done_move), player_color.enemy());

                self.arena.push(child);
                self.arena[parent].children.push(idx);
            }
        }
        self.arena[parent].is_expanded = true;
    }

    /// Simulate the game with random
    pub fn simulate(&self, mut state: State, node: usize) -> i32 {
        let mut move_result = MoveResult::InProgress;

        let mut reward = 0;
        let mut player_color = self.arena[node].player_color;
        let mut rng = thread_rng();

        while !move_result.is_win() {
            let moves = state.generate_all_legal_moves(player_color);
            // There is a possibility that randomly chosen card
            // will not have legal moves.
            // We should fall back to the second card
            if moves.len() == 0 {
                // If no move at all, pass the turn with random card
                println!("{}", state.display());
                println!("{:?}", player_color);
                println!("{:?}", state);
                println!("Moves {:?}", moves);
                println!("Move result: {:?}", move_result);
                let card_idx = match player_color {
                    PlayerColor::Red => rng.gen_range(0..2),
                    PlayerColor::Blue => rng.gen_range(2..4),
                };
                state.pass(card_idx);

                player_color.switch();
                reward = self.reward(move_result, player_color);
                continue;
            }
            let mov = moves[rng.gen_range(0..moves.len())];

            move_result = state.make_move(&mov.1, player_color, mov.0);
            player_color.switch();
            reward = self.reward(move_result, player_color);
        }

        if move_result.is_win() {
            reward = self.reward(move_result, player_color);
        }

        reward
    }

    pub fn reward(&self, move_result: MoveResult, player_color: PlayerColor) -> i32 {
        match (player_color, move_result) {
            (PlayerColor::Red, MoveResult::RedWin) => 1,
            (PlayerColor::Red, MoveResult::BlueWin) => -1,
            (PlayerColor::Blue, MoveResult::RedWin) => -1,
            (PlayerColor::Blue, MoveResult::BlueWin) => 1,
            _ => -100,
        }
    }

    pub fn back_propagate(&mut self, node_idx: usize, mut reward: i32) {
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
            node.to_string(&self.state.deck)
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
    pub reward: i32,
    pub winrate: f32,
    pub is_terminal: bool,
    pub is_expanded: bool,
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
            reward: 0,
            winrate: 0.,
            is_terminal: false,
            is_expanded: false,
            player_color,
        }
    }

    pub fn update(&mut self, reward: i32) {
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
    use crate::game::{
        card::{DRAGON, FROG, HORSE, ORIGINAL_CARDS, RABBIT, TIGER},
        piece::PieceKind,
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
        MctsArena::new(state, search_time, PlayerColor::Red, 5, 2f32.sqrt())
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
        let state = arena.state.clone();

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
        let state = arena.state.clone();

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
    fn test_search() {
        // Theoritically using the same time, we should get the same results
        let mut arena = arena();
        let mov = arena.search();
        let expected = DoneMove {
            mov: Move {
                from: 21,
                to: 18,
                piece: PieceKind::Pawn,
            },
            used_card_idx: 0,
        };
        assert_eq!(mov, expected);
    }
}
