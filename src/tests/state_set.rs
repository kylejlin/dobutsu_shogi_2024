use super::*;

use std::collections::HashSet;

use rand::Rng;

#[test]
fn state_set_is_consistent_with_hash_set() {
    const FUZZ_TIMES: usize = 1000;

    let mut prng = deterministic_prng();

    for _ in 0..FUZZ_TIMES {
        let (mut state_set, reference) = random_state_set_pair(&mut prng);

        for state in reference.iter().copied() {
            assert!(state_set.add(state).did_addend_already_exist);
        }

        let state_set_vec = state_set.into_unsorted_vec();
        for state in state_set_vec.iter().copied() {
            assert!(reference.contains(&state));
            assert_eq!(state.state(), state.0);
        }

        assert_eq!(state_set_vec.len(), reference.len());
    }
}

#[test]
fn state_set_vec_is_consistent_with_hash_set() {
    const FUZZ_TIMES: usize = 1000;

    let mut prng = deterministic_prng();

    for _ in 0..FUZZ_TIMES {
        let (state_set, reference) = random_state_set_pair(&mut prng);
        let state_set_vec = state_set.into_unsorted_vec();

        for state in reference.iter().copied() {
            assert!(state_set_vec.contains(&state));
        }

        for state in state_set_vec.iter().copied() {
            assert!(reference.contains(&state));
            assert_eq!(state.state(), state.0);
        }

        assert_eq!(state_set_vec.len(), reference.len());
    }
}

fn random_state_set_pair(prng: &mut XorShiftRng) -> (StateSet, HashSet<SearchNode>) {
    let mut state_set = StateSet::empty();
    let mut reference = HashSet::new();

    let count = prng.gen_range(0..1000);
    for _ in 0..count {
        let state = random_state(prng);
        state_set.add(state);
        reference.insert(state);
    }

    (state_set, reference)
}

fn random_state(prng: &mut XorShiftRng) -> SearchNode {
    let raw: u64 = prng.gen();
    let state = SearchNode(raw).state();
    SearchNode(state)
}
