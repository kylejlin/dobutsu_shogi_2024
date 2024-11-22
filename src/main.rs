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
    let pruned_tree_path = Path::new(file!())
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("pruned_tree.dat");
    let best_child_map_path = Path::new(file!())
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("best_child_map.dat");

    let solution = load_or_compute_solution(&solution_path, &reachable_states_path);
    let best_child_map = load_or_compute_best_child_map(&solution, &best_child_map_path);

    let mut input_buffer = String::with_capacity(256);

    println!("Tree inspector ready. Type \"launch\" to launch, or \"prune\" to prune.");
    println!("Launching will clear the console, so be sure to save any important information.");
    loop {
        input_buffer.clear();
        std::io::stdin().read_line(&mut input_buffer).unwrap();

        let trimmed_input = input_buffer.trim();

        if trimmed_input == "launch" {
            launch_tree_inspector(&solution);
            break;
        }

        if trimmed_input == "prune" {
            prune(&best_child_map, &pruned_tree_path);
            break;
        }

        println!("Invalid command. Type \"launch\" to launch, or \"prune\" to prune.");
    }
}

fn launch_tree_inspector(solution: &[SearchNode]) {
    let mut input_buffer = String::with_capacity(256);

    let mut history = vec![correct_nonstate_fields(SearchNode::initial(), solution)];

    loop {
        clear_console();

        let top = *history.last().unwrap();
        println!("----------------------------------------------------------------");
        println!("Current state:\n{}", top.pretty());
        match best_child_index(top, &solution) {
            Some(i) => println!("Best child index: {i}.",),
            None => println!("Best child index: None (node is terminal)."),
        }
        println!(
            "Children: {}",
            top.children()
                .into_iter()
                .map(|child| correct_nonstate_fields(child, &solution))
                .collect::<Vec<_>>()
                .pretty()
        );
        println!("Enter a command: ");
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
                    history.push(correct_nonstate_fields(children[index], &solution));
                }
            }
        }
    }
}

fn prune(best_child_map: &StateMap<SearchNode>, pruned_tree_path: &Path) -> StateSet {
    let initial_state = SearchNode(SearchNode::initial().state());

    let gote_optimal_start_time = Instant::now();
    let mut prev_time = gote_optimal_start_time;
    let mut countup = 0;
    let mut checkpoints = 0;
    const CHECKPOINT_SIZE: u64 = 1_000_000;

    let gote_optimal = prune::prune_assuming_one_player_plays_optimally(
        initial_state,
        Player::Gote,
        best_child_map,
        |_| {
            countup += 1;

            if countup >= CHECKPOINT_SIZE {
                countup %= CHECKPOINT_SIZE;
                checkpoints += 1;
                println!(
                    "Pruned {checkpoints} checkpoints (gote-optimal). Duration: {:?}",
                    prev_time.elapsed()
                );
                prev_time = Instant::now();
            }
        },
    );
    let gote_optimal_duration = gote_optimal_start_time.elapsed();
    println!(
        "Completed pruning the gote-optimal tree. It took {} dequeues and {:?}.",
        checkpoints * CHECKPOINT_SIZE + countup,
        gote_optimal_duration,
    );

    let sente_optimal_start_time = Instant::now();
    prev_time = sente_optimal_start_time;

    let sente_optimal = prune::prune_assuming_one_player_plays_optimally(
        initial_state,
        Player::Sente,
        best_child_map,
        |_| {
            countup += 1;

            if countup >= CHECKPOINT_SIZE {
                countup %= CHECKPOINT_SIZE;
                checkpoints += 1;
                println!(
                    "Pruned {checkpoints} checkpoints (sente-optimal). Duration: {:?}",
                    prev_time.elapsed()
                );
                prev_time = Instant::now();
            }
        },
    );
    let sente_optimal_duration = sente_optimal_start_time.elapsed();
    println!(
        "Completed pruning the sente-optimal tree. It took {} dequeues and {:?}.",
        checkpoints * CHECKPOINT_SIZE + countup,
        sente_optimal_duration,
    );

    println!(
        "Completed pruning both trees. It took {:?}.",
        sente_optimal_duration + gote_optimal_duration
    );

    let combined = sente_optimal.union(&gote_optimal);
    let combined_vec = combined.to_sorted_vec();
    let bytes = node_slice_to_bytes(&combined_vec);
    fs::write(&pruned_tree_path, bytes).unwrap();
    println!(
        "Wrote pruned tree ({} nodes) to {:?}.",
        combined_vec.len(),
        pruned_tree_path
    );
    combined
}

fn load_or_compute_best_child_map(
    solution: &[SearchNode],
    best_child_map_path: &Path,
) -> StateMap<SearchNode> {
    if best_child_map_path.exists() {
        println!("Loading best child map from {:?}.", best_child_map_path);
        let bytes = fs::read(&best_child_map_path).unwrap();
        let pairs = bytes_to_node_pair_vec(&bytes);
        let mut saved = StateMap::empty();
        for (parent, child) in pairs {
            saved.add(parent, child);
        }
        println!("Loaded best child map from {:?}.", best_child_map_path);
        saved
    } else {
        println!("Computing best child map. This will probably take a while.");

        let start_time = Instant::now();
        let mut prev_time = start_time;
        let mut countup = 0;
        let mut checkpoints = 0;
        const CHECKPOINT_SIZE: u64 = 1_000_000;

        let best_child_map = best_child_map(solution, |_| {
            countup += 1;

            if countup >= CHECKPOINT_SIZE {
                countup %= CHECKPOINT_SIZE;
                checkpoints += 1;
                println!(
                    "Found best children for {checkpoints} checkpoints. Duration: {:?}",
                    prev_time.elapsed()
                );
                prev_time = Instant::now();
            }
        });
        println!(
            "Computed best child map for {} nodes. It took {:?}.",
            checkpoints * CHECKPOINT_SIZE + countup,
            start_time.elapsed()
        );

        let bytes = node_pair_slice_to_bytes(&best_child_map.to_sorted_vec());
        fs::write(&best_child_map_path, bytes).unwrap();
        println!("Wrote best child map to {:?}.", best_child_map_path);
        best_child_map
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
        let mut progress = Progress::default();
        const CHECKPOINT_SIZE: u64 = 1_000_000;
        solve(&mut reachable_states, &mut progress, |current_progress| {
            countup += 1;

            if countup >= CHECKPOINT_SIZE {
                countup %= CHECKPOINT_SIZE;
                checkpoints += 1;
                println!(
                    "Backtracked {checkpoints} checkpoints. Duration: {:?}",
                    prev_time.elapsed()
                );
                println!("Progress:\n{:#?}", current_progress);
                println!();
                prev_time = Instant::now();

                true
            } else {
                false
            }
        });
        let solution = reachable_states;
        println!(
            "Completed retrograde analysis on {} states. It took {:?}.",
            checkpoints * CHECKPOINT_SIZE + countup,
            start_time.elapsed()
        );

        let bytes = node_slice_to_bytes(&solution);
        fs::write(&solution_path, bytes).unwrap();
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

fn node_pair_slice_to_bytes(reachable_states: &[(SearchNode, SearchNode)]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(reachable_states.len() * 2 * std::mem::size_of::<u64>());
    for (parent, child) in reachable_states {
        bytes.extend_from_slice(&parent.0.to_le_bytes());
        bytes.extend_from_slice(&child.0.to_le_bytes());
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

fn bytes_to_node_pair_vec(bytes: &[u8]) -> Vec<(SearchNode, SearchNode)> {
    let mut reachable_states = Vec::with_capacity(bytes.len() / (2 * std::mem::size_of::<u64>()));
    for chunk in bytes.chunks_exact(2 * std::mem::size_of::<u64>()) {
        let mut parent_bytes = [0; std::mem::size_of::<u64>()];
        parent_bytes.copy_from_slice(&chunk[..std::mem::size_of::<u64>()]);
        let mut child_bytes = [0; std::mem::size_of::<u64>()];
        child_bytes.copy_from_slice(&chunk[std::mem::size_of::<u64>()..]);
        reachable_states.push((
            SearchNode(u64::from_le_bytes(parent_bytes)),
            SearchNode(u64::from_le_bytes(child_bytes)),
        ));
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

fn correct_nonstate_fields(target: SearchNode, solution: &[SearchNode]) -> SearchNode {
    let target_state = target.state();
    let index = solution
        .binary_search_by(|other| other.state().cmp(&target_state))
        .expect("Could not find node in solution.");
    solution[index]
}

fn best_child_index(parent: SearchNode, solution: &[SearchNode]) -> Option<usize> {
    let children = parent.children();
    if children.is_empty() {
        return None;
    }

    let mut best_index = 0;
    let mut best_outcome: Outcome = get_node_outcome(children[0], solution).unwrap_or(Outcome(0));

    for (i, child) in children.iter().enumerate().skip(1) {
        let outcome = get_node_outcome(*child, solution).unwrap_or(Outcome(0));
        // We invert perspectives, since child nodes represent the opponent's turn.
        // Therefore, lower scores are better.
        if outcome < best_outcome {
            best_index = i;
            best_outcome = outcome;
        }
    }

    Some(best_index)
}

fn get_node_outcome(target: SearchNode, solution: &[SearchNode]) -> Option<Outcome> {
    correct_nonstate_fields(target, solution).best_outcome()
}

fn clear_console() {
    print!("{esc}c", esc = 27 as char);
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
}
