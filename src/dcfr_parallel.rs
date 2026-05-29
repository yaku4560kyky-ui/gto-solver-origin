use crate::dcfr::{DCFRNode, cfr, private_card_deals};
use crate::leduc::LeducState;
use rayon::prelude::*;
use std::collections::HashMap;

pub fn train_parallel(iterations: u32) -> HashMap<String, DCFRNode> {
    let deals = private_card_deals();
    let mut nodes = HashMap::new();
    for iteration in 1..=iterations {
        for node in nodes.values_mut() {
            discount_node(node, iteration);
        }
        let snapshot = nodes.clone();
        let iteration_nodes = deals
            .par_iter()
            .map(|deal| {
                let mut local_nodes = snapshot.clone();
                cfr(&mut local_nodes, LeducState::new(*deal), 1.0, 1.0);
                delta_nodes(&snapshot, local_nodes)
            })
            .reduce(HashMap::new, merge_nodes);
        merge_nodes_into(&mut nodes, iteration_nodes);
    }
    nodes
}

fn merge_nodes(
    mut left: HashMap<String, DCFRNode>,
    right: HashMap<String, DCFRNode>,
) -> HashMap<String, DCFRNode> {
    merge_nodes_into(&mut left, right);
    left
}

fn merge_nodes_into(left: &mut HashMap<String, DCFRNode>, right: HashMap<String, DCFRNode>) {
    for (key, node) in right {
        let entry = left.entry(key).or_insert_with(DCFRNode::new);
        for i in 0..3 {
            entry.regret_sum[i] += node.regret_sum[i];
            entry.strategy_sum[i] += node.strategy_sum[i];
        }
    }
}

fn discount_node(node: &mut DCFRNode, iteration: u32) {
    let t = iteration as f64;
    let pos_discount = t.powf(1.5) / (t.powf(1.5) + 1.0);
    let neg_discount = 0.5;
    let strategy_discount = (t.powf(2.0) / (t + 1.0)).powf(2.0);
    for regret in &mut node.regret_sum {
        *regret *= if *regret >= 0.0 {
            pos_discount
        } else {
            neg_discount
        };
    }
    for sum in &mut node.strategy_sum {
        *sum *= strategy_discount;
    }
}

fn delta_nodes(
    before: &HashMap<String, DCFRNode>,
    after: HashMap<String, DCFRNode>,
) -> HashMap<String, DCFRNode> {
    let mut deltas = HashMap::new();
    for (key, after_node) in after {
        let before_node = before.get(&key).cloned().unwrap_or_else(DCFRNode::new);
        let mut delta = DCFRNode::new();
        for i in 0..3 {
            delta.regret_sum[i] = after_node.regret_sum[i] - before_node.regret_sum[i];
            delta.strategy_sum[i] = after_node.strategy_sum[i] - before_node.strategy_sum[i];
        }
        deltas.insert(key, delta);
    }
    deltas
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dcfr;

    #[test]
    fn test_parallel_consistency() {
        let sequential = dcfr::train(1_000);
        let parallel = train_parallel(1_000);
        assert_eq!(sequential.len(), parallel.len());
    }
}
