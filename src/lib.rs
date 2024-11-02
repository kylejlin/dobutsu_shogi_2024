pub const PLY_LIMIT: u8 = 200;

pub fn calculate() -> SolutionMap {
    let mut solution_cache = SolutionCache::new();

    let mut stack: Vec<SearchNode> = Vec::with_capacity(PLY_LIMIT as usize);
    stack.push(SearchNode::initial());

    loop {
        let last_node = stack.last().unwrap().clone();
        let explorer_index = last_node.clone().explorer_index();

        if explorer_index == 0 {
            stack.pop();

            let solution: Solution = last_node.into();
            solution_cache.set(solution.clone());

            if stack.is_empty() {
                break;
            }

            stack.last_mut().unwrap().record_solution(solution);

            continue;
        }

        let last_node = stack.last_mut().unwrap();
        let new_quasinode = last_node.explore(explorer_index);

        if new_quasinode.clone().is_terminal() {
            last_node.record_solution(new_quasinode.into());
            continue;
        }

        let new_node: SearchNode = new_quasinode.into();

        if let Some(solution) = solution_cache.get(new_node.clone()) {
            last_node.record_solution(solution);
            continue;
        }

        stack.push(new_node);
    }

    solution_cache.into()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SolutionMap {
    raw: Vec<Solution>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Solution(pub u64);

// We could easily make this `Copy`,
// but we intentionally choose not to.
// This is to prevent unintended copying,
// since there are times we want to mutate a `SearchNode`.
// in-place.
#[derive(Clone, Debug, PartialEq, Eq)]
struct SearchNode(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SearchQuasinode(pub u64);

#[derive(Clone, Debug, PartialEq, Eq)]
struct SolutionCache {
    raw: [Option<
        Box<CacheBin<CacheBin<CacheBin<CacheBin<CacheBin<[OptionalCachedEvaluation; 16]>>>>>>,
    >; 256 * 256],
}

type CacheBin<T> = [Option<Box<T>>; 16];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct OptionalCachedEvaluation(i16);

impl SearchNode {
    const fn initial() -> SearchNode {
        const fn ascending(a: u64, b: u64) -> (u64, u64) {
            if a <= b {
                (a, b)
            } else {
                (b, a)
            }
        }

        let active_chick: u64 = 0b0_01_01_0;
        let passive_chick: u64 = 0b1_10_01_0;
        let (chick0, chick1) = ascending(active_chick, passive_chick);

        let active_elephant: u64 = 0b0_00_00;
        let passive_elephant: u64 = 0b1_11_10;
        let (elephant0, elephant1) = ascending(active_elephant, passive_elephant);

        let active_giraffe: u64 = 0b0_00_10;
        let passive_giraffe: u64 = 0b1_11_00;
        let (giraffe0, giraffe1) = ascending(active_giraffe, passive_giraffe);

        let active_lion: u64 = 0b00_01;
        let passive_lion: u64 = 0b11_01;

        let ply_count: u64 = 0;

        let lowest_unexplored_action: u64 = 0;

        let best_discovered_evaluation: u64 = NEGATIVE_200_I9;

        SearchNode(
            (chick0 << (0 + 9 + 7 + 8 + 4 + 4 + 5 + 5 + 5 + 5 + 6))
                | (chick1 << (0 + 9 + 7 + 8 + 4 + 4 + 5 + 5 + 5 + 5))
                | (elephant0 << (0 + 9 + 7 + 8 + 4 + 4 + 5 + 5 + 5))
                | (elephant1 << (0 + 9 + 7 + 8 + 4 + 4 + 5 + 5))
                | (giraffe0 << (0 + 9 + 7 + 8 + 4 + 4 + 5))
                | (giraffe1 << (0 + 9 + 7 + 8 + 4 + 4))
                | (active_lion << (0 + 9 + 7 + 8 + 4))
                | (passive_lion << (0 + 9 + 7 + 8))
                | (ply_count << (0 + 9 + 7))
                | (lowest_unexplored_action << (0 + 9))
                | (best_discovered_evaluation << 0),
        )
    }

    fn record_solution(&mut self, solution: Solution) {
        let incumbent_score = i16::from_zero_padded_i9(self.0 & 0b1_1111_1111);

        // We need to invert the solution's score, since the solution is from one ply in the future.
        // A win for the next ply's active player
        // is a loss for the current ply's active player, and vice-versa.
        // Therefore, we must invert.
        let challenger_score = -i16::from_zero_padded_i9(solution.0 & 0b1_1111_1111);

        if challenger_score > incumbent_score {
            self.0 = (self.0 & !0b1_1111_1111) | challenger_score.into_zero_padded_i9_unchecked();
        }
    }

    fn explorer_index(self) -> usize {
        return ((self.0 >> 9) & 0b111_1111) as usize;
    }

    fn explore(&mut self, explorer_index: usize) -> SearchQuasinode {
        EXPLORERS[explorer_index](self)
    }
}

impl SearchQuasinode {
    fn is_terminal(self) -> bool {
        const TERMINAL_MAGIC_NUMBER_MASK: u64 = 0b111_1111 << 9;
        (self.0 & TERMINAL_MAGIC_NUMBER_MASK) == TERMINAL_MAGIC_NUMBER_MASK
    }
}

impl From<SearchNode> for Solution {
    fn from(node: SearchNode) -> Self {
        Solution(node.0)
    }
}

impl From<SearchQuasinode> for Solution {
    fn from(quasinode: SearchQuasinode) -> Self {
        Solution(quasinode.0)
    }
}

impl From<SearchQuasinode> for SearchNode {
    fn from(quasinode: SearchQuasinode) -> Self {
        SearchNode(quasinode.0)
    }
}

impl SolutionCache {
    fn new() -> SolutionCache {
        let empty: Option<
            Box<CacheBin<CacheBin<CacheBin<CacheBin<CacheBin<[OptionalCachedEvaluation; 16]>>>>>>,
        > = Default::default();

        let mut v = Vec::with_capacity(256 * 256);

        for _ in 0..256 * 256 {
            v.push(empty.clone());
        }

        SolutionCache {
            raw: v.try_into().unwrap(),
        }
    }

    fn get(&self, node: SearchNode) -> Option<Solution> {
        let bin0 = &self.raw[(node.0 >> 48) as usize].as_ref()?;
        let bin1 = bin0[((node.0 >> (48 - 1 * 4)) & 0b1111) as usize].as_ref()?;
        let bin2 = &bin1[((node.0 >> (48 - 2 * 4)) & 0b1111) as usize].as_ref()?;
        let bin3 = &bin2[((node.0 >> (48 - 3 * 4)) & 0b1111) as usize].as_ref()?;
        let bin4 = &bin3[((node.0 >> (48 - 4 * 4)) & 0b1111) as usize].as_ref()?;
        let bin5 = &bin4[((node.0 >> (48 - 5 * 4)) & 0b1111) as usize].as_ref()?;
        let raw = bin5[((node.0 >> (48 - 6 * 4)) & 0b1111) as usize].into_zero_padded_i9()?;

        let left = node.0 & 0xFFFF_FFFF_FF00_0000;
        let right = raw;
        Some(Solution(left | right))
    }

    fn set(&mut self, solution: Solution) {
        let bin0 =
            (&mut self.raw[(solution.0 >> 48) as usize]).get_or_insert_with(Default::default);
        let bin1 = (&mut bin0[((solution.0 >> (48 - 1 * 4)) & 0b1111) as usize])
            .get_or_insert_with(Default::default);
        let bin2 = (&mut bin1[((solution.0 >> (48 - 2 * 4)) & 0b1111) as usize])
            .get_or_insert_with(Default::default);
        let bin3 = (&mut bin2[((solution.0 >> (48 - 3 * 4)) & 0b1111) as usize])
            .get_or_insert_with(Default::default);
        let bin4 = (&mut bin3[((solution.0 >> (48 - 4 * 4)) & 0b1111) as usize])
            .get_or_insert_with(Default::default);
        let bin5 = (&mut bin4[((solution.0 >> (48 - 5 * 4)) & 0b1111) as usize])
            .get_or_insert_with(Default::default);
        let raw = &mut bin5[((solution.0 >> (48 - 6 * 4)) & 0b1111) as usize];

        *raw = OptionalCachedEvaluation::from_zero_padded_i9(solution.0 & 0b1_1111_1111);
    }
}

impl From<SolutionCache> for SolutionMap {
    fn from(_cache: SolutionCache) -> Self {
        todo!()
    }
}

impl OptionalCachedEvaluation {
    const NONE: Self = OptionalCachedEvaluation(i16::MIN);

    fn into_zero_padded_i9(self) -> Option<u64> {
        if self == Self::NONE {
            return None;
        }

        Some(self.0.into_zero_padded_i9_unchecked())
    }
}

trait FromZeroPaddedI9<T> {
    fn from_zero_padded_i9(value: T) -> Self;
}

impl FromZeroPaddedI9<u64> for OptionalCachedEvaluation {
    fn from_zero_padded_i9(value: u64) -> OptionalCachedEvaluation {
        OptionalCachedEvaluation(i16::from_zero_padded_i9(value))
    }
}

impl FromZeroPaddedI9<u64> for i16 {
    fn from_zero_padded_i9(value: u64) -> i16 {
        // Handle negative values
        if (value & (1 << 9)) != 0 {
            const C: i16 = -(1 << 8);
            let v8 = (value & 0b1111_1111) as i16;
            return C + v8;
        }

        value as i16
    }
}

trait IntoZeroPaddedI9Unchecked<T> {
    /// If `self` does not fit into a 9-bit
    /// two's complement signed integer,
    /// then the behavior is undefined.
    fn into_zero_padded_i9_unchecked(self) -> T;
}

impl IntoZeroPaddedI9Unchecked<u64> for i16 {
    fn into_zero_padded_i9_unchecked(self) -> u64 {
        if self < 0 {
            return ((1 << 9) + self) as u64;
        }

        self as u64
    }
}

impl Default for OptionalCachedEvaluation {
    fn default() -> Self {
        Self::NONE
    }
}

/// -200 in 9-bit two's complement, left-padded with zeros
/// to fill the 64-bit integer.
const NEGATIVE_200_I9: u64 = 0b100111000;

const EXPLORERS: [fn(&mut SearchNode) -> SearchQuasinode; 128] = [todo_dummy; 128];

fn todo_dummy(_node: &mut SearchNode) -> SearchQuasinode {
    // TODO: Make sure to update the lowest unexplored action field.

    // TODO: Make sure to sayu-normalize the quasinode.

    todo!()
}
