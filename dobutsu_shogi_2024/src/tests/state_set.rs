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

        let mut state_set_cardinality = 0;
        state_set.visit_in_order(|state| {
            assert_eq!(state.0 & 0xFF_FFFF_FFFF, state.0);
            assert!(reference.contains(&state));
            state_set_cardinality += 1;
        });

        assert_eq!(state_set_cardinality, reference.len());
    }
}

#[test]
fn state_set_vec_is_consistent_with_hash_set() {
    const FUZZ_TIMES: usize = 1000;

    let mut prng = deterministic_prng();

    for _ in 0..FUZZ_TIMES {
        let (state_set, reference) = random_state_set_pair(&mut prng);
        let state_set_vec = state_set.to_sorted_vec();

        for state in reference.iter().copied() {
            assert!(state_set_vec.contains(&state));
        }

        for state in state_set_vec.iter().copied() {
            assert_eq!(state.0 & 0xFF_FFFF_FFFF, state.0);
            assert!(reference.contains(&state));
        }

        assert_eq!(state_set_vec.len(), reference.len());
    }
}

#[test]
fn state_set_vec_is_sorted() {
    const FUZZ_TIMES: usize = 1000;

    let mut prng = deterministic_prng();

    for _ in 0..FUZZ_TIMES {
        let (state_set, _) = random_state_set_pair(&mut prng);
        let state_set_vec = state_set.to_sorted_vec();

        for i in 1..state_set_vec.len() {
            assert!(state_set_vec[i - 1].0 < state_set_vec[i].0);
        }
    }
}

#[test]
fn state_set_visitable_in_key_order() {
    const FUZZ_TIMES: usize = 1000;

    let mut prng = deterministic_prng();

    for _ in 0..FUZZ_TIMES {
        let (state_map, _) = random_state_set_pair(&mut prng);
        let visited = {
            let mut out = vec![];
            state_map.visit_in_order(|node| out.push(node));
            out
        };

        for i in 1..visited.len() {
            assert!(visited[i - 1].0 < visited[i].0);
        }
    }
}

fn random_state_set_pair(prng: &mut XorShiftRng) -> (StateSet, HashSet<State>) {
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

fn random_state(prng: &mut XorShiftRng) -> State {
    let raw: u64 = prng.gen();
    State(raw & 0xFF_FFFF_FFFF)
}
