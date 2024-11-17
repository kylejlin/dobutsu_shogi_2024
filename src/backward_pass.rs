use super::*;

use std::collections::VecDeque;

/// This function will solve the game when provided
/// with a slice of all reachable states.
///
/// The slice of states will be sorted.
pub fn solve(nodes: &mut [SearchNode], mut on_node_processed: impl FnMut(SearchNode)) {
    nodes.sort_unstable();

    let mut parent_buffer = Vec::with_capacity(8 * 12);

    init_required_child_report_count_and_best_known_outcome(nodes, &mut parent_buffer);

    let mut known_queue = VecDeque::with_capacity(nodes.len());
    add_terminal_nodes(nodes, &mut known_queue);

    while let Some(child) = known_queue.pop_front() {
        parent_buffer.clear();
        child.visit_parents(&mut parent_buffer);

        let child_outcome = child.best_known_outcome();

        if child_outcome.0 < 0 {
            for &parent_with_incorrect_nonstate_fields in &parent_buffer {
                let parent_state = parent_with_incorrect_nonstate_fields.state();
                let Ok(parent_index) = nodes.binary_search_by(|other| {
                    let other_state = other.state();
                    other_state.cmp(&parent_state)
                }) else {
                    // It's possible that a theoretical parent is actually unreachable.
                    continue;
                };

                let original_parent = nodes[parent_index];

                if original_parent.required_child_report_count() == 0 {
                    // It's possible that the parent has already determined
                    // its best outcome before seeing all of its children's best outcomes.
                    // This happens when a child reports a loss.
                    continue;
                }

                let updated_parent = original_parent
                    .record_child_outcome(child_outcome)
                    .set_required_child_report_count_to_zero();

                known_queue.push_back(updated_parent);

                nodes[parent_index] = updated_parent;
            }
        } else {
            for &parent_with_incorrect_nonstate_fields in &parent_buffer {
                let parent_state = parent_with_incorrect_nonstate_fields.state();
                let Ok(parent_index) = nodes.binary_search_by(|other| {
                    let other_state = other.state();
                    other_state.cmp(&parent_state)
                }) else {
                    // It's possible that a theoretical parent is actually unreachable.
                    continue;
                };

                let original_parent = nodes[parent_index];

                if original_parent.required_child_report_count() == 0 {
                    // It's possible that the parent has already determined
                    // its best outcome before seeing all of its children's best outcomes.
                    // This happens when a child reports a loss.
                    continue;
                }

                let updated_parent = original_parent
                    .record_child_outcome(child_outcome)
                    .decrement_required_child_report_count();

                if updated_parent.required_child_report_count() == 0 {
                    known_queue.push_back(updated_parent);
                }

                nodes[parent_index] = updated_parent;
            }
        }

        on_node_processed(child);
    }
}

fn init_required_child_report_count_and_best_known_outcome(
    nodes: &mut [SearchNode],
    parent_buffer: &mut Vec<SearchNode>,
) {
    const DELETION_MASK: u64 = !((0b111_1111 << Offset::REQUIRED_CHILD_REPORT_COUNT.0)
        | (0b1_1111_1111 << Offset::BEST_KNOWN_OUTCOME.0));

    for node in nodes {
        match node.terminality() {
            Terminality::Nonterminal => {
                node.0 = (node.0 & DELETION_MASK)
                    | ((node.total_child_count(parent_buffer) as u64)
                        << Offset::REQUIRED_CHILD_REPORT_COUNT.0)
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
    fn total_child_count(self, parent_buffer: &mut Vec<SearchNode>) -> u8 {
        parent_buffer.clear();
        self.visit_children(parent_buffer);
        parent_buffer.len() as u8
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
