use std::fs;
use std::path::Path;
use std::time::Instant;

fn main() {
    println!("Starting tree search. This will probably take several hours.");
    let now = Instant::now();
    let solution_map = dobutsu_shogi_2024::calculate();
    let elapsed = now.elapsed();
    println!("Completed tree search. It took {:?}.", elapsed);

    let path = Path::new(file!())
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("solution.dat");
    let bytes = solution_map.to_bytes();
    fs::write(path, bytes).unwrap();
}
