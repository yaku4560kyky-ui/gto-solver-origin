use crate::leduc::infoset;
use crate::leduc::{LeducDeck, LeducState};
use std::collections::HashMap;

const ALPHA: f64 = 1.5;
const BETA: f64 = 0.0;
const GAMMA: f64 = 2.0;
const MAX_ACTIONS: usize = 3;

#[derive(Debug, Clone)]
pub struct DCFRNode {
    pub regret_sum: [f64; MAX_ACTIONS],
    pub strategy_sum: [f64; MAX_ACTIONS],
}

impl DCFRNode {
    pub fn new() -> Self {
        Self {
            regret_sum: [0.0; MAX_ACTIONS],
            strategy_sum: [0.0; MAX_ACTIONS],
        }
    }

    pub fn get_strategy(&self, action_count: usize) -> [f64; MAX_ACTIONS] {
        let mut strategy = [0.0; MAX_ACTIONS];
        let positive_sum: f64 = self.regret_sum[..action_count]
            .iter()
            .map(|regret| regret.max(0.0))
            .sum();
        if positive_sum > 0.0 {
            for (i, value) in strategy.iter_mut().enumerate().take(action_count) {
                *value = self.regret_sum[i].max(0.0) / positive_sum;
            }
        } else {
            let uniform = 1.0 / action_count as f64;
            for value in strategy.iter_mut().take(action_count) {
                *value = uniform;
            }
        }
        strategy
    }

    pub fn get_average_strategy(&self, action_count: usize) -> [f64; MAX_ACTIONS] {
        let mut strategy = [0.0; MAX_ACTIONS];
        let normalizing_sum: f64 = self.strategy_sum[..action_count].iter().sum();
        if normalizing_sum > 0.0 {
            for (i, value) in strategy.iter_mut().enumerate().take(action_count) {
                *value = self.strategy_sum[i] / normalizing_sum;
            }
        } else {
            let uniform = 1.0 / action_count as f64;
            for value in strategy.iter_mut().take(action_count) {
                *value = uniform;
            }
        }
        strategy
    }

    fn discount(&mut self, iteration: u32) {
        let t = iteration as f64;
        let pos_discount = t.powf(ALPHA) / (t.powf(ALPHA) + 1.0);
        let neg_discount = t.powf(BETA) / (t.powf(BETA) + 1.0);
        let strategy_discount = (t.powf(GAMMA) / (t + 1.0)).powf(GAMMA);
        for regret in &mut self.regret_sum {
            *regret *= if *regret >= 0.0 {
                pos_discount
            } else {
                neg_discount
            };
        }
        for sum in &mut self.strategy_sum {
            *sum *= strategy_discount;
        }
    }
}

pub fn train(iterations: u32) -> HashMap<String, DCFRNode> {
    let mut nodes: HashMap<String, DCFRNode> = HashMap::new();
    let deals = private_card_deals();
    for iteration in 1..=iterations {
        for node in nodes.values_mut() {
            node.discount(iteration);
        }
        for deal in &deals {
            let state = LeducState::new(*deal);
            cfr(&mut nodes, state, 1.0, 1.0);
        }
    }
    nodes
}

pub fn cfr(nodes: &mut HashMap<String, DCFRNode>, state: LeducState, p0: f64, p1: f64) -> f64 {
    if state.is_terminal() {
        return state.payoff(0);
    }

    if state.round == 1 && state.community.is_none() {
        let mut utility = 0.0;
        let community_cards = state.remaining_community_cards();
        let chance_prob = 1.0 / community_cards.len() as f64;
        for card in community_cards {
            let mut next_state = state.clone();
            next_state.community = Some(card);
            utility += chance_prob * cfr(nodes, next_state, p0, p1);
        }
        return utility;
    }

    let player = state.current_player();
    let legal_actions = state.legal_actions();
    let action_count = legal_actions.len();
    let info_set = infoset::key(&state, player);
    let strategy = nodes
        .entry(info_set.clone())
        .or_insert_with(DCFRNode::new)
        .get_strategy(action_count);

    {
        let node = nodes.get_mut(&info_set).expect("node exists");
        let reach = if player == 0 { p0 } else { p1 };
        for (i, probability) in strategy.iter().enumerate().take(action_count) {
            node.strategy_sum[i] += reach * probability;
        }
    }

    let mut action_utils = [0.0; MAX_ACTIONS];
    let mut node_util = 0.0;
    for (i, action) in legal_actions.iter().enumerate() {
        let mut next_state = state.clone();
        next_state.apply_action(*action);
        action_utils[i] = if player == 0 {
            cfr(nodes, next_state, p0 * strategy[i], p1)
        } else {
            cfr(nodes, next_state, p0, p1 * strategy[i])
        };
        node_util += strategy[i] * action_utils[i];
    }

    let perspective_node_util = if player == 0 { node_util } else { -node_util };
    let opponent_reach = if player == 0 { p1 } else { p0 };
    let node = nodes.get_mut(&info_set).expect("node exists");
    for i in 0..action_count {
        let perspective_action_util = if player == 0 {
            action_utils[i]
        } else {
            -action_utils[i]
        };
        node.regret_sum[i] += opponent_reach * (perspective_action_util - perspective_node_util);
    }

    node_util
}

pub fn private_card_deals() -> Vec<[crate::leduc::LeducCard; 2]> {
    let deck = LeducDeck::cards();
    let mut deals = Vec::with_capacity(30);
    for i in 0..deck.len() {
        for j in 0..deck.len() {
            if i != j {
                deals.push([deck[i], deck[j]]);
            }
        }
    }
    deals
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dcfr_runs() {
        let nodes = train(1_000);
        assert!(!nodes.is_empty());
    }
}
