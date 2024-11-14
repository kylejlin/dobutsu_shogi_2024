use super::*;

use std::collections::VecDeque;

/// Returns a sorted vector of all states reachable from the provided initial state.
pub fn reachable_states(
    initial_state: SearchNode,
    mut on_node_processed: impl FnMut(SearchNode),
) -> Vec<SearchNode> {
    let mut reachable_states = StateSet::empty();
    reachable_states.add(initial_state);

    let mut queue = std::iter::once(initial_state).collect::<VecDeque<_>>();

    while let Some(node) = queue.pop_front() {
        node.visit_children(|new_child| {
            if !reachable_states.add(new_child).did_addend_already_exist {
                queue.push_back(new_child);
            }
        });

        on_node_processed(node);
    }

    reachable_states.into_sorted_vec()
}
