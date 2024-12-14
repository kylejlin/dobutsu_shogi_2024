use super::*;

use std::collections::VecDeque;

/// Returns a set of states reachable from
/// the provided initial state.
pub fn reachable_states(
    initial_state: State,
    mut on_state_processed: impl FnMut(State),
) -> StateSet {
    let mut reachable = StateSet::empty();
    reachable.add(initial_state);

    let mut queue = std::iter::once(initial_state).collect::<VecDeque<_>>();

    while let Some(state) = queue.pop_front() {
        state.visit_children(|new_child| {
            if !reachable.add(new_child).did_addend_already_exist {
                queue.push_back(new_child);
            }
        });

        on_state_processed(state);
    }

    reachable
}
