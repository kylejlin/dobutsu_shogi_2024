use std::fs;
use std::path::Path;
use std::time::Instant;

use dobutsu_shogi_2024::*;

fn main() {
    let reachable_states_path = Path::new(file!())
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("reachable_states.dat");
    let solution_path = Path::new(file!())
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("solution.dat");

    let reachable_states = if reachable_states_path.exists() {
        println!("Loading reachable states from {:?}.", reachable_states_path);
        let bytes = fs::read(&reachable_states_path).unwrap();
        let saved = bytes_to_node_vec(&bytes);
        println!(
            "Loaded {} reachable states from {:?}.",
            saved.len(),
            reachable_states_path
        );
        saved
    } else {
        println!("Starting tree search. This will probably take several hours.");
        let now = Instant::now();
        let reachable_states = reachable_states(SearchNode::initial());
        let elapsed = now.elapsed();
        println!("Completed tree search. It took {:?}.", elapsed);

        let bytes = node_slice_to_bytes(&reachable_states);
        fs::write(&reachable_states_path, bytes).unwrap();
        println!("Wrote solution to {:?}.", reachable_states_path);
        reachable_states
    };

    println!("Starting retrograde analysis. This will probably take several hours.");
    let now = Instant::now();
    let mut reachable_states = reachable_states;
    solve(&mut reachable_states);
    let solution = reachable_states;
    let elapsed = now.elapsed();
    println!("Completed retrograde analysis. It took {:?}.", elapsed);

    let bytes = node_slice_to_bytes(&solution);
    fs::write(&reachable_states_path, bytes).unwrap();
    println!("Wrote solution to {:?}.", solution_path);
}

fn node_slice_to_bytes(reachable_states: &[SearchNode]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(reachable_states.len() * std::mem::size_of::<u64>());
    for state in reachable_states {
        bytes.extend_from_slice(&state.0.to_le_bytes())
    }
    bytes
}

fn bytes_to_node_vec(bytes: &[u8]) -> Vec<SearchNode> {
    let mut reachable_states = Vec::with_capacity(bytes.len() / std::mem::size_of::<u64>());
    for chunk in bytes.chunks_exact(std::mem::size_of::<u64>()) {
        let mut bytes = [0; std::mem::size_of::<u64>()];
        bytes.copy_from_slice(chunk);
        reachable_states.push(SearchNode(u64::from_le_bytes(bytes)));
    }
    reachable_states
}
