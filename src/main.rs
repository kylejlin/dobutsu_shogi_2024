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

    let solution = load_or_compute_solution(&solution_path, &reachable_states_path);
}

fn load_or_compute_solution(solution_path: &Path, reachable_states_path: &Path) -> Vec<SearchNode> {
    if solution_path.exists() {
        println!("Loading solution from {:?}.", solution_path);
        let bytes = fs::read(&solution_path).unwrap();
        let saved = bytes_to_node_vec(&bytes);
        println!("Loaded solution from {:?}.", solution_path);
        saved
    } else {
        let reachable_states = load_or_compute_reachable_states(&reachable_states_path);

        println!("Starting retrograde analysis. This will probably take several hours.");
        let mut reachable_states = reachable_states;
        let start_time = Instant::now();
        let mut prev_time = start_time;
        let mut countup = 0;
        let mut checkpoints = 0;
        const CHECKPOINT_SIZE: u64 = 1_000_000;
        solve(&mut reachable_states, |_| {
            countup += 1;

            if countup >= CHECKPOINT_SIZE {
                countup %= CHECKPOINT_SIZE;
                checkpoints += 1;
                println!(
                    "Backtracked {checkpoints} checkpoints. Duration: {:?}",
                    prev_time.elapsed()
                );
                prev_time = Instant::now();
            }
        });
        let solution = reachable_states;
        println!(
            "Completed retrograde analysis on {} states. It took {:?}.",
            checkpoints * CHECKPOINT_SIZE + countup,
            start_time.elapsed()
        );

        let bytes = node_slice_to_bytes(&solution);
        fs::write(&reachable_states_path, bytes).unwrap();
        println!("Wrote solution to {:?}.", solution_path);
        solution
    }
}

fn load_or_compute_reachable_states(reachable_states_path: &Path) -> Vec<SearchNode> {
    if reachable_states_path.exists() {
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
        println!("Computing reachable states. This will probably take a while (around 45 minutes on my 2018 Macbook Pro).");
        let start_time = Instant::now();
        let mut prev_time = start_time;
        let mut countup = 0;
        let mut checkpoints = 0;
        const CHECKPOINT_SIZE: u64 = 1_000_000;
        let reachable_states = reachable_states(SearchNode::initial(), |_| {
            countup += 1;

            if countup >= CHECKPOINT_SIZE {
                countup %= CHECKPOINT_SIZE;
                checkpoints += 1;
                println!(
                    "Reached {checkpoints} checkpoints. Duration: {:?}",
                    prev_time.elapsed()
                );
                prev_time = Instant::now();
            }
        });
        println!(
            "Computed all {} reachable states. It took {:?}.",
            reachable_states.len(),
            start_time.elapsed()
        );

        let bytes = node_slice_to_bytes(&reachable_states);
        fs::write(&reachable_states_path, bytes).unwrap();
        println!("Wrote solution to {:?}.", reachable_states_path);
        reachable_states
    }
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
