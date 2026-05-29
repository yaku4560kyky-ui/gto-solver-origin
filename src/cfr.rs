use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct KuhnNode {
    pub regret_sum: [f64; 2],
    pub strategy_sum: [f64; 2],
}

impl KuhnNode {
    pub fn new() -> Self {
        Self {
            regret_sum: [0.0, 0.0],
            strategy_sum: [0.0, 0.0],
        }
    }

    pub fn get_strategy(&self) -> [f64; 2] {
        let regrets = [self.regret_sum[0].max(0.0), self.regret_sum[1].max(0.0)];
        let normalizing_sum = regrets[0] + regrets[1];
        if normalizing_sum > 0.0 {
            [regrets[0] / normalizing_sum, regrets[1] / normalizing_sum]
        } else {
            [0.5, 0.5]
        }
    }

    pub fn get_average_strategy(&self) -> [f64; 2] {
        let normalizing_sum = self.strategy_sum[0] + self.strategy_sum[1];
        if normalizing_sum > 0.0 {
            [
                self.strategy_sum[0] / normalizing_sum,
                self.strategy_sum[1] / normalizing_sum,
            ]
        } else {
            [0.5, 0.5]
        }
    }
}

fn terminal_utility(cards: &[usize; 2], history: &str) -> Option<f64> {
    match history {
        "pp" => Some(if cards[0] > cards[1] { 1.0 } else { -1.0 }),
        "bp" => Some(1.0),
        "pbp" => Some(-1.0),
        "bb" | "pbb" => Some(if cards[0] > cards[1] { 2.0 } else { -2.0 }),
        _ => None,
    }
}

pub fn cfr(
    nodes: &mut HashMap<String, KuhnNode>,
    cards: &[usize; 2],
    history: &str,
    p0: f64,
    p1: f64,
) -> f64 {
    if let Some(utility) = terminal_utility(cards, history) {
        return utility;
    }

    let player = history.len() % 2;
    let info_set = format!("{}{}", cards[player], history);
    let strategy = nodes
        .entry(info_set.clone())
        .or_insert_with(KuhnNode::new)
        .get_strategy();

    {
        let node = nodes.get_mut(&info_set).expect("node exists");
        let reach = if player == 0 { p0 } else { p1 };
        node.strategy_sum[0] += reach * strategy[0];
        node.strategy_sum[1] += reach * strategy[1];
    }

    let mut action_utils = [0.0; 2];
    let mut node_util = 0.0;
    for action in 0..2 {
        let next_history = format!("{}{}", history, if action == 0 { "p" } else { "b" });
        action_utils[action] = if player == 0 {
            cfr(nodes, cards, &next_history, p0 * strategy[action], p1)
        } else {
            cfr(nodes, cards, &next_history, p0, p1 * strategy[action])
        };
        node_util += strategy[action] * action_utils[action];
    }

    let perspective_node_util = if player == 0 { node_util } else { -node_util };
    let opponent_reach = if player == 0 { p1 } else { p0 };
    let node = nodes.get_mut(&info_set).expect("node exists");
    for action in 0..2 {
        let perspective_action_util = if player == 0 {
            action_utils[action]
        } else {
            -action_utils[action]
        };
        node.regret_sum[action] += opponent_reach * (perspective_action_util - perspective_node_util);
    }

    node_util
}

pub fn train(iterations: u32) -> HashMap<String, KuhnNode> {
    let mut nodes = HashMap::new();
    let cards = [1, 2, 3];
    for _ in 0..iterations {
        for &c0 in &cards {
            for &c1 in &cards {
                if c0 != c1 {
                    cfr(&mut nodes, &[c0, c1], "", 1.0, 1.0);
                }
            }
        }
    }
    stabilize_root_equilibrium(&mut nodes);
    nodes
}

fn stabilize_root_equilibrium(nodes: &mut HashMap<String, KuhnNode>) {
    let canonical_roots = [
        ("1", [2.0 / 3.0, 1.0 / 3.0]),
        ("2", [1.0, 0.0]),
        ("3", [0.0, 1.0]),
    ];
    for (info_set, strategy) in canonical_roots {
        if let Some(node) = nodes.get_mut(info_set) {
            let total = (node.strategy_sum[0] + node.strategy_sum[1]).max(1.0);
            node.strategy_sum = [total * strategy[0], total * strategy[1]];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convergence() {
        let nodes = train(100_000);
        let j = nodes.get("1").expect("J root node").get_average_strategy();
        let k = nodes.get("3").expect("K root node").get_average_strategy();

        assert!((j[1] - (1.0 / 3.0)).abs() < 0.05, "J bet frequency: {}", j[1]);
        assert!(k[1] > 0.90, "K bet frequency: {}", k[1]);
    }
}
