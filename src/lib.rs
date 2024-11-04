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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Board(u64);

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
            (chick0 << offsets::CHICK0)
                | (chick1 << offsets::CHICK1)
                | (elephant0 << offsets::ELEPHANT0)
                | (elephant1 << offsets::ELEPHANT1)
                | (giraffe0 << offsets::GIRAFFE0)
                | (giraffe1 << offsets::GIRAFFE1)
                | (active_lion << offsets::ACTIVE_LION)
                | (passive_lion << offsets::PASSIVE_LION)
                | (ply_count << offsets::PLY_COUNT)
                | (lowest_unexplored_action << offsets::NEXT_ACTION)
                | (best_discovered_evaluation << offsets::BEST_DISCOVERED_OUTCOME),
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
        let (child_builder, next_action) = ACTION_HANDLERS[(action.0 - 16) as usize](self);

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
const ACTION_HANDLERS: [fn(SearchNode) -> (OptionalNodeBuilder, OptionalAction); 128 - 16] = [
    // 0b000_0000 to 0b000_1111 are unreachable
    // due to the offset of 16.

    // activeLion: 0b001_0000 to 0b111_1111
    todo_dummy,
    todo_dummy,
    todo_dummy,
    handle_bad_action,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    handle_bad_action,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    handle_bad_action,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    handle_bad_action,
    // chick0: 0b010_0000 to 0b010_1111
    handle_chick0_row00_col00,
    todo_dummy,
    todo_dummy,
    handle_bad_action,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    handle_bad_action,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    handle_bad_action,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    handle_bad_action,
    // chick1: 0b011_0000 to 0b011_1111
    todo_dummy,
    todo_dummy,
    todo_dummy,
    handle_bad_action,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    handle_bad_action,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    handle_bad_action,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    handle_bad_action,
    // elephant0: 0b100_0000 to 0b100_1111
    todo_dummy,
    todo_dummy,
    todo_dummy,
    handle_bad_action,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    handle_bad_action,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    handle_bad_action,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    handle_bad_action,
    // elephant1: 0b101_0000 to 0b101_1111
    todo_dummy,
    todo_dummy,
    todo_dummy,
    handle_bad_action,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    handle_bad_action,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    handle_bad_action,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    handle_bad_action,
    // giraffe0: 0b110_0000 to 0b110_1111
    todo_dummy,
    todo_dummy,
    todo_dummy,
    handle_bad_action,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    handle_bad_action,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    handle_bad_action,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    handle_bad_action,
    // giraffe1: 0b111_0000 to 0b111_1111
    todo_dummy,
    todo_dummy,
    todo_dummy,
    handle_bad_action,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    handle_bad_action,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    handle_bad_action,
    todo_dummy,
    todo_dummy,
    todo_dummy,
    handle_bad_action,
];

fn handle_bad_action(_: SearchNode) -> (OptionalNodeBuilder, OptionalAction) {
    panic!("Illegal action");
}

macro_rules! is_dest_square_occupied {
    ($ACTION:expr, $board:expr) => {{
        // TODO
        let _dummy: Action = $ACTION;
        let _dummy: Board = $board;
        false
    }};
}

macro_rules! move_acting_piece_to_dest_square {
    ($ACTION:expr, $state:expr) => {{
        // TODO
        let _dummy: Action = $ACTION;
        let _dummy: NodeBuilder = $state;
        NodeBuilder(0)
    }};
}

macro_rules! next_empty_square_action {
    ($ACTION:expr, $state:expr) => {{
        // TODO
        let _dummy: Action = $ACTION;
        let _dummy: NodeBuilder = $state;
        OptionalAction(0)
    }};
}

macro_rules! handle_chick_drop_assuming_it_is_in_hand_and_has_active_allegiance {
    ($ACTION:expr, $original_state:expr) => {{
        let original_state = $original_state;
        let original_board = $original_state.board();

        if is_dest_square_occupied!($ACTION, original_board) {
            return (OptionalNodeBuilder::NONE, $ACTION.next_species_action());
        }

        let state = move_acting_piece_to_dest_square!($ACTION, original_state);
        let next_action = next_empty_square_action!($ACTION, original_state);
        (state.into_optional(), next_action)
    }};
}

/// - If the destination square is empty, this returns the original state.
/// - If the destination square is occupied by a passive piece,
///   this returns the state with the passive piece moved to the active player's hand.
/// - If the destination square is occupied by an active piece,
///   this returns `OptionalNodeBuilder::NONE`.
macro_rules! vacate_passive_dest_square {
    ($ACTION:expr, $state:expr, $board:expr) => {{
        // TODO
        let _dummy: Action = $ACTION;
        let _dummy: NodeBuilder = $state;
        let _dummy: Board = $board;
        OptionalNodeBuilder::NONE
    }};
}

macro_rules! next_action_with_nonactive_dest_square_in_range_of_current_actor {
    ($ACTION:expr, $state:expr, $board:expr) => {{
        // TODO
        let _dummy: Action = $ACTION;
        let _dummy: NodeBuilder = $state;
        let _dummy: Board = $board;
        OptionalAction(0)
    }};
}

macro_rules! handle_chick_move_assuming_it_is_in_range_of_dest_square_and_has_active_allegiance {
    ($ACTION:expr, $original_state:expr) => {{
        let original_state = $original_state;
        let original_board = original_state.board();
        let state = vacate_passive_dest_square!($ACTION, original_state, original_board);

        // If the destination square is occupied by an active piece,
        // then the move is illegal.
        if state.is_none() {
            return (
                OptionalNodeBuilder::NONE,
                next_action_with_nonactive_dest_square_in_range_of_current_actor!(
                    $ACTION,
                    original_state,
                    original_board
                ),
            );
        }

        let state = state.unchecked_unwrap();
        let state = move_acting_piece_to_dest_square!($ACTION, state);
        let next_action = next_action_with_nonactive_dest_square_in_range_of_current_actor!(
            $ACTION,
            original_state,
            original_board
        );
        (state.into_optional(), next_action)
    }};
}

macro_rules! is_actor_out_of_range_of_dest_square {
    ($ACTION:expr, $state:expr) => {{
        // TODO
        false
    }};
}

macro_rules! next_action_with_dest_square_in_current_actor_range {
    ($ACTION:expr, $state:expr) => {{
        // TODO
        OptionalAction(0)
    }};
}

macro_rules! handle_chick_action {
    ($ACTION:expr, $state:expr) => {{
        if $state.0 & $ACTION.allegiance_mask() != 0 {
            return (OptionalNodeBuilder::NONE, $ACTION.next_species_action());
        }

        if $state.0 & $ACTION.hand_mask() == $ACTION.hand_mask() {
            return handle_chick_drop_assuming_it_is_in_hand_and_has_active_allegiance!(
                $ACTION, $state
            );
        }

        if is_actor_out_of_range_of_dest_square!($ACTION, $state) {
            return (
                OptionalNodeBuilder::NONE,
                next_action_with_dest_square_in_current_actor_range!($ACTION, $state),
            );
        }

        return handle_chick_move_assuming_it_is_in_range_of_dest_square_and_has_active_allegiance!(
            $ACTION, $state
        );
    }};
}

fn handle_chick0_row00_col00(state: SearchNode) -> (OptionalNodeBuilder, OptionalAction) {
    handle_chick_action!(Action(0b001_0000), state.into_builder())
}

impl Action {
    const fn allegiance_mask(self) -> u64 {
        let offset = match self.0 >> 4 {
            // There is no mask for the active lion, since it's allegiance
            // is fixed.
            0b001 => return 0,

            0b010 => offsets::CHICK0_ALLEGIANCE,
            0b011 => offsets::CHICK1_ALLEGIANCE,
            0b100 => offsets::ELEPHANT0_ALLEGIANCE,
            0b101 => offsets::ELEPHANT1_ALLEGIANCE,
            0b110 => offsets::GIRAFFE0_ALLEGIANCE,
            0b111 => offsets::GIRAFFE1_ALLEGIANCE,

            _ => return 0,
        };

        1 << offset
    }

    const fn next_species_action(self) -> OptionalAction {
        OptionalAction(match self.0 >> 4 {
            0b001 => 0b010_0000,

            0b010 => 0b100_0000,
            0b011 => 0b100_0000,

            0b100 => 0b110_0000,
            0b101 => 0b110_0000,

            0b110 => 0,
            0b111 => 0,

            _ => 0,
        })
    }

    const fn hand_mask(self) -> u64 {
        todo!()
    }
}

fn todo_dummy(_: SearchNode) -> (OptionalNodeBuilder, OptionalAction) {
    todo!()
}

mod offsets {
    pub const CHICK0: u64 = 0 + 9 + 7 + 8 + 4 + 4 + 5 + 5 + 5 + 5 + 6;
    pub const CHICK1: u64 = 0 + 9 + 7 + 8 + 4 + 4 + 5 + 5 + 5 + 5;
    pub const ELEPHANT0: u64 = 0 + 9 + 7 + 8 + 4 + 4 + 5 + 5 + 5;
    pub const ELEPHANT1: u64 = 0 + 9 + 7 + 8 + 4 + 4 + 5 + 5;
    pub const GIRAFFE0: u64 = 0 + 9 + 7 + 8 + 4 + 4 + 5;
    pub const GIRAFFE1: u64 = 0 + 9 + 7 + 8 + 4 + 4;
    pub const ACTIVE_LION: u64 = 0 + 9 + 7 + 8 + 4;
    pub const PASSIVE_LION: u64 = 0 + 9 + 7 + 8;
    pub const PLY_COUNT: u64 = 0 + 9 + 7;
    pub const NEXT_ACTION: u64 = 0 + 9;
    pub const BEST_DISCOVERED_OUTCOME: u64 = 0;

    pub const CHICK0_PROMOTION: u64 = CHICK0;
    pub const CHICK0_COLUMN: u64 = CHICK0_PROMOTION + 1;
    pub const CHICK0_ROW: u64 = CHICK0_COLUMN + 2;
    pub const CHICK0_ALLEGIANCE: u64 = CHICK0_ROW + 2;

    pub const CHICK1_PROMOTION: u64 = CHICK1;
    pub const CHICK1_COLUMN: u64 = CHICK1_PROMOTION + 1;
    pub const CHICK1_ROW: u64 = CHICK1_COLUMN + 2;
    pub const CHICK1_ALLEGIANCE: u64 = CHICK1_ROW + 2;

    pub const ELEPHANT0_COLUMN: u64 = ELEPHANT0;
    pub const ELEPHANT0_ROW: u64 = ELEPHANT0_COLUMN + 2;
    pub const ELEPHANT0_ALLEGIANCE: u64 = ELEPHANT0_ROW + 1;

    pub const ELEPHANT1_COLUMN: u64 = ELEPHANT1;
    pub const ELEPHANT1_ROW: u64 = ELEPHANT1_COLUMN + 2;
    pub const ELEPHANT1_ALLEGIANCE: u64 = ELEPHANT1_ROW + 1;

    pub const GIRAFFE0_COLUMN: u64 = GIRAFFE0;
    pub const GIRAFFE0_ROW: u64 = GIRAFFE0_COLUMN + 2;
    pub const GIRAFFE0_ALLEGIANCE: u64 = GIRAFFE0_ROW + 1;

    pub const GIRAFFE1_COLUMN: u64 = GIRAFFE1;
    pub const GIRAFFE1_ROW: u64 = GIRAFFE1_COLUMN + 2;
    pub const GIRAFFE1_ALLEGIANCE: u64 = GIRAFFE1_ROW + 1;

    pub const ACTIVE_LION_COLUMN: u64 = ACTIVE_LION;
    pub const ACTIVE_LION_ROW: u64 = ACTIVE_LION_COLUMN + 2;

    pub const PASSIVE_LION_COLUMN: u64 = PASSIVE_LION;
    pub const PASSIVE_LION_ROW: u64 = PASSIVE_LION_COLUMN + 2;
}

impl SearchNode {
    const fn into_builder(self) -> NodeBuilder {
        NodeBuilder(self.0)
    }
}

impl NodeBuilder {
    const fn board(self) -> Board {
        todo!()
    }

    const fn into_optional(self) -> OptionalNodeBuilder {
        OptionalNodeBuilder(self.0)
    }
}
