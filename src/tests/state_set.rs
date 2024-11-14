use super::*;

use std::collections::HashSet;

use rand::Rng;

// pub fn random_bin0_pair(rng: &mut impl Rng) -> (Bin0, HashSet<u8>) {
//     let mut bin0 = Bin0::empty();
//     let mut reference = HashSet::default();

//     let size: u8 = rng.gen();
//     for _ in 0..size {
//         let n: u8 = rng.gen();
//         bin0.insert(n);
//         reference.insert(n);
//     }

//     (bin0, reference)
// }

// pub fn random_bin1_pair(rng: &mut impl Rng) -> (Bin1, HashSet<[u8; 2]>) {
//     let mut bin1 = Bin1::empty();
//     let mut reference = HashSet::default();

//     let size: u8 = rng.gen();
//     for _ in 0..size {
//         let le_0: u8 = rng.gen();
//         let le_1: u8 = rng.gen();

//         let bin0 = {
//             let n = le_1;
//             if let Some(bin0) = &mut bin1[n] {
//                 &mut *bin0
//             } else {
//                 bin1[n] = Some(Box::new(Bin0::empty()));
//                 bin1[n]
//                     .as_mut()
//                     .expect("bin1[n] should not be None since it was just set to Some")
//             }
//         };
//         bin0.insert(le_0);

//         reference.insert([le_0, le_1]);
//     }

//     (bin1, reference)
// }
