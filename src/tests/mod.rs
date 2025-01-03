use super::{pretty::IntoPretty, *};

use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng;

use std::collections::HashSet;

mod i9;
mod legal_moves;
mod state_map;
mod state_set;

#[test]
fn initial_state_is_correct() {
    insta::assert_snapshot!(State::initial().pretty());
}

#[test]
fn initial_state_allegiance_inversion_is_correct() {
    insta::assert_snapshot!(State::initial()
        .into_builder()
        .invert_active_player()
        .pretty());
}

#[test]
fn initial_state_partially_built_is_correct() {
    insta::assert_snapshot!(State::initial().into_builder().build().pretty());
}

#[test]
fn initial_state_allegiance_inverted_partially_built_is_correct() {
    insta::assert_snapshot!(State::initial()
        .into_builder()
        .invert_active_player()
        .build()
        .pretty());
}

#[test]
fn initial_state_children_are_correct() {
    insta::assert_snapshot!(State::initial().children().pretty());
}

#[test]
fn initial_state_child0_children_are_correct() {
    let child0 = State::initial().children()[0].pretty();
    let children = child0.0.children().pretty();
    insta::assert_snapshot!(format!("parent:\n{child0}\n\nchildren:\n{children}"));
}

#[test]
fn allegiance_inversion_is_involutive() {
    fuzz(1_000_000, |state| {
        let state = state.into_builder();
        let double_inverted = state.invert_active_player().invert_active_player();
        if state != double_inverted {
            let state = state.pretty();
            let double_inverted = double_inverted.pretty();
            panic!("Allegiance inversion is not involutive.\n\nSTATE:\n\n{state}\n\nDOUBLE INVERTED:\n\n{double_inverted}");
        }
    });
}

#[test]
fn every_child_lists_parent() {
    fuzz(1_000_000, |parent| {
        parent.visit_children(|child| {
            let mut found_parent = false;
            child.visit_parents(|child_parent| {
                found_parent |= child_parent == parent;
            });
            if !found_parent {
                let parent = parent.pretty();
                let parent_children = parent.0.children().pretty();
                let child = child.pretty();
                let child_parents = child.0.parents().pretty();
                panic!("Child did not list parent.\n\nPARENT:\n\n{parent}\n\nCHILD:\n\n{child}\n\nCHILD.PARENTS:\n\n{child_parents}\n\nPARENT.CHILDREN:\n\n{parent_children}");
            }
        });
    });
}

#[test]
fn every_parent_lists_child() {
    fuzz(1_000_000, |child| {
        child.visit_parents(|parent| {
            let mut found_child = false;
            parent.visit_children(|parent_child| {
                found_child |= parent_child == child;
            });
            if !found_child {
                let child = child.pretty();
                let child_parents = child.0.parents().pretty();
                let parent = parent.pretty();
                let parent_children = parent.0.children().pretty();
                panic!("Parent did not list child.\n\nCHILD:\n\n{child}\n\nPARENT:\n\n{parent}\n\nPARENT.CHILDREN:\n\n{parent_children}\n\nCHILD.PARENTS:\n\n{child_parents}");
            }
        })
    });
}

#[test]
fn terminality_and_childlessness_are_equivalent() {
    fuzz(1_000_000, |state| {
        let mut has_child = false;
        state.visit_children(|_| {
            has_child = true;
        });

        assert_eq!(state.is_terminal(), !has_child);
    });
}

#[test]
fn visited_children_are_unique() {
    fuzz(1_000_000, |parent| {
        let mut visited = HashSet::new();
        parent.visit_children(|child| {
            if visited.contains(&child) {
                let parent = parent.pretty();
                let child = child.pretty();
                panic!("Visited child twice.\n\nPARENT:\n\n{parent}\n\nCHILD:\n\n{child}");
            }

            visited.insert(child);
        });
    });
}

#[test]
fn visited_parents_are_unique() {
    fuzz(1_000_000, |child| {
        let mut visited = HashSet::new();
        child.visit_parents(|parent| {
            if visited.contains(&parent) {
                let child = child.pretty();
                let parent = parent.pretty();
                panic!("Visited parent twice.\n\nCHILD:\n\n{child}\n\nPARENT:\n\n{parent}");
            }

            visited.insert(parent);
        });
    });
}

/// Pseudorandomly plays `game_count` games and calls `callback`
/// for each state in each game.
///
/// The games are guaranteed to be the same for all invocations
/// of `fuzz`, so you can safely use `fuzz` in your
/// tests without introducing nondeterminism.
pub fn fuzz<F: FnMut(State)>(game_count: usize, mut callback: F) {
    let mut rng = deterministic_prng();
    let mut child_buffer = Vec::with_capacity(8 * 12);

    for _ in 0..game_count {
        let mut state = State::initial();

        callback(state);

        while !state.is_terminal() {
            child_buffer.clear();
            state.visit_children(|child| child_buffer.push(child));

            let selected_child = child_buffer[rng.gen_range(0..child_buffer.len())];

            callback(selected_child);

            state = selected_child;
        }
    }
}

fn deterministic_prng() -> XorShiftRng {
    /// Randomly chosen seed
    const PRNG_SEED: [u8; 16] = [
        113, 8, 5, 99, 97, 161, 194, 214, 140, 140, 80, 143, 213, 130, 254, 107,
    ];

    XorShiftRng::from_seed(PRNG_SEED)
}
