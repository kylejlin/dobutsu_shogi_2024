use std::collections::VecDeque;

use super::*;

use crate::state_map::*;

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
    best_child_map: &StateMap<SearchNode>,
    mut on_node_processed: impl FnMut(SearchNode),
) -> StateSet {
    let mut once_enqueued_states_where_optimal_player_is_active = StateSet::empty();
    let mut once_enqueued_states_where_unpredictable_player_is_active = StateSet::empty();
    let mut queue = VecDeque::new();

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
            let best_child = best_child_map.get(item.state);

            if best_child.is_null() {
                continue;
            }

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
