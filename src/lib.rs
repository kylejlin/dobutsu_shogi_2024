// A note about fileds with the comment "This must be non-zero":
// I know we _could_ use a `NonZeroU64` (or another respective `NonZero*` type),
// but that would clutter the code with a bunch of unwraps,
// which hurts readability and performance.

pub const PLY_LIMIT: u8 = 200;

pub fn calculate() -> CompactSolutionMap {
    let mut solution_cache = SolutionCache::new();

    let mut stack: Vec<SearchNode> = Vec::with_capacity(PLY_LIMIT as usize);
    stack.push(SearchNode::initial());

    loop {
        let last_node = *stack.last().unwrap();

        let action = match last_node.next_action() {
            Ok(action) => action,

            Err(solution) => {
                stack.pop();

                solution_cache.set(solution);

                if stack.is_empty() {
                    break;
                }

                stack.last_mut().unwrap().record_solution(solution);

                continue;
            }
        };

        let last_node_mut = stack.last_mut().unwrap();
        let (new_parent, child) = last_node_mut.explore(action);
        *last_node_mut = new_parent;

        if child.is_none() {
            continue;
        }
        let child = child.unchecked_unwrap();

        let solution = solution_cache.get(child);
        if solution.is_some() {
            *last_node_mut = last_node_mut.record_solution(solution.unchecked_unwrap());
            continue;
        }

        stack.push(child);
    }

    solution_cache.into()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompactSolutionMap {
    raw: Vec<Solution>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Solution(
    /// This must be non-zero.
    pub u64,
);

/// An optional solution `o` represents None if and only if `o.0 == 0`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct OptionalSolution(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SearchNode(
    // This must be non-zero.
    u64,
);

/// An optional node `o` represents None if and only if `o.0 == 0`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct OptionalSearchNode(u64);

/// This is like a `SearchNode`,
/// but with the `chick0 <= chick1` invariant
/// (and all similar invariants) removed.
/// In other words, `NodeBuilder` represents a
/// possibly "corrupted" node,
/// and `SearchNode` is the subset of `NodeBuilder`
/// representing "valid" nodes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct NodeBuilder(
    /// This must be non-zero.
    u64,
);

/// An optional node builder `o` represents None if and only if `o.0 == 0`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct OptionalNodeBuilder(u64);

/// The **least** significant 7 bits are used.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Action(
    /// This must be non-zero.
    u8,
);

/// The **least** significant 7 bits are used.
/// An optional action `o` represents None if and only if `o.0 == 0`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct OptionalAction(u8);

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

    fn record_solution(self, solution: Solution) -> Self {
        let incumbent_score = i16::from_zero_padded_i9(self.0 & 0b1_1111_1111);

        // We need to invert the solution's score, since the solution is from one ply in the future.
        // A win for the next ply's active player
        // is a loss for the current ply's active player, and vice-versa.
        // Therefore, we must invert.
        let challenger_score = -i16::from_zero_padded_i9(solution.0 & 0b1_1111_1111);

        if challenger_score > incumbent_score {
            return Self(
                (self.0 & !0b1_1111_1111) | challenger_score.into_zero_padded_i9_unchecked(),
            );
        }

        self
    }

    fn next_action(self) -> Result<Action, Solution> {
        let raw = ((self.0 >> 9) & 0b111_1111) as u8;
        if raw == 0 {
            return Err(Solution(self.0));
        }
        Ok(Action(raw))
    }

    fn explore(self, action: Action) -> (Self, OptionalSearchNode) {
        let (child_builder, next_action) = ACTION_HANDLERS[action.0 as usize](self);

        let new_self = self.set_next_action(next_action);
        let child = if child_builder.is_none() {
            OptionalSearchNode::NONE
        } else {
            let built = child_builder
                .unchecked_unwrap()
                .invert_active_player()
                .increment_ply_count()
                .horizontally_normalize()
                .init_best_discovered_outcome_and_next_action();
            OptionalSearchNode(built.0)
        };
        (new_self, child)
    }

    fn set_next_action(self, next_action: OptionalAction) -> Self {
        let raw = next_action.0 as u64;
        Self((self.0 & !(0b111_1111 << 9)) | (raw << 9))
    }

    fn ply_count(self) -> u64 {
        (self.0 >> (0 + 9 + 7)) & 0b1111_1111
    }
}

impl NodeBuilder {
    fn invert_active_player(self) -> Self {
        todo!()
    }

    fn increment_ply_count(self) -> Self {
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

    fn get(&self, node: SearchNode) -> OptionalSolution {
        let Some(bin0) = &self.raw[(node.0 >> 48) as usize].as_ref() else {
            return OptionalSolution::NONE;
        };
        let Some(bin1) = bin0[((node.0 >> (48 - 1 * 4)) & 0b1111) as usize].as_ref() else {
            return OptionalSolution::NONE;
        };
        let Some(bin2) = &bin1[((node.0 >> (48 - 2 * 4)) & 0b1111) as usize].as_ref() else {
            return OptionalSolution::NONE;
        };
        let Some(bin3) = &bin2[((node.0 >> (48 - 3 * 4)) & 0b1111) as usize].as_ref() else {
            return OptionalSolution::NONE;
        };
        let Some(bin4) = &bin3[((node.0 >> (48 - 4 * 4)) & 0b1111) as usize].as_ref() else {
            return OptionalSolution::NONE;
        };
        let Some(bin5) = &bin4[((node.0 >> (48 - 5 * 4)) & 0b1111) as usize].as_ref() else {
            return OptionalSolution::NONE;
        };
        let Some(raw) = bin5[((node.0 >> (48 - 6 * 4)) & 0b1111) as usize].into_zero_padded_i9()
        else {
            return OptionalSolution::NONE;
        };

        let left = node.0 & 0xFFFF_FFFF_FF00_0000;
        let right = raw;
        OptionalSolution(left | right)
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

impl OptionalSolution {
    const NONE: Self = OptionalSolution(0);

    fn is_none(self) -> bool {
        self == Self::NONE
    }

    fn is_some(self) -> bool {
        self != Self::NONE
    }

    fn unchecked_unwrap(self) -> Solution {
        Solution(self.0)
    }
}

impl OptionalSearchNode {
    const NONE: Self = OptionalSearchNode(0);

    fn is_none(self) -> bool {
        self == Self::NONE
    }

    fn unchecked_unwrap(self) -> SearchNode {
        SearchNode(self.0)
    }
}

impl OptionalNodeBuilder {
    const NONE: Self = OptionalNodeBuilder(0);

    fn is_none(self) -> bool {
        self == Self::NONE
    }

    fn unchecked_unwrap(self) -> NodeBuilder {
        NodeBuilder(self.0)
    }
}

impl OptionalAction {
    const NONE: Self = OptionalAction(0);

    fn is_none(self) -> bool {
        self == Self::NONE
    }

    fn unchecked_unwrap(self) -> Action {
        Action(self.0)
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
const ACTION_HANDLERS: [fn(SearchNode) -> (OptionalNodeBuilder, OptionalAction); 128] = [
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

fn handle_bad_action(_: SearchNode) -> (OptionalNodeBuilder, OptionalAction) {
    panic!("Illegal action");
}

const CHICK_0_ALLEGIANCE_MASK: u64 = 0b1 << (0 + 9 + 7 + 8 + 4 + 4 + 5 + 5 + 5 + 5 + 6 + 5);

const CHICK1_STARTING_ACTION: Action = Action(0b010_0000);

fn handle_chick0_row00_col00(_state: SearchNode) -> (OptionalNodeBuilder, OptionalAction) {
    // const THIS_ACTION: Action = Action(unsafe { NonZeroU8::new_unchecked(0b001_0000) });
    // const NEXT_BOUND: Action = unsafe { THIS_ACTION.increment_unchecked() };
    // const NEXT_PIECE_ACTION: Option<Action> = Some(CHICK1_STARTING_ACTION);

    // let min_reachable = state.get_minimum_reachable_chick0_action(THIS_ACTION);
    // if min_reachable != Some(THIS_ACTION) {
    //     return (None, min_reachable.or(NEXT_PIECE_ACTION));
    // }

    // let Some(state) = state.vacate_row00_col00() else {
    //     return (None, NEXT_PIECE_ACTION);
    // };

    // let state = state
    //     .set_chick0_position_and_normalize(0b00_00)
    //     .flip_active_player();
    // let min_reachable = state.get_minimum_reachable_chick0_action(NEXT_BOUND);
    // (Some(state), min_reachable.or(NEXT_PIECE_ACTION))
    todo!()
}

// impl TimelessState {
//     fn flip_active_player(self) -> Self {
//         todo!()
//     }

//     fn is_chick0_passive(self) -> bool {
//         self.0 & CHICK_0_ALLEGIANCE_MASK != 0
//     }

//     fn set_chick0_position_and_normalize(self, position: u64) -> Self {
//         todo!()
//     }

//     /// - If row 0, column 0 is empty, we return the original state.
//     /// - If it is occupied by a passive piece, we move that piece
//     ///   to the active player's hand, and return the new state.
//     /// - If it is occupied by an active piece, return `None`.
//     fn vacate_row00_col00(self) -> Option<Self> {
//         todo!()
//     }
// }

// An action is "reachable" by a certain piece
// if the piece is allegiant to the active player,
// and the piece's move pattern allows it to move to the target square.
// The target square may contain an active piece.
// As a corollary, a reachable action is not necessarily legal.
// impl TimelessState {
//     fn get_minimum_reachable_chick0_action(self, lower_bound: Action) -> Option<Action> {
//         if self.is_chick0_passive() {
//             return None;
//         }

//         todo!()
//     }
// }

// impl Action {
//     const unsafe fn increment_unchecked(self) -> Action {
//         let raw = self.0.get();

//         if raw & 0b11 == 0b10 {
//             return Action(unsafe { NonZeroU8::new_unchecked(raw) });
//         }

//         Action(unsafe { NonZeroU8::new_unchecked(raw + 1) })
//     }
// }

fn todo_dummy(_: SearchNode) -> (OptionalNodeBuilder, OptionalAction) {
    todo!()
}
