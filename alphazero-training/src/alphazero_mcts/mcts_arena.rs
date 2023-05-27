use std::time::Instant;

use onitama_game::game::{
    card::CARD_NAMES, deck::Deck, done_move::DoneMove, move_result::MoveResult,
    player_color::PlayerColor, r#move::Move, state::State,
};

use rand_distr::{Dirichlet, Distribution};
use tch::{IndexOp, Tensor};

use crate::{
    common::{create_tensor_from_state, Options},
    net::{ConvResNet, ResTowerTensor},
};

use super::AlphaZeroMctsConfig;

#[derive(Debug)]
pub struct MoveEvaluation {
    pub mov: usize,
    pub probability: f64,
}

#[derive(Clone)]
pub struct MctsState {
    pub state: State,
    pub player_color: PlayerColor,
}

pub struct EvaluationResult {
    pub legal_moves: Vec<(usize, Move)>,
    pub state_tensor: Tensor,
    pub value: f64,
    pub priors: Vec<Vec<f64>>,
}

pub struct MctsArena<'a> {
    pub game_state: MctsState,
    pub config: AlphaZeroMctsConfig,
    pub arena: Vec<MctsNode>,
    pub playouts: u32,
    pub model: &'a ConvResNet,
    pub options: Options,
    pub reward: fn(MoveResult, PlayerColor) -> f64,
}

impl<'a> MctsArena<'a> {
    pub fn new(
        state: State,
        player_color: PlayerColor,
        config: AlphaZeroMctsConfig,
        model: &'a ConvResNet,
        options: Options,
        reward: fn(MoveResult, PlayerColor) -> f64,
    ) -> Self {
        // root is always first in the arena
        let root = MctsNode::new(None, 0, None, player_color, 1.);
        let arena = vec![root];

        Self {
            game_state: MctsState {
                state,
                player_color,
            },
            config,
            arena,
            playouts: 0,
            model,
            options,
            reward,
        }
    }

    /// Seach the best move
    pub fn search(&mut self) -> (DoneMove, Tensor) {
        let now = Instant::now();

        while self.playouts < self.config.max_playouts && now.elapsed() < self.config.search_time {
            self.playout();
            self.playouts += 1;
        }

        let children = &self.arena[0].children;

        let priors = self.calculate_priors(children);

        let best_child_idx = children
            .iter()
            .max_by(|&a, &b| {
                let a = self.arena[*a].visits as f64 / self.arena[0].visits as f64;
                let b = self.arena[*b].visits as f64 / self.arena[0].visits as f64;
                a.total_cmp(&b)
            })
            .expect("Must find the best child");

        (
            self.arena[*best_child_idx]
                .mov
                .expect("A child node must have a move"),
            priors,
        )
    }

    fn calculate_priors(&self, children: &Vec<usize>) -> Tensor {
        let mut priors = Tensor::zeros(&[2, 25], self.options.to_tuple());

        for child_idx in children.iter() {
            let child = &self.arena[*child_idx];

            let idx = match child.mov.unwrap().used_card_idx {
                0 | 2 => 0,
                1 | 3 => 1,
                _ => panic!("Incorrect card index"),
            };
            *(&mut priors.i((idx, child.mov.unwrap().mov.to as i64))) += child.visits as i64;
        }
        let sum = priors.sum(self.options.kind);
        // prevent division by zero
        if f64::from(&sum) > 0. {
            priors /= sum;
        }

        priors
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

        // 2. Evaluate position using the neural network
        let evaluation_result = self.evaluate(&game_state);

        // 3. Expand if node is capable of it
        if !self.arena[node_idx].is_expanded && !self.arena[node_idx].is_terminal {
            self.expand(node_idx, &evaluation_result);
        }

        // 4. Back propagate the result back to the root
        // Reward color is dependent on the parent color, because
        // the child is the next move from the parent
        let parent = self.arena[node_idx].parent.unwrap_or(0);
        let reward_color = self.arena[parent].player_color;

        let move_result = game_state.state.current_state();

        if move_result.is_win() {
            let reward = (self.reward)(move_result, reward_color);
            self.back_propagate(node_idx, reward);
        } else {
            self.back_propagate(node_idx, evaluation_result.value);
        }
    }

    /// Selects the best child by UCT score
    /// 1. Get parent's children
    /// 2. Select the best child using UCT score
    /// 3. Return the best child's index
    fn select(&self, parent: &MctsNode) -> usize {
        let children = &parent.children;
        // Values from Silver paper
        let epsilon = 0.25;
        let eta = 0.03;
        let mut rng = rand::thread_rng();

        let mut uct = |child: &MctsNode| {
            // if it is a root node, apply Dirichlet noise
            if parent.parent == None && self.config.train {
                let action_num = children.len();
                let dirichlet = Dirichlet::new_with_size(eta, action_num).unwrap();
                let noise_vector = dirichlet.sample(&mut rng);
                // Root is at 0th index, other root children are in sequence
                let noise = noise_vector[child.idx - 1];

                child.winrate
                    + self.config.exploration_c
                        * (child.probability * (1. - epsilon) + noise * epsilon)
                        * ((parent.visits as f64).sqrt() / (child.visits + 1) as f64)
            } else {
                child.winrate
                    + self.config.exploration_c
                        * child.probability
                        * ((parent.visits as f64).sqrt() / (child.visits + 1) as f64)
            }
        };

        // Need to use max by, since it is not possible to compare floats
        // with max_by_key
        let best_child_idx = children
            .iter()
            .max_by(|&a, &b| {
                let uct_a = uct(&self.arena[*a]);
                let uct_b = uct(&self.arena[*b]);
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
    pub fn expand(&mut self, parent: usize, eval_result: &EvaluationResult) {
        // Player color is switched, because next layer will represent
        // the enemy made moves
        // let player_color = cloned_state.player_color;
        let player_color = self.arena[parent].player_color;

        for (card_idx, mov) in eval_result.legal_moves.iter() {
            let done_move = DoneMove {
                mov: *mov,
                used_card_idx: *card_idx,
            };
            let idx = self.size();
            let prob = match card_idx {
                0 | 2 => eval_result.priors[0][mov.to as usize],
                1 | 3 => eval_result.priors[1][mov.to as usize],
                _ => 1.,
            };
            let child = MctsNode::new(
                Some(parent),
                idx,
                Some(done_move),
                player_color.enemy(),
                prob,
            );

            self.arena.push(child);
            self.arena[parent].children.push(idx);
        }
        self.arena[parent].is_expanded = true;
    }

    /// Evaluates positions with the neural network
    /// 1. Create a Tensor representing the current position.
    ///     A Tensor must be the size of (1, Channels, 5, 5)
    /// 2. Feed this Tensor to the neural network and get a policy and a value
    /// 3. Return back legal actions, their probabilities and a value
    pub fn evaluate(&self, state: &MctsState) -> EvaluationResult {
        let t = create_tensor_from_state(&state.state, state.player_color, self.options.to_tuple());
        let results = tch::no_grad(|| self.model.forward(&t, false));

        // Squeeze batch dimension that should be [1], so [1, 2, 25] -> [2, 25]
        let first_card_policy = Vec::<f64>::from(results.policy.squeeze_dim(0).i((0, ..)));
        let second_card_policy = Vec::<f64>::from(results.policy.squeeze_dim(0).i((1, ..)));

        let legal_moves = state.state.generate_all_legal_moves(state.player_color);

        let mut priors = vec![vec![0f64; 25]; 2];
        for legal_move in legal_moves.iter() {
            let (card_idx, mov) = *legal_move;
            let idx = mov.to as usize;
            match card_idx {
                0 | 2 => priors[0][idx] = first_card_policy[idx],
                1 | 3 => priors[1][idx] = second_card_policy[idx],
                idx => panic!("Incorrect card index {} was somehow used", idx),
            }
        }

        let first_card_probs_sum: f64 = priors[0].iter().sum();
        let second_card_probs_sum: f64 = priors[1].iter().sum();

        if first_card_probs_sum > 0. {
            priors[0]
                .iter_mut()
                .for_each(|p| *p /= first_card_probs_sum);
        }

        if second_card_probs_sum > 0. {
            priors[1]
                .iter_mut()
                .for_each(|p| *p /= second_card_probs_sum);
        }

        EvaluationResult {
            legal_moves,
            state_tensor: t,
            // Squeezing the batch dimension [1, 1] -> [1]
            value: f64::from(results.value.squeeze_dim(0)),
            priors,
        }
    }

    pub fn back_propagate(&mut self, node_idx: usize, mut reward: f64) {
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

    pub fn evaluate_state(&self, state: &State, player_color: PlayerColor) -> ResTowerTensor {
        let t = create_tensor_from_state(&state, player_color, self.options.to_tuple());
        tch::no_grad(|| self.model.forward(&t, false))
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

// #[derive(Clone)]
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
    pub reward: f64,
    pub winrate: f64,
    pub is_terminal: bool,
    pub is_expanded: bool,
    pub player_color: PlayerColor,
    pub probability: f64,
}

impl MctsNode {
    pub fn new(
        parent: Option<usize>,
        idx: usize,
        mov: Option<DoneMove>,
        player_color: PlayerColor,
        probability: f64,
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
            probability,
        }
    }

    pub fn update(&mut self, reward: f64) {
        self.visits += 1;
        self.reward += reward;
        self.winrate = self.reward as f64 / self.visits as f64;
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
            "[M: {}, C: {}, Q: {}, W: {}, N: {}, T: {}, E: {}, P: {}]",
            mov,
            self.player_color.to_string(),
            self.winrate,
            self.reward,
            self.visits,
            self.is_terminal,
            self.is_expanded,
            self.probability
        )
    }
}
