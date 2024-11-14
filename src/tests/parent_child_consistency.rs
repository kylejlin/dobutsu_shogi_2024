use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng;

use super::*;

#[ignore]
#[test]
fn every_child_lists_parent_as_parent() {
    fuzz(10_000, |state| {
        state.visit_children(|child| {
            let mut found_parent = false;
            child.visit_parents(|parent| {
                found_parent |= parent == state;
            });
            if !found_parent {
                let parent = state.pretty();
                let parent_children = parent.0.children().pretty();
                let child = child.pretty();
                let child_parents = child.0.parents().pretty();
                panic!("Child did not list parent as parent.\n\nPARENT:\n\n{parent}\n\nPARENT.CHILDREN:\n\n{parent_children}\n\nCHILD:\n\n{child}\n\nCHILD.PARENTS:\n\n{child_parents}");
            }
        })
    });
}

/// Pseudorandomly plays `game_count` games and calls `callback`
/// for each state in each game.
///
/// The games are guaranteed to be the same for all invocations
/// of `fuzz`, so you can safely use `fuzz` in your
/// tests without introducing nondeterminism.
pub fn fuzz<F: FnMut(SearchNode)>(game_count: usize, mut callback: F) {
    let mut rng = XorShiftRng::from_seed(PRNG_SEED);
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

/// Randomly chosen seed
pub const PRNG_SEED: [u8; 16] = [
    113, 8, 5, 99, 97, 161, 194, 214, 140, 140, 80, 143, 213, 130, 254, 107,
];
