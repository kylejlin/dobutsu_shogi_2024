use std::fs::{self, File};
use std::io::{Read, Write};
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

type BestChildMap = StateMap<StateAndStats>;

fn main() {
    let solution_path = Path::new(file!())
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("solution.dat");
    let simple_db_path = Path::new(file!())
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("db");

    let solution = load_or_compute_solution(&solution_path);

    let mut input_buffer = String::with_capacity(256);

    println!("Tree inspector ready. Type \"launch\" to launch or \"simpledb\" to create a simple best-child database.");
    println!("Launching will clear the console, so be sure to save any important information.");
    loop {
        input_buffer.clear();
        std::io::stdin().read_line(&mut input_buffer).unwrap();

        let trimmed_input = input_buffer.trim();

        if trimmed_input == "launch" {
            launch_tree_inspector(&solution);
            break;
        }

        if trimmed_input == "simpledb" {
            create_simple_db(&solution, &simple_db_path);
            break;
        }

        println!("Invalid command. Type \"launch\" to launch or \"simpledb\" to create a simple best-child database.");
    }
}

fn launch_tree_inspector(solution: &BestChildMap) {
    let mut input_buffer = String::with_capacity(256);

    let mut history = vec![(State::initial(), State::initial().best_outcome(solution))];

    loop {
        clear_console();

        let (top_state, top_outcome) = *history.last().unwrap();
        println!("----------------------------------------------------------------");
        println!(
            "Current state:\n{}",
            top_state
                .with_stats(StateStats::new(top_outcome, 127))
                .pretty()
        );
        match top_state.best_child_index(&solution) {
            Some(i) => println!("Best child index: {i}.",),
            None => println!("Best child index: None (node is terminal)."),
        }
        println!(
            "Children: {}",
            top_state
                .children()
                .into_iter()
                .map(|child| child.with_stats(StateStats::new(child.best_outcome(solution), 127)))
                .collect::<Vec<StateAndStats>>()
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
                let children = history.last().unwrap().0.children();
                if index >= children.len() {
                    println!("Invalid child index.");
                } else {
                    let child = children[index];
                    history.push((child, child.best_outcome(solution)));
                }
            }
        }
    }
}

trait StateHelperMethods {
    fn best_child_index(self, solution: &BestChildMap) -> Option<usize>;
    fn best_outcome(self, solution: &BestChildMap) -> Outcome;
}

impl StateHelperMethods for State {
    fn best_child_index(self, solution: &BestChildMap) -> Option<usize> {
        let best_child = solution.get(self);
        if best_child.is_null() {
            return None;
        }

        let best_child = best_child.state();

        let mut i = 0;
        let mut best_child_index = None;
        self.visit_children(|child| {
            if child == best_child {
                best_child_index = Some(i);
            }

            i += 1;
        });

        best_child_index
    }

    fn best_outcome(self, solution: &BestChildMap) -> Outcome {
        match self.terminality() {
            Terminality::Loss => return Outcome::loss_in(0),
            Terminality::Win => return Outcome::win_in(0),
            Terminality::Nonterminal => {}
        }

        let child = solution.get(self);
        assert!(
            !child.is_null(),
            "Cannot find best child of non-terminal state.\n\nSTATE:\n\n{}",
            self.pretty()
        );

        if child.stats().required_child_report_count() > 0 {
            return Outcome::DRAW;
        }

        child
            .stats()
            .best_outcome()
            .unwrap()
            .invert()
            .delay_by_one()
    }
}

fn create_simple_db(solution: &BestChildMap, simple_db_path: &Path) {
    if simple_db_path.exists() {
        println!(
            "Simple best-child database already exists at {:?}.",
            simple_db_path
        );
        println!("Aborting.");
        return;
    }

    println!(
        "Creating simple best-child database at {:?}.",
        simple_db_path
    );

    let start_time = Instant::now();
    let mut prev_time = start_time;
    let mut countup = 0;
    let mut checkpoints = 0;
    const CHECKPOINT_SIZE: u64 = 1_000_000;

    const U64_BYTES: usize = std::mem::size_of::<u64>();
    const NODES_PER_PACKET: usize = 1024;
    let mut packet_buffer: Vec<u8> = Vec::with_capacity(U64_BYTES * NODES_PER_PACKET);
    let mut parent_of_most_recent_packet_addition: Option<State> = None;
    let mut byte_quintuplets_representing_packet_parent_shifted_state_maximums: Vec<u8> = vec![];

    fs::create_dir(&simple_db_path).unwrap();
    solution.visit_in_key_order(|parent, child| {
        packet_buffer.extend_from_slice(&child.0.to_le_bytes());
        parent_of_most_recent_packet_addition = Some(parent);

        if packet_buffer.len() == U64_BYTES * NODES_PER_PACKET {
            let parent_state_bytes = parent.0.to_le_bytes();
            let prefix = simple_db_path
                .join(format!("{:02x}", parent_state_bytes[0]))
                .join(format!("{:02x}", parent_state_bytes[1]))
                .join(format!("{:02x}", parent_state_bytes[2]))
                .join(format!("{:02x}", parent_state_bytes[3]));
            fs::create_dir_all(&prefix).unwrap();

            let file_path = prefix.join(format!("{:02x}.dat", parent_state_bytes[4]));
            fs::write(&file_path, &packet_buffer).unwrap();

            byte_quintuplets_representing_packet_parent_shifted_state_maximums
                .extend_from_slice(&parent_state_bytes[0..5]);
            packet_buffer.clear();
        }

        countup += 1;

        if countup >= CHECKPOINT_SIZE {
            countup %= CHECKPOINT_SIZE;
            checkpoints += 1;
            println!(
                "Reached {checkpoints} database checkpoints. Duration: {:?}",
                prev_time.elapsed()
            );
            prev_time = Instant::now();
        }
    });

    if let Some(parent) = parent_of_most_recent_packet_addition {
        if !packet_buffer.is_empty() {
            let parent_state_bytes = parent.0.to_le_bytes();
            let prefix = simple_db_path
                .join(format!("{:02x}", parent_state_bytes[0]))
                .join(format!("{:02x}", parent_state_bytes[1]))
                .join(format!("{:02x}", parent_state_bytes[2]))
                .join(format!("{:02x}", parent_state_bytes[3]));
            fs::create_dir_all(&prefix).unwrap();

            let file_path = prefix.join(format!("{:02x}.dat", parent_state_bytes[4]));
            fs::write(&file_path, &packet_buffer).unwrap();

            byte_quintuplets_representing_packet_parent_shifted_state_maximums
                .extend_from_slice(&parent_state_bytes[0..5]);
            packet_buffer.clear();
        }
    }

    let packet_count = byte_quintuplets_representing_packet_parent_shifted_state_maximums.len() / 5;
    fs::write(
        simple_db_path.join("maximums.dat"),
        byte_quintuplets_representing_packet_parent_shifted_state_maximums,
    )
    .unwrap();

    println!(
        "Created simple best-child database ({packet_count} packets) at {:?}. It took {:?}.",
        simple_db_path,
        start_time.elapsed()
    );
}

fn load_or_compute_solution(solution_path: &Path) -> StateMap<StateAndStats> {
    if solution_path.exists() {
        println!("Loading best child map from {:?}.", solution_path);
        let mut file = File::open(&solution_path).unwrap();
        let mut out = StateMap::empty();
        const CHECKPOINT_SIZE: usize = 10_000_000;
        const U64_BYTES: usize = std::mem::size_of::<u64>();
        let mut buffer: Box<[u8; CHECKPOINT_SIZE * 2 * U64_BYTES]> =
            Box::new([0; CHECKPOINT_SIZE * 2 * U64_BYTES]);
        let mut buffer_len = 0;
        let mut checkpoints = 0;
        let start_time = Instant::now();
        loop {
            let bytes_read = file.read(&mut buffer[buffer_len..]).unwrap();
            if bytes_read == 0 && !buffer[buffer_len..].is_empty() {
                break;
            }

            buffer_len += bytes_read;

            if buffer_len == CHECKPOINT_SIZE * 2 * U64_BYTES {
                for i in (0..buffer_len).step_by(2 * U64_BYTES) {
                    let mut parent_bytes = [0; U64_BYTES];
                    parent_bytes.copy_from_slice(&buffer[i..i + U64_BYTES]);
                    let parent = State(u64::from_le_bytes(parent_bytes));

                    let mut child_bytes = [0; U64_BYTES];
                    child_bytes.copy_from_slice(&buffer[i + U64_BYTES..i + 2 * U64_BYTES]);
                    let child = StateAndStats(u64::from_le_bytes(child_bytes));

                    out.add(parent, child);
                }

                buffer_len = 0;

                checkpoints += 1;
                println!("Loaded {checkpoints} best child checkpoints.");
            }
        }

        for i in (0..buffer_len).step_by(2 * U64_BYTES) {
            let mut parent_bytes = [0; U64_BYTES];
            parent_bytes.copy_from_slice(&buffer[i..i + U64_BYTES]);
            let parent = State(u64::from_le_bytes(parent_bytes));

            let mut child_bytes = [0; U64_BYTES];
            child_bytes.copy_from_slice(&buffer[i + U64_BYTES..i + 2 * U64_BYTES]);
            let child = StateAndStats(u64::from_le_bytes(child_bytes));

            out.add(parent, child);
        }

        println!(
            "Loaded best child map ({} states) from {:?}. It took {:?}.",
            CHECKPOINT_SIZE * checkpoints + buffer_len / (2 * U64_BYTES),
            solution_path,
            start_time.elapsed()
        );

        out
    } else {
        let state_stats = compute_state_stats(compute_reachable_states());

        println!("Computing best child map. This will probably take a while.");

        let start_time = Instant::now();
        let mut prev_time = start_time;
        let mut countup = 0;
        let mut checkpoints = 0;
        const CHECKPOINT_SIZE: u64 = 1_000_000;

        let solution = best_child_map(&state_stats, |_| {
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

        {
            let mut file = File::create(&solution_path).unwrap();
            let mut out_buffer =
                Vec::with_capacity(2 * std::mem::size_of::<u64>() * (CHECKPOINT_SIZE as usize));
            solution.visit_in_key_order(|parent, child| {
                out_buffer.extend_from_slice(&parent.0.to_le_bytes());
                out_buffer.extend_from_slice(&child.0.to_le_bytes());

                if out_buffer.len() >= 2 * std::mem::size_of::<u64>() * (CHECKPOINT_SIZE as usize) {
                    file.write_all(&out_buffer).unwrap();
                    out_buffer.clear();
                }
            });
            file.write_all(&out_buffer).unwrap();
            out_buffer.clear();
        }

        println!("Wrote best child map to {:?}.", solution_path);
        solution
    }
}

fn compute_state_stats(reachable_states: StateMap<StateStats>) -> StateMap<StateStats> {
    println!("Starting retrograde analysis. This will probably take several hours.");
    let mut reachable_states = reachable_states;
    let start_time = Instant::now();
    let mut prev_time = start_time;
    let mut countup = 0;
    let mut checkpoints = 0;
    let mut progress = Progress::default();
    const CHECKPOINT_SIZE: u64 = 1_000_000;
    compute_stats(&mut reachable_states, &mut progress, |current_progress| {
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

    solution
}

fn compute_reachable_states() -> StateMap<StateStats> {
    println!("Computing reachable states. This will probably take a while (around 45 minutes on my 2018 Macbook Pro).");
    let start_time = Instant::now();
    let mut prev_time = start_time;
    let mut countup = 0;
    let mut checkpoints = 0;
    const CHECKPOINT_SIZE: u64 = 1_000_000;
    let reachable_states = reachable_states(State::initial(), |_| {
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
        checkpoints * CHECKPOINT_SIZE + countup,
        start_time.elapsed()
    );

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

fn clear_console() {
    print!("{esc}c", esc = 27 as char);
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
}
