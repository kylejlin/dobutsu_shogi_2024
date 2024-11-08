use std::fs;
use std::path::Path;
use std::time::Instant;

use dobutsu_shogi_2024::forward::*;

fn main() {
    println!("Starting tree search. This will probably take several hours.");
    let now = Instant::now();
    let reachable_states = reachable_states(SearchNode::initial());
    let elapsed = now.elapsed();
    println!("Completed tree search. It took {:?}.", elapsed);

    let path = Path::new(file!())
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("solution.dat");
    let bytes = reachable_states_to_bytes(&reachable_states);
    fs::write(path, bytes).unwrap();
}

fn reachable_states_to_bytes(reachable_states: &[BackwardNode]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(reachable_states.len() * std::mem::size_of::<u64>());
    for state in reachable_states {
        bytes.extend_from_slice(&state.0.to_le_bytes())
    }
    bytes
}
