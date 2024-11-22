use core::panic;
use std::collections::VecDeque;

use super::*;

use crate::pretty::*;

#[derive(Clone, Copy, Debug)]
struct QueueItem {
    state: SearchNode,
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

    match optimal_player {
        Player::Sente => {
            once_enqueued_states_where_optimal_player_is_active.add(initial_state);
        }
        Player::Gote => {
            once_enqueued_states_where_unpredictable_player_is_active.add(initial_state);
        }
    }
    queue.push_back(QueueItem {
        state: initial_state,
        active_player: Player::Sente,
    });

    while let Some(item) = queue.pop_front() {
        // If the active player is the optimal player,
        // we only need to explore the best child.

        if item.active_player == optimal_player {
            let Some(best_child) = best_child(item.state, nodes) else {
                continue;
            };

            if once_enqueued_states_where_unpredictable_player_is_active
                .add(best_child)
                .did_addend_already_exist
            {
                continue;
            }

            queue.push_back(QueueItem {
                state: best_child,
                active_player: !item.active_player,
            });

            on_node_processed(item.state);

            continue;
        }

        // Otherwise, the active player is the unpredictable player.
        // Therefore, we need to explore all possible children.

        item.state.visit_children(|child| {
            if once_enqueued_states_where_optimal_player_is_active
                .add(child)
                .did_addend_already_exist
            {
                return;
            }

            queue.push_back(QueueItem {
                state: child,
                active_player: !item.active_player,
            });
        });

        on_node_processed(item.state);
    }

    once_enqueued_states_where_optimal_player_is_active
        .union(&once_enqueued_states_where_unpredictable_player_is_active)
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
