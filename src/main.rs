use std::fs;
use std::path::Path;
use std::time::Instant;

use dobutsu_shogi_2024::*;
use pretty::IntoPretty;

#[derive(Clone, Copy, Debug)]
enum Command {
    Help,
    Parent,
    Child(usize),
}

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
    let mut history = vec![SearchNode::initial()];
    let mut input_buffer = String::with_capacity(256);

    loop {
        let top = *history.last().unwrap();
        println!("----------------------------------------------------------------");
        println!("Current state: {}", top.pretty());
        println!("Children: {}", top.children().pretty());
        match best_child_index(top, &solution) {
            Some(i) => println!("Best child index: {i}.",),
            None => println!("Best child index: None (node is terminal)."),
        }
        print!("Enter a command: ");
        input_buffer.clear();
        std::io::stdin().read_line(&mut input_buffer).unwrap();

        let Ok(command) = Command::parse(&input_buffer) else {
            println!("Invalid command. Type \"help\" for a list of commands.");
            continue;
        };

        match command {
            Command::Help => {
                println!("Commands:");
                println!("    help: Print this help message.");
                println!("    parent: Go to the parent state.");
                println!("    child <index>: Go to the child at the given index.");
            }

            Command::Parent => {
                if history.len() == 1 {
                    println!("Already at the initial state.");
                } else {
                    history.pop();
                }
            }

            Command::Child(index) => {
                let children = history.last().unwrap().children();
                if index >= children.len() {
                    println!("Invalid child index.");
                } else {
                    history.push(children[index]);
                }
            }
        }
    }
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

impl Command {
    fn parse(input: &str) -> Result<Self, ()> {
        let input = input.trim();
        match input {
            "help" => Ok(Self::Help),
            "parent" => Ok(Self::Parent),
            _ => {
                if input.starts_with("child ") {
                    let child_index: usize =
                        input["child ".len()..].parse().map_err(std::mem::drop)?;
                    Ok(Self::Child(child_index))
                } else {
                    Err(())
                }
            }
        }
    }
}

fn best_child_index(parent: SearchNode, solution: &[SearchNode]) -> Option<usize> {
    let children = parent.children();
    if children.is_empty() {
        return None;
    }

    let mut best_index = 0;
    let mut best_outcome = get_node_outcome(children[0], solution);

    for (i, child) in children.iter().enumerate().skip(1) {
        let outcome = get_node_outcome(*child, solution);
        // We invert perspectives, since child nodes represent the opponent's turn.
        // Therefore, lower scores are better.
        if outcome < best_outcome {
            best_index = i;
            best_outcome = outcome;
        }
    }

    Some(best_index)
}

fn get_node_outcome(target: SearchNode, solution: &[SearchNode]) -> Outcome {
    let target_state = target.state();
    let index = solution
        .binary_search_by(|other| other.state().cmp(&target_state))
        .expect("Could not find node in solution.");
    let node = solution[index];
    node.best_known_outcome()
}
