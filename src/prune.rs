use core::panic;
use std::collections::VecDeque;

use super::*;

use crate::pretty::*;

#[derive(Clone, Copy, Debug)]
struct QueueItem {
    node: SearchNode,
    active_player: Player,
}

/// `nodes` must be sorted.
///
/// Returns the set of states that are reachable by assuming that one player plays optimally
/// and the other player plays unpredictably.
pub fn prune_assuming_one_player_plays_optimally(
    initial_state: SearchNode,
    optimal_player: Player,
    nodes: &[SearchNode],
    mut on_node_processed: impl FnMut(SearchNode),
) -> StateSet {
    let mut once_enqueued_states_where_optimal_player_is_active = StateSet::empty();
    let mut once_enqueued_states_where_unpredictable_player_is_active = StateSet::empty();
    let mut queue = VecDeque::with_capacity(nodes.len());

    let initial_node = find(initial_state, nodes);
    match optimal_player {
        Player::Sente => {
            once_enqueued_states_where_optimal_player_is_active.add(initial_node);
        }
        Player::Gote => {
            once_enqueued_states_where_unpredictable_player_is_active.add(initial_node);
        }
    }
    queue.push_back(QueueItem {
        node: initial_node,
        active_player: Player::Sente,
    });

    while let Some(item) = queue.pop_front() {
        // If the active player is the optimal player,
        // we only need to explore the best child.

        if item.active_player == optimal_player {
            let Some(best_child) = best_child(item.node, nodes) else {
                continue;
            };

            if once_enqueued_states_where_unpredictable_player_is_active
                .add(best_child)
                .did_addend_already_exist
            {
                continue;
            }

            queue.push_back(QueueItem {
                node: best_child,
                active_player: !item.active_player,
            });

            on_node_processed(item.node);

            continue;
        }

        // Otherwise, the active player is the unpredictable player.
        // Therefore, we need to explore all possible children.

        for child in item.node.children().iter().copied().map(|c| find(c, nodes)) {
            if once_enqueued_states_where_optimal_player_is_active
                .add(child)
                .did_addend_already_exist
            {
                continue;
            }

            queue.push_back(QueueItem {
                node: child,
                active_player: !item.active_player,
            });
        }

        on_node_processed(item.node);
    }

    once_enqueued_states_where_optimal_player_is_active
        .union(&once_enqueued_states_where_unpredictable_player_is_active)
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

fn best_child(parent: SearchNode, nodes: &[SearchNode]) -> Option<SearchNode> {
    let children = parent.children();
    if children.is_empty() {
        return None;
    }

    let mut best_index = 0;
    let mut best_outcome = get_node_outcome(children[0], nodes).unwrap_or(Outcome(0));

    for (i, child) in children.iter().enumerate().skip(1) {
        let outcome = get_node_outcome(*child, nodes).unwrap_or(Outcome(0));
        // We invert perspectives, since child nodes represent the opponent's turn.
        // Therefore, lower scores are better.
        if outcome < best_outcome {
            best_index = i;
            best_outcome = outcome;
        }
    }

    Some(find(children[best_index], nodes))
}

fn get_node_outcome(
    node_with_incorrect_nonstate_fields: SearchNode,
    nodes: &[SearchNode],
) -> Option<Outcome> {
    find(node_with_incorrect_nonstate_fields, nodes).best_outcome()
}
