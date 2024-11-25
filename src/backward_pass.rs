use super::*;

use std::collections::VecDeque;

pub fn initial_stat_map(
    reachable: &StateSet,
    mut on_state_processed: impl FnMut(State),
) -> StateMap<StateStats> {
    let mut map = StateMap::empty();

    reachable.visit(|state| {
        map.add(state, state.guess_stats());

        on_state_processed(state);
    });

    map
}

/// This function will solve the game when provided
/// with a state map of all possible states.
///
/// The map must be initialized such that every state `s`
/// is mapped to `s.guess_stats()`.
pub fn compute_stats(
    map: &mut StateMap<StateStats>,
    progress: &mut Progress,
    mut on_state_processed: impl FnMut(&Progress) -> bool,
) {
    let mut known_queue: VecDeque<(State, Outcome)> = VecDeque::new();
    add_terminal_states(map, &mut known_queue);

    while let Some((child, child_outcome)) = known_queue.pop_front() {
        if child_outcome.0 < 0 {
            visit_parents(
                child,
                map,
                progress,
                |original_parent, parent_stats_mut, progress| {
                    *parent_stats_mut = parent_stats_mut
                        .record_child_outcome(child_outcome)
                        .set_required_child_report_count_to_zero();

                    known_queue.push_back((original_parent, parent_stats_mut.best_known_outcome()));

                    progress.queue_pushes += 1;
                    progress.winning_parent_conclusions += 1;
                },
            );
        } else {
            visit_parents(
                child,
                map,
                progress,
                |original_parent, parent_stats_mut, progress| {
                    *parent_stats_mut = parent_stats_mut
                        .record_child_outcome(child_outcome)
                        .decrement_required_child_report_count();

                    if parent_stats_mut.required_child_report_count() == 0 {
                        known_queue
                            .push_back((original_parent, parent_stats_mut.best_known_outcome()));

                        progress.queue_pushes += 1;
                        progress.losing_parent_conclusions += 1;
                    } else {
                        progress.uncertain_parent_conclusions += 1;
                    }
                },
            );
        }

        if on_state_processed(&progress) {
            *progress = Progress::default();
        }
    }
}

#[inline(always)]
fn visit_parents(
    child: State,
    map: &mut StateMap<StateStats>,
    progress: &mut Progress,
    mut visitor: impl FnMut(State, &mut StateStats, &mut Progress),
) {
    child.visit_parents(|parent| {
        let Some(parent_stats_mut) = map.get_mut(parent) else {
            // It's possible that a theoretical parent is actually unreachable.
            progress.unreachable_parent_visits += 1;
            return;
        };
        let original_parent_stats = *parent_stats_mut;

        let required_child_report_count = original_parent_stats.required_child_report_count();

        // TODO: Delete after debugging.
        {
            use crate::pretty::*;
            assert!(
                required_child_report_count <= 8 * 12,
                "Required_child_report_count is too large ({}).\n\nSTATE:\n\n{}",
                required_child_report_count,
                parent.with_stats(original_parent_stats).pretty()
            );
        }

        if required_child_report_count == 0 {
            // It's possible that the parent has already determined
            // its best outcome before seeing all of its children's best outcomes.
            // This happens when a child reports a loss.
            progress.already_solved_parent_visits += 1;
            return;
        }

        progress.unsolved_parent_visits += 1;

        visitor(parent, parent_stats_mut, progress);
    });
}

fn add_terminal_states(map: &StateMap<StateStats>, known_queue: &mut VecDeque<(State, Outcome)>) {
    map.visit_in_key_order(|state, _| match state.terminality() {
        Terminality::Loss => known_queue.push_back((state, Outcome::loss_in(0))),
        Terminality::Win => known_queue.push_back((state, Outcome::win_in(0))),

        Terminality::Nonterminal => {}
    });
}
