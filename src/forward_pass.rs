use super::*;

use std::collections::VecDeque;

/// Returns a `StateMap` that maps each reachable state `s`
/// to `s.guess_stats()`.
pub fn reachable_states(
    initial_state: State,
    mut on_state_processed: impl FnMut(State),
) -> StateMap<StateStats> {
    let mut reachable = StateMap::empty();
    reachable.add(initial_state, initial_state.guess_stats());

    let mut queue = std::iter::once(initial_state).collect::<VecDeque<_>>();

    while let Some(state) = queue.pop_front() {
        state.visit_children(|new_child| {
            if !reachable
                .add(new_child, new_child.guess_stats())
                .did_addend_already_exist
            {
                queue.push_back(new_child);
            }
        });

        on_state_processed(state);
    }

    reachable
}
