use super::*;

use std::{collections::HashMap, num::NonZeroU64};

use rand::Rng;

#[test]
fn state_map_is_consistent_with_hash_map() {
    const FUZZ_TIMES: usize = 1000;

    let mut prng = deterministic_prng();

    for _ in 0..FUZZ_TIMES {
        let (state_map, reference) = random_state_map_pair(&mut prng);

        for (&key, &val) in reference.iter() {
            assert_eq!(val, state_map.get(key));
        }

        let mut state_map_cardinality = 0;
        state_map.visit_in_key_order(|key, val| {
            assert_eq!(Some(val), reference.get(&key).copied());
            assert_eq!(key.0 & 0xFF_FFFF_FFFF, key.0);
            state_map_cardinality += 1;
        });

        assert_eq!(state_map_cardinality, reference.len());
    }
}

#[test]
fn state_map_vec_is_consistent_with_hash_map() {
    const FUZZ_TIMES: usize = 1000;

    let mut prng = deterministic_prng();

    for _ in 0..FUZZ_TIMES {
        let (state_map, reference) = random_state_map_pair(&mut prng);
        let state_map_vec = state_map.to_sorted_vec();

        for (&key, &val) in reference.iter() {
            let actual_val = state_map_vec
                .binary_search_by(|other| other.0.cmp(&key))
                .ok()
                .map(|i| state_map_vec[i].1);
            assert_eq!(Some(val), actual_val);
        }

        for (key, val) in state_map_vec.iter().copied() {
            assert_eq!(key.0 & 0xFF_FFFF_FFFF, key.0);
            assert_eq!(Some(val), reference.get(&key).copied());
        }

        assert_eq!(state_map_vec.len(), reference.len());
    }
}

#[test]
fn state_map_vec_is_sorted() {
    const FUZZ_TIMES: usize = 1000;

    let mut prng = deterministic_prng();

    for _ in 0..FUZZ_TIMES {
        let (state_map, _) = random_state_map_pair(&mut prng);
        let state_map_vec = state_map.to_sorted_vec();

        for i in 1..state_map_vec.len() {
            assert!(state_map_vec[i - 1].0 < state_map_vec[i].0);
        }
    }
}

#[test]
fn state_map_visitable_in_key_order() {
    const FUZZ_TIMES: usize = 1000;

    let mut prng = deterministic_prng();

    for _ in 0..FUZZ_TIMES {
        let (state_map, _) = random_state_map_pair(&mut prng);
        let visited = {
            let mut out = vec![];
            state_map.visit_in_key_order(|node, _| out.push(node));
            out
        };

        for i in 1..visited.len() {
            assert!(visited[i - 1].0 < visited[i].0);
        }
    }
}

fn random_state_map_pair(
    prng: &mut XorShiftRng,
) -> (
    StateMap<Option<NonZeroU64>>,
    HashMap<State, Option<NonZeroU64>>,
) {
    let mut state_map = StateMap::empty();
    let mut reference = HashMap::new();

    let count = prng.gen_range(0..1000);
    for _ in 0..count {
        let key = random_state(prng);
        let val = NonZeroU64::new(prng.gen());
        state_map.add(key, val);
        reference.insert(key, val);
    }

    (state_map, reference)
}

fn random_state(prng: &mut XorShiftRng) -> State {
    let raw: u64 = prng.gen();
    State(raw & 0xFF_FFFF_FFFF)
}
