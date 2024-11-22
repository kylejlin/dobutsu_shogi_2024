use super::*;

use crate::{pretty::*, state_map::Null};

impl Null for SearchNode {
    fn null() -> Self {
        SearchNode(0)
    }
}

pub fn best_child_map(nodes: &[SearchNode]) -> StateMap<SearchNode> {
    let mut out = StateMap::empty();

    for &node in nodes {
        out.add(node, best_child(node, nodes).unwrap_or_else(Null::null));
    }

    out
}

fn best_child(parent: SearchNode, nodes: &[SearchNode]) -> Option<SearchNode> {
    let mut best_child = None;
    let mut best_outcome = Outcome(i16::MAX);
    parent.visit_children(|child| {
        let outcome = get_node_outcome(child, nodes).unwrap_or(Outcome(0));
        // We invert perspectives, since child nodes represent the opponent's turn.
        // Therefore, lower scores are better.
        if outcome < best_outcome {
            best_child = Some(child);
            best_outcome = outcome;
        }
    });
    best_child
}

fn get_node_outcome(
    node_with_incorrect_nonstate_fields: SearchNode,
    nodes: &[SearchNode],
) -> Option<Outcome> {
    find(node_with_incorrect_nonstate_fields, nodes).best_outcome()
}

fn find(node_with_incorrect_nonstate_fields: SearchNode, nodes: &[SearchNode]) -> SearchNode {
    let state = node_with_incorrect_nonstate_fields.state();
    let node = nodes
        .binary_search_by(|other| other.state().cmp(&state))
        .ok()
        .map(|i| nodes[i]);

    if let Some(node) = node {
        node
    } else {
        panic!(
            "Could not find node in node vector.\n\nNode:\n\n{}",
            node_with_incorrect_nonstate_fields.pretty()
        )
    }
}
