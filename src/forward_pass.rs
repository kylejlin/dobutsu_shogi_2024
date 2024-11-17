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
    let mut child_buffer = Vec::with_capacity(8 * 12);

    while let Some(node) = queue.pop_front() {
        child_buffer.clear();
        node.visit_children(&mut child_buffer);

        for &child in &child_buffer {
            if !reachable_states.add(child).did_addend_already_exist {
                queue.push_back(child);
            }
        }

        on_node_processed(node);
    }

    reachable_states.into_sorted_vec()
}
