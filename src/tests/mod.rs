use super::{pretty::IntoPretty, *};

use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng;

mod i9;
mod legal_moves;
mod parent_child_consistency;
mod state_set;

#[test]
fn initial_search_node_is_correct() {
    insta::assert_snapshot!(SearchNode::initial().pretty());
}

#[test]
fn initial_search_node_allegiance_inversion_is_correct() {
    insta::assert_snapshot!(SearchNode::initial()
        .into_builder()
        .invert_active_player()
        .pretty());
}

#[test]
fn initial_search_node_partially_built_is_correct() {
    insta::assert_snapshot!(SearchNode::initial().into_builder().build().pretty());
}

#[test]
fn initial_search_node_allegiance_inverted_partially_built_is_correct() {
    insta::assert_snapshot!(SearchNode::initial()
        .into_builder()
        .invert_active_player()
        .build()
        .pretty());
}

#[test]
fn initial_search_node_children_are_correct() {
    insta::assert_snapshot!(SearchNode::initial().children().pretty());
}

#[test]
fn initial_search_node_child0_children_are_correct() {
    let child0 = SearchNode::initial().children()[0].pretty();
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
fn terminality_and_childlessness_are_equivalent() {
    fuzz(1_000_000, |state| {
        let mut has_child = false;
        state.visit_children(|_| {
            has_child = true;
        });

        assert_eq!(state.is_terminal(), !has_child);
    });
}

/// Pseudorandomly plays `game_count` games and calls `callback`
/// for each state in each game.
///
/// The games are guaranteed to be the same for all invocations
/// of `fuzz`, so you can safely use `fuzz` in your
/// tests without introducing nondeterminism.
pub fn fuzz<F: FnMut(SearchNode)>(game_count: usize, mut callback: F) {
    let mut rng = deterministic_prng();
    let mut child_buffer = Vec::with_capacity(8 * 12);

    for _ in 0..game_count {
        let mut state = SearchNode::initial();

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
