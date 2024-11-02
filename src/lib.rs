use std::num::NonZeroU8;

pub const PLY_LIMIT: u8 = 200;

pub fn calculate() -> CompactSolutionMap {
    let mut solution_cache = SolutionCache::new();

    let mut stack: Vec<SearchNode> = Vec::with_capacity(PLY_LIMIT as usize);
    stack.push(SearchNode::initial());

    loop {
        let last_node = stack.last().unwrap().clone();

        let action = match last_node.clone().next_action() {
            Ok(action) => action,

            Err(solution) => {
                stack.pop();

                solution_cache.set(solution.clone());

                if stack.is_empty() {
                    break;
                }

                stack.last_mut().unwrap().record_solution(solution);

                continue;
            }
        };

        let last_node = stack.last_mut().unwrap();
        let Some(new_node) = last_node.explore(action) else {
            continue;
        };

        if let Some(solution) = solution_cache.get(new_node.clone()) {
            last_node.record_solution(solution);
            continue;
        }

        stack.push(new_node);
    }

    solution_cache.into()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompactSolutionMap {
    raw: Vec<Solution>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Solution(pub u64);

// We could easily make this `Copy`,
// but we intentionally choose not to.
// This is to prevent unintended copying,
// since there are times we want to mutate a `SearchNode`.
// in-place.
#[derive(Clone, Debug, PartialEq, Eq)]
struct SearchNode(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TimelessState(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TimelessStateToNodeConverter(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Action(NonZeroU8);

#[derive(Clone, Debug, PartialEq, Eq)]
struct SolutionCache {
    raw: [Option<
        Box<CacheBin<CacheBin<CacheBin<CacheBin<CacheBin<[OptionalCachedEvaluation; 16]>>>>>>,
    >; 256 * 256],
}

type CacheBin<T> = [Option<Box<T>>; 16];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct OptionalCachedEvaluation(i16);

impl CompactSolutionMap {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.raw.len() * 8);

        for solution in &self.raw {
            let solution = solution.0.to_le_bytes();
            bytes.extend_from_slice(&solution);
        }

        bytes
    }
}

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

    fn next_action(self) -> Result<Action, Solution> {
        let raw = ((self.0 >> 9) & 0b111_1111) as u8;
        let Some(raw) = NonZeroU8::new(raw) else {
            return Err(Solution(self.0));
        };
        Ok(Action(raw))
    }

    fn explore(&mut self, action: Action) -> Option<SearchNode> {
        let (new_timeless_state, next_action) =
            ACTION_HANDLERS[action.0.get() as usize](self.clone());

        self.set_next_action(next_action);

        Some(new_timeless_state?.into_node(self.clone().ply_count() + 1))
    }

    fn set_next_action(&mut self, next_action: Option<Action>) {
        let raw: u64 = match next_action {
            None => 0,
            Some(n) => n.0.get() as u64,
        };

        self.0 = (self.0 & !(0b111_1111 << 9)) | (raw << 9);
    }

    fn ply_count(self) -> u64 {
        (self.0 >> (0 + 9 + 7)) & 0b1111_1111
    }
}

impl TimelessState {
    fn into_node(self, ply_count: u64) -> SearchNode {
        TimelessStateToNodeConverter(self.0).into_node(ply_count)
    }
}

impl TimelessStateToNodeConverter {
    fn into_node(self, ply_count: u64) -> SearchNode {
        let raw = self
            .set_ply_count(ply_count)
            .horizontally_normalize()
            .init_best_discovered_outcome_and_next_action()
            .0;
        SearchNode(raw)
    }

    fn set_ply_count(self, _ply_count: u64) -> Self {
        todo!()
    }

    fn horizontally_normalize(self) -> Self {
        let flipped = self.horizontally_flip();

        if flipped.0 < self.0 {
            return flipped;
        }

        self
    }

    fn horizontally_flip(self) -> Self {
        todo!()
    }

    /// If the this is terminal, then we set the best discovered outcome
    /// to the outcome of the game, and we set the next action to `None`.
    /// Otherwise, we set the best discovered outcome to `-200`,
    /// and we set the next action `to Action(0b001_0000)`.
    fn init_best_discovered_outcome_and_next_action(self) -> Self {
        todo!()
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

impl From<SolutionCache> for CompactSolutionMap {
    fn from(cache: SolutionCache) -> Self {
        let mut raw = Vec::new();

        cache.write(&mut raw);

        raw.sort_unstable();

        CompactSolutionMap { raw }
    }
}

impl SolutionCache {
    fn write(&self, out: &mut Vec<Solution>) {
        for (i0, bin0) in self.raw.iter().enumerate() {
            let Some(bin0) = bin0 else {
                continue;
            };
            let prefix = (i0 as u64) << 48;
            self.write_bin0(prefix, bin0, out);
        }
    }

    fn write_bin0(
        &self,
        prefix: u64,
        bin0: &Box<
            CacheBin<CacheBin<CacheBin<CacheBin<CacheBin<[OptionalCachedEvaluation; 16]>>>>>,
        >,
        out: &mut Vec<Solution>,
    ) {
        for (i1, bin1) in bin0.iter().enumerate() {
            let Some(bin1) = bin1 else {
                continue;
            };
            let prefix = prefix | ((i1 as u64) << (48 - 1 * 4));
            self.write_bin1(prefix, bin1, out);
        }
    }

    fn write_bin1(
        &self,
        prefix: u64,
        bin1: &Box<CacheBin<CacheBin<CacheBin<CacheBin<[OptionalCachedEvaluation; 16]>>>>>,
        out: &mut Vec<Solution>,
    ) {
        for (i2, bin2) in bin1.iter().enumerate() {
            let Some(bin2) = bin2 else {
                continue;
            };
            let prefix = prefix | ((i2 as u64) << (48 - 2 * 4));
            self.write_bin2(prefix, bin2, out);
        }
    }

    fn write_bin2(
        &self,
        prefix: u64,
        bin2: &Box<CacheBin<CacheBin<CacheBin<[OptionalCachedEvaluation; 16]>>>>,
        out: &mut Vec<Solution>,
    ) {
        for (i3, bin3) in bin2.iter().enumerate() {
            let Some(bin3) = bin3 else {
                continue;
            };
            let prefix = prefix | ((i3 as u64) << (48 - 3 * 4));
            self.write_bin3(prefix, bin3, out);
        }
    }

    fn write_bin3(
        &self,
        prefix: u64,
        bin3: &Box<CacheBin<CacheBin<[OptionalCachedEvaluation; 16]>>>,
        out: &mut Vec<Solution>,
    ) {
        for (i4, bin4) in bin3.iter().enumerate() {
            let Some(bin4) = bin4 else {
                continue;
            };
            let prefix = prefix | ((i4 as u64) << (48 - 4 * 4));
            self.write_bin4(prefix, bin4, out);
        }
    }

    fn write_bin4(
        &self,
        prefix: u64,
        bin4: &Box<CacheBin<[OptionalCachedEvaluation; 16]>>,
        out: &mut Vec<Solution>,
    ) {
        for (i5, bin5) in bin4.iter().enumerate() {
            let Some(bin5) = bin5 else {
                continue;
            };
            let prefix = prefix | ((i5 as u64) << (48 - 5 * 4));
            self.write_bin5(prefix, bin5, out);
        }
    }

    fn write_bin5(
        &self,
        prefix: u64,
        bin5: &Box<[OptionalCachedEvaluation; 16]>,
        out: &mut Vec<Solution>,
    ) {
        for (i6, raw) in bin5.iter().enumerate() {
            let Some(outcome_score) = raw.into_zero_padded_i9() else {
                continue;
            };
            let solution = prefix | ((i6 as u64) << (48 - 6 * 4)) | outcome_score;
            out.push(Solution(solution));
        }
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

/// An action handler will return the result of applying an action
/// to the input state, assuming the action is legal.
/// If the action is illegal, then the handler will return `None`
/// instead of the resulting timeless state.
/// Regardless of the legality of the action,
/// the handler will return an `Option<Action>`
/// that represents the next (possibly illegal) action to be explored.
const ACTION_HANDLERS: [fn(SearchNode) -> (Option<TimelessState>, Option<Action>); 128] = [
    // illegal: 0b000_0000 to 0b000_1111
    handle_bad_action,
    handle_bad_action,
    handle_bad_action,
    handle_bad_action,
    handle_bad_action,
    handle_bad_action,
    handle_bad_action,
    handle_bad_action,
    handle_bad_action,
    handle_bad_action,
    handle_bad_action,
    handle_bad_action,
    handle_bad_action,
    handle_bad_action,
    handle_bad_action,
    handle_bad_action,
    // chick0: 0b001_0000 to 0b001_1111
    handle_chick0_row00_col00,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    // chick1: 0b010_0000 to 0b010_1111
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    // elephant0: 0b011_0000 to 0b011_1111
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    // elephant1: 0b100_0000 to 0b100_1111
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    // giraffe0: 0b101_0000 to 0b101_1111
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    // giraffe1: 0b110_0000 to 0b110_1111
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    // activeLion: 0b111_0000 to 0b111_1111
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    todo_dummy,
];

fn handle_chick0_row00_col00(node: SearchNode) -> (Option<TimelessState>, Option<Action>) {
    let node = node.0;
    const CHICK_0_ALLEGIANCE_MASK: u64 = 1 << (0 + 9 + 7 + 8 + 4 + 4 + 5 + 5 + 5 + 5 + 6 + 5);

    // If chick0 is allegiant to the passive player, we cannot move it.
    if node & CHICK_0_ALLEGIANCE_MASK != 0 {
        const C: NonZeroU8 = unsafe { NonZeroU8::new_unchecked(0b010_0000) };
        return (None, Some(Action(C)));
    }

    todo!()
}

fn handle_bad_action(_: SearchNode) -> (Option<TimelessState>, Option<Action>) {
    panic!("Illegal action");
}

fn todo_dummy(_node: SearchNode) -> (Option<TimelessState>, Option<Action>) {
    todo!()
}
