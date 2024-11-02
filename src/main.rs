use std::fs;
use std::path::Path;

fn main() {
    let path = Path::new(file!())
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("solution.dat");
    let solution_map = dobutsu_shogi_2024::calculate();
    let bytes = solution_map.to_bytes();
    fs::write(&path, bytes).unwrap();
}
