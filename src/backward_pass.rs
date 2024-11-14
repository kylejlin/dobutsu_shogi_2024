use super::*;

use std::collections::VecDeque;

/// This function will solve the game when provided
/// with a slice of all reachable states.
///
/// The slice of states will be sorted.
pub fn solve(nodes: &mut [SearchNode], mut on_node_processed: impl FnMut(SearchNode)) {
    nodes.sort_unstable();

    init_required_child_report_count_and_best_known_outcome(nodes);

    let mut known_queue = VecDeque::with_capacity(nodes.len());
    add_terminal_nodes(nodes, &mut known_queue);

    while let Some(child) = known_queue.pop_front() {
        let child_outcome = child.best_known_outcome();

        if child_outcome.0 < 0 {
            visit_parents(child, nodes, |parent_mut| {
                *parent_mut = parent_mut
                    .record_child_outcome(child_outcome)
                    .set_required_child_report_count_to_zero();

                known_queue.push_back(*parent_mut);
            });
        } else {
            visit_parents(child, nodes, |parent_mut| {
                *parent_mut = parent_mut
                    .record_child_outcome(child_outcome)
                    .decrement_required_child_report_count();

                if parent_mut.required_child_report_count() == 0 {
                    known_queue.push_back(*parent_mut);
                }
            });
        }

        on_node_processed(child);
    }
}

#[inline(always)]
fn visit_parents(
    node_with_incorrect_nonstate_fields: SearchNode,
    states: &mut [SearchNode],
    mut parent_mutator: impl FnMut(&mut SearchNode),
) {
    node_with_incorrect_nonstate_fields.visit_parents(|parent_with_incorrect_nonstate_fields| {
        let parent_state = parent_with_incorrect_nonstate_fields.state();
        let Ok(parent_index) = states.binary_search_by(|other| {
            let other_state = other.state();
            other_state.cmp(&parent_state)
        }) else {
            // It's possible that a theoretical parent is actually unreachable.
            return;
        };

        let parent_mut = &mut states[parent_index];

        if parent_mut.required_child_report_count() == 0 {
            // It's possible that the parent has already determined
            // its best outcome before seeing all of its children's best outcomes.
            // This happens when a child reports a loss.
            return;
        }

        parent_mutator(parent_mut);
    });
}

fn init_required_child_report_count_and_best_known_outcome(nodes: &mut [SearchNode]) {
    const DELETION_MASK: u64 = !((0b111_1111 << Offset::REQUIRED_CHILD_REPORT_COUNT.0)
        | (0b1_1111_1111 << Offset::BEST_KNOWN_OUTCOME.0));

    for node in nodes {
        match node.terminality() {
            Terminality::Nonterminal => {
                node.0 = (node.0 & DELETION_MASK)
                    | ((node.total_child_count() as u64) << Offset::REQUIRED_CHILD_REPORT_COUNT.0)
                    | (NEGATIVE_201_I9 << Offset::BEST_KNOWN_OUTCOME.0);
            }

            Terminality::Win => {
                node.0 = (node.0 & DELETION_MASK)
                    | (0 << Offset::REQUIRED_CHILD_REPORT_COUNT.0)
                    | (POSITIVE_201_I9 << Offset::BEST_KNOWN_OUTCOME.0);
            }

            Terminality::Loss => {
                node.0 = (node.0 & DELETION_MASK)
                    | (0 << Offset::REQUIRED_CHILD_REPORT_COUNT.0)
                    | (NEGATIVE_201_I9 << Offset::BEST_KNOWN_OUTCOME.0);
            }
        }
    }
}

fn add_terminal_nodes(nodes: &[SearchNode], known_queue: &mut VecDeque<SearchNode>) {
    for node in nodes {
        if node.is_terminal() {
            known_queue.push_back(*node);
        }
    }
}

impl SearchNode {
    fn total_child_count(self) -> u8 {
        let mut count = 0;
        self.visit_children(|_| count += 1);
        count
    }

    #[must_use]
    fn record_child_outcome(self, child_outcome: Outcome) -> Self {
        let incumbent = self.best_known_outcome();
        let challenger = child_outcome.invert().delay_by_one();
        if challenger > incumbent {
            Self(
                (self.0 & !(0b1_1111_1111 << Offset::BEST_KNOWN_OUTCOME.0))
                    | (challenger.0.into_zero_padded_i9_unchecked()
                        << Offset::BEST_KNOWN_OUTCOME.0),
            )
        } else {
            self
        }
    }

    #[must_use]
    fn decrement_required_child_report_count(self) -> Self {
        Self(self.0 - (1 << Offset::REQUIRED_CHILD_REPORT_COUNT.0))
    }

    #[must_use]
    fn set_required_child_report_count_to_zero(self) -> Self {
        Self(self.0 & !(0b111_1111 << Offset::REQUIRED_CHILD_REPORT_COUNT.0))
    }
}

impl Outcome {
    const fn invert(self) -> Self {
        Self(-self.0)
    }

    const fn delay_by_one(self) -> Self {
        Self(self.0 - self.0.signum())
    }
}
