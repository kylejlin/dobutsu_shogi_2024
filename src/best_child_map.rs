use super::*;

use crate::pretty::*;

pub fn best_child_map(
    map: &StateMap<StateStats>,
    mut on_state_processed: impl FnMut(State),
) -> StateMap<StateAndStats> {
    let mut out = StateMap::empty();

    map.visit_in_key_order(|parent, _| {
        if let Some(best_child) = parent.best_child(map) {
            out.add(parent, best_child);
        }

        on_state_processed(parent);
    });

    out
}

impl State {
    fn best_child(self, map: &StateMap<StateStats>) -> Option<StateAndStats> {
        let mut best_child = None;
        let mut best_outcome = Outcome(i16::MAX);
        self.visit_children(|child| {
            let outcome = child.outcome(map).unwrap_or(Outcome(0));
            // We invert perspectives, since child nodes represent the opponent's turn.
            // Therefore, lower scores are better.
            if outcome < best_outcome {
                best_child = Some(child);
                best_outcome = outcome;
            }
        });

        let best_child = best_child?;
        Some(best_child.with_stats(best_child.stats(map)))
    }

    fn outcome(self, map: &StateMap<StateStats>) -> Option<Outcome> {
        self.stats(map).best_outcome()
    }

    fn stats(self, map: &StateMap<StateStats>) -> StateStats {
        let stats = map.get(self);
        assert!(
            !stats.is_null(),
            "State is not in stats map.\n\nSTATE:\n\n{}",
            self.pretty()
        );
        stats
    }
}
