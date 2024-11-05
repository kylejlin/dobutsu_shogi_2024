// A note about fileds with the comment "This must be non-zero":
// I know we _could_ use a `NonZeroU64` (or another respective `NonZero*` type),
// but that would clutter the code with a bunch of unwraps,
// which hurts readability and performance.

use offsets::CHICK0;

pub const MAX_PLY_COUNT: u8 = 200;

pub fn calculate() -> CompactSolutionMap {
    let mut solution_cache = SolutionCache::new();

    let mut stack: Vec<SearchNode> = Vec::with_capacity(MAX_PLY_COUNT as usize);
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

        let best_discovered_evaluation: u64 = NEGATIVE_201_I9;

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
            child_builder
                .unchecked_unwrap()
                .invert_active_player()
                .increment_ply_count()
                .init_best_discovered_outcome_and_next_action()
                .build()
                .into_optional()
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
        const ALLEGIANCE_INVERSION_MASK: u64 = (1 << offsets::CHICK0_ALLEGIANCE)
            | (1 << offsets::CHICK1_ALLEGIANCE)
            | (1 << offsets::ELEPHANT0_ALLEGIANCE)
            | (1 << offsets::ELEPHANT1_ALLEGIANCE)
            | (1 << offsets::GIRAFFE0_ALLEGIANCE)
            | (1 << offsets::GIRAFFE1_ALLEGIANCE);

        const ACTIVE_LION_MASK: u64 = 0b1111 << offsets::ACTIVE_LION;
        const PASSIVE_LION_MASK: u64 = 0b1111 << offsets::PASSIVE_LION;

        let active_lion_bits_in_original_position = self.0 & ACTIVE_LION_MASK;
        let passive_lion_bits_in_original_position = self.0 & PASSIVE_LION_MASK;

        let out = self.0 ^ ALLEGIANCE_INVERSION_MASK;

        let out = (out & !ACTIVE_LION_MASK)
            | (passive_lion_bits_in_original_position
                << (offsets::ACTIVE_LION - offsets::PASSIVE_LION));

        let out = (out & !PASSIVE_LION_MASK)
            | (active_lion_bits_in_original_position
                >> (offsets::ACTIVE_LION - offsets::PASSIVE_LION));

        Self(out)
    }

    const fn increment_ply_count(self) -> Self {
        const C: u64 = 1 << offsets::PLY_COUNT;
        Self(self.0 + C)
    }

    /// If the this is terminal, then we set the best discovered outcome
    /// to the outcome of the game, and we set the next action to `None`.
    /// Otherwise, we set the best discovered outcome to `-200`,
    /// and we set the next action `to Action(0b001_0000)`.
    const fn init_best_discovered_outcome_and_next_action(self) -> Self {
        const ACTIVE_LION_HAND_MASK: u64 = 0b1111 << offsets::ACTIVE_LION;

        // If the active lion is in the passive player's hand,
        // the active player has lost.
        if self.0 & ACTIVE_LION_HAND_MASK == ACTIVE_LION_HAND_MASK {
            return self.init_best_discovered_outcome_and_next_action_assuming_loss();
        }

        const ACTIVE_LION_TRY_MASK: u64 = 0b11 << offsets::ACTIVE_LION_ROW;

        // If the active lion is in the last row,
        // the active player has won.
        if self.0 & ACTIVE_LION_TRY_MASK == ACTIVE_LION_TRY_MASK {
            return self.init_best_discovered_outcome_and_next_action_assuming_win();
        }

        const PLY_COUNT_MASK: u64 = 0xFF << offsets::PLY_COUNT;
        const MAX_PLY_COUNT_SHIFTED: u64 = (MAX_PLY_COUNT as u64) << offsets::PLY_COUNT;
        if self.0 & PLY_COUNT_MASK == MAX_PLY_COUNT_SHIFTED {
            return self.init_best_discovered_outcome_and_next_action_assuming_draw();
        }

        const DEFAULT_FIRST_ACTION: Action = Action(0b001_0000);
        Self(
            (self.0 & !0xFFFF)
                | ((DEFAULT_FIRST_ACTION.0 as u64) << offsets::NEXT_ACTION)
                | (NEGATIVE_201_I9 << offsets::BEST_DISCOVERED_OUTCOME),
        )
    }

    const fn init_best_discovered_outcome_and_next_action_assuming_loss(self) -> Self {
        Self(
            (self.0 & !0xFFFF)
                | (NEGATIVE_201_I9 << offsets::BEST_DISCOVERED_OUTCOME)
                | ((OptionalAction::NONE.0 as u64) << offsets::NEXT_ACTION),
        )
    }

    const fn init_best_discovered_outcome_and_next_action_assuming_win(self) -> Self {
        Self(
            (self.0 & !0xFFFF)
                | (POSITIVE_201_I9 << offsets::BEST_DISCOVERED_OUTCOME)
                | ((OptionalAction::NONE.0 as u64) << offsets::NEXT_ACTION),
        )
    }

    const fn init_best_discovered_outcome_and_next_action_assuming_draw(self) -> Self {
        Self(
            (self.0 & !0xFFFF)
                | (0 << offsets::BEST_DISCOVERED_OUTCOME)
                | ((OptionalAction::NONE.0 as u64) << offsets::NEXT_ACTION),
        )
    }

    const fn build(self) -> SearchNode {
        let nonflipped = self.build_without_horizontal_normalization();
        let flipped = self
            .horizontally_flip()
            .build_without_horizontal_normalization();

        if flipped.0 < nonflipped.0 {
            return flipped;
        }

        nonflipped
    }

    /// Ensures that `chick0 <= chick1`, `elephant0 <= elephant1`, and `giraffe0 <= giraffe1`.
    const fn build_without_horizontal_normalization(self) -> SearchNode {
        const CHICK0_MASK: u64 = 0b11_1111 << offsets::CHICK0;
        const CHICK1_MASK: u64 = 0b11_1111 << offsets::CHICK1;
        const ELEPHANT0_MASK: u64 = 0b1_1111 << offsets::ELEPHANT0;
        const ELEPHANT1_MASK: u64 = 0b1_1111 << offsets::ELEPHANT1;
        const GIRAFFE0_MASK: u64 = 0b1_1111 << offsets::GIRAFFE0;
        const GIRAFFE1_MASK: u64 = 0b1_1111 << offsets::GIRAFFE1;

        let chick0 = self.0 & CHICK0_MASK;
        let chick1 = self.0 & CHICK1_MASK;
        let chick1_shifted = chick1 << (offsets::CHICK0 - offsets::CHICK1);
        let (chick0, chick1) = if chick0 <= chick1_shifted {
            (chick0, chick1)
        } else {
            (
                chick0 >> (offsets::CHICK0 - offsets::CHICK1),
                chick1_shifted,
            )
        };

        let elephant0 = self.0 & ELEPHANT0_MASK;
        let elephant1 = self.0 & ELEPHANT1_MASK;
        let elephant1_shifted = elephant1 << (offsets::ELEPHANT0 - offsets::ELEPHANT1);
        let (elephant0, elephant1) = if elephant0 <= elephant1_shifted {
            (elephant0, elephant1)
        } else {
            (
                elephant0 >> (offsets::ELEPHANT0 - offsets::ELEPHANT1),
                elephant1_shifted,
            )
        };

        let giraffe0 = self.0 & GIRAFFE0_MASK;
        let giraffe1 = self.0 & GIRAFFE1_MASK;
        let giraffe1_shifted = giraffe1 << (offsets::GIRAFFE0 - offsets::GIRAFFE1);
        let (giraffe0, giraffe1) = if giraffe0 <= giraffe1_shifted {
            (giraffe0, giraffe1)
        } else {
            (
                giraffe0 >> (offsets::GIRAFFE0 - offsets::GIRAFFE1),
                giraffe1_shifted,
            )
        };

        const NONLION_MASK: u64 = 0xFFFF_FFFF;
        SearchNode(
            (self.0 & !NONLION_MASK)
                | chick0
                | chick1
                | elephant0
                | elephant1
                | giraffe0
                | giraffe1,
        )
    }

    const fn horizontally_flip(self) -> Self {
        const CHICK0_COORDS_MASK: u64 = 0b1111 << offsets::CHICK0_COLUMN;
        let chick0_coords = self.0 & CHICK0_COORDS_MASK;
        let chick0_coords_flipped = if chick0_coords == CHICK0_COORDS_MASK {
            chick0_coords
        } else {
            (0b10 << offsets::CHICK0_COLUMN) - chick0_coords
        };

        const CHICK1_COORDS_MASK: u64 = 0b1111 << offsets::CHICK1_COLUMN;
        let chick1_coords = self.0 & CHICK1_COORDS_MASK;
        let chick1_coords_flipped = if chick1_coords == CHICK1_COORDS_MASK {
            chick1_coords
        } else {
            (0b10 << offsets::CHICK1_COLUMN) - chick1_coords
        };

        const ELEPHANT0_COORDS_MASK: u64 = 0b1111 << offsets::ELEPHANT0_COLUMN;
        let elephant0_coords = self.0 & ELEPHANT0_COORDS_MASK;
        let elephant0_coords_flipped = if elephant0_coords == ELEPHANT0_COORDS_MASK {
            elephant0_coords
        } else {
            (0b10 << offsets::ELEPHANT0_COLUMN) - elephant0_coords
        };

        const ELEPHANT1_COORDS_MASK: u64 = 0b1111 << offsets::ELEPHANT1_COLUMN;
        let elephant1_coords = self.0 & ELEPHANT1_COORDS_MASK;
        let elephant1_coords_flipped = if elephant1_coords == ELEPHANT1_COORDS_MASK {
            elephant1_coords
        } else {
            (0b10 << offsets::ELEPHANT1_COLUMN) - elephant1_coords
        };

        const GIRAFFE0_COORDS_MASK: u64 = 0b1111 << offsets::GIRAFFE0_COLUMN;
        let giraffe0_coords = self.0 & GIRAFFE0_COORDS_MASK;
        let giraffe0_coords_flipped = if giraffe0_coords == GIRAFFE0_COORDS_MASK {
            giraffe0_coords
        } else {
            (0b10 << offsets::GIRAFFE0_COLUMN) - giraffe0_coords
        };

        const GIRAFFE1_COORDS_MASK: u64 = 0b1111 << offsets::GIRAFFE1_COLUMN;
        let giraffe1_coords = self.0 & GIRAFFE1_COORDS_MASK;
        let giraffe1_coords_flipped = if giraffe1_coords == GIRAFFE1_COORDS_MASK {
            giraffe1_coords
        } else {
            (0b10 << offsets::GIRAFFE1_COLUMN) - giraffe1_coords
        };

        const ACTIVE_LION_COORDS_MASK: u64 = 0b1111 << offsets::ACTIVE_LION_COLUMN;
        let active_lion_coords = self.0 & ACTIVE_LION_COORDS_MASK;
        let active_lion_coords_flipped = if active_lion_coords == ACTIVE_LION_COORDS_MASK {
            active_lion_coords
        } else {
            (0b10 << offsets::ACTIVE_LION_COLUMN) - active_lion_coords
        };

        const PASSIVE_LION_COORDS_MASK: u64 = 0b1111 << offsets::PASSIVE_LION_COLUMN;
        let passive_lion_coords = self.0 & PASSIVE_LION_COORDS_MASK;
        let passive_lion_coords_flipped = if passive_lion_coords == PASSIVE_LION_COORDS_MASK {
            passive_lion_coords
        } else {
            (0b10 << offsets::PASSIVE_LION_COLUMN) - passive_lion_coords
        };

        Self(
            (self.0 & !(0xFF_FFFF_FFFF << offsets::PASSIVE_LION))
                | chick0_coords_flipped
                | chick1_coords_flipped
                | elephant0_coords_flipped
                | elephant1_coords_flipped
                | giraffe0_coords_flipped
                | giraffe1_coords_flipped
                | active_lion_coords_flipped
                | passive_lion_coords_flipped,
        )
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
        if (value & (1 << 8)) != 0 {
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

    const fn is_none(self) -> bool {
        self.0 == Self::NONE.0
    }

    const fn is_some(self) -> bool {
        self.0 != Self::NONE.0
    }

    const fn unchecked_unwrap(self) -> Solution {
        Solution(self.0)
    }
}

impl OptionalSearchNode {
    const NONE: Self = OptionalSearchNode(0);

    const fn is_none(self) -> bool {
        self.0 == Self::NONE.0
    }

    const fn unchecked_unwrap(self) -> SearchNode {
        SearchNode(self.0)
    }
}

impl OptionalNodeBuilder {
    const NONE: Self = OptionalNodeBuilder(0);

    const fn is_none(self) -> bool {
        self.0 == Self::NONE.0
    }

    const fn unchecked_unwrap(self) -> NodeBuilder {
        NodeBuilder(self.0)
    }
}

impl OptionalAction {
    const NONE: Self = OptionalAction(0);

    const fn is_none(self) -> bool {
        self.0 == Self::NONE.0
    }

    const fn unchecked_unwrap(self) -> Action {
        Action(self.0)
    }
}

impl SearchNode {
    const fn into_builder(self) -> NodeBuilder {
        NodeBuilder(self.0)
    }

    const fn into_optional(self) -> OptionalSearchNode {
        OptionalSearchNode(self.0)
    }
}

impl NodeBuilder {
    const fn board(self) -> Board {
        const CHICK0_COORDS_MASK: u64 = 0b1111 << offsets::CHICK0_COLUMN;
        const CHICK1_COORDS_MASK: u64 = 0b1111 << offsets::CHICK1_COLUMN;
        const ELEPHANT0_COORDS_MASK: u64 = 0b1111 << offsets::ELEPHANT0_COLUMN;
        const ELEPHANT1_COORDS_MASK: u64 = 0b1111 << offsets::ELEPHANT1_COLUMN;
        const GIRAFFE0_COORDS_MASK: u64 = 0b1111 << offsets::GIRAFFE0_COLUMN;
        const GIRAFFE1_COORDS_MASK: u64 = 0b1111 << offsets::GIRAFFE1_COLUMN;
        const ACTIVE_LION_COORDS_MASK: u64 = 0b1111 << offsets::ACTIVE_LION_COLUMN;
        const PASSIVE_LION_COORDS_MASK: u64 = 0b1111 << offsets::PASSIVE_LION_COLUMN;

        let chick0_coords = self.0 & CHICK0_COORDS_MASK;
        let chick1_coords = self.0 & CHICK1_COORDS_MASK;
        let elephant0_coords = self.0 & ELEPHANT0_COORDS_MASK;
        let elephant1_coords = self.0 & ELEPHANT1_COORDS_MASK;
        let giraffe0_coords = self.0 & GIRAFFE0_COORDS_MASK;
        let giraffe1_coords = self.0 & GIRAFFE1_COORDS_MASK;
        let active_lion_coords = self.0 & ACTIVE_LION_COORDS_MASK;
        let passive_lion_coords = self.0 & PASSIVE_LION_COORDS_MASK;

        const LION_SQUARE_PIECE: u64 = 0b001;
        const CHICK0_SQUARE_PIECE: u64 = 0b010;
        const CHICK1_SQUARE_PIECE: u64 = 0b011;
        const ELEPHANT0_SQUARE_PIECE: u64 = 0b100;
        const ELEPHANT1_SQUARE_PIECE: u64 = 0b101;
        const GIRAFFE0_SQUARE_PIECE: u64 = 0b110;
        const GIRAFFE1_SQUARE_PIECE: u64 = 0b111;

        let mut board: u64 = 0;

        // For each piece, we first check whether it's in the hand.
        // If so, we skip it.
        // Otherwise, we calculate the board offset and add the piece to the board.

        if chick0_coords != CHICK0_COORDS_MASK {
            let board_offset = coords_to_board_offset(chick0_coords >> offsets::CHICK0_COLUMN);
            let allegiance_in_bit3 = (self.0 >> (offsets::CHICK0_ALLEGIANCE - 3)) & (1 << 3);
            board |= (allegiance_in_bit3 | CHICK0_SQUARE_PIECE) << board_offset;
        }

        if chick1_coords != CHICK1_COORDS_MASK {
            let board_offset = coords_to_board_offset(chick1_coords >> offsets::CHICK1_COLUMN);
            let allegiance_in_bit3 = (self.0 >> (offsets::CHICK1_ALLEGIANCE - 3)) & (1 << 3);
            board |= (allegiance_in_bit3 | CHICK1_SQUARE_PIECE) << board_offset;
        }

        if elephant0_coords != ELEPHANT0_COORDS_MASK {
            let board_offset =
                coords_to_board_offset(elephant0_coords >> offsets::ELEPHANT0_COLUMN);
            let allegiance_in_bit3 = (self.0 >> (offsets::ELEPHANT0_ALLEGIANCE - 3)) & (1 << 3);
            board |= (allegiance_in_bit3 | ELEPHANT0_SQUARE_PIECE) << board_offset;
        }

        if elephant1_coords != ELEPHANT1_COORDS_MASK {
            let board_offset =
                coords_to_board_offset(elephant1_coords >> offsets::ELEPHANT1_COLUMN);
            let allegiance_in_bit3 = (self.0 >> (offsets::ELEPHANT1_ALLEGIANCE - 3)) & (1 << 3);
            board |= (allegiance_in_bit3 | ELEPHANT1_SQUARE_PIECE) << board_offset;
        }

        if giraffe0_coords != GIRAFFE0_COORDS_MASK {
            let board_offset = coords_to_board_offset(giraffe0_coords >> offsets::GIRAFFE0_COLUMN);
            let allegiance_in_bit3 = (self.0 >> (offsets::GIRAFFE0_ALLEGIANCE - 3)) & (1 << 3);
            board |= (allegiance_in_bit3 | GIRAFFE0_SQUARE_PIECE) << board_offset;
        }

        if giraffe1_coords != GIRAFFE1_COORDS_MASK {
            let board_offset = coords_to_board_offset(giraffe1_coords >> offsets::GIRAFFE1_COLUMN);
            let allegiance_in_bit3 = (self.0 >> (offsets::GIRAFFE1_ALLEGIANCE - 3)) & (1 << 3);
            board |= (allegiance_in_bit3 | GIRAFFE1_SQUARE_PIECE) << board_offset;
        }

        if active_lion_coords != ACTIVE_LION_COORDS_MASK {
            let board_offset =
                coords_to_board_offset(active_lion_coords >> offsets::ACTIVE_LION_COLUMN);
            const ALLEGIANCE_IN_BIT3: u64 = 0 << 3;
            board |= (ALLEGIANCE_IN_BIT3 | LION_SQUARE_PIECE) << board_offset;
        }

        if passive_lion_coords != PASSIVE_LION_COORDS_MASK {
            let board_offset =
                coords_to_board_offset(passive_lion_coords >> offsets::PASSIVE_LION_COLUMN);
            const ALLEGIANCE_IN_BIT3: u64 = 1 << 3;
            board |= (ALLEGIANCE_IN_BIT3 | LION_SQUARE_PIECE) << board_offset;
        }

        Board(board)
    }

    const fn into_optional(self) -> OptionalNodeBuilder {
        OptionalNodeBuilder(self.0)
    }
}

const fn coords_to_board_offset(coords: u64) -> u64 {
    let col = coords & 0b11;
    let row = coords >> 2;
    (row * 3 + col) * 4
}

/// `-200`` in 9-bit two's complement, left-padded with zeros
/// to fill the 64-bit integer.
const NEGATIVE_201_I9: u64 = 0b1_0011_0111;

/// `200` in 9-bit two's complement, left-padded with zeros
/// to fill the 64-bit integer.
const POSITIVE_201_I9: u64 = 0b0_1100_1001;

/// An action handler will return the result of applying an action
/// to the input state, if the action is legal.
/// If the action is illegal, then the handler will return `None`
/// instead of the resulting timeless state.
///
/// Regardless of the legality of the action,
/// the handler will return an `Option<Action>`
/// that represents the next (possibly illegal) action to be explored.
///
/// The handler assumes that the input state is non-terminal.
/// It will not check for terminality.
const ACTION_HANDLERS: [fn(SearchNode) -> (OptionalNodeBuilder, OptionalAction); 128 - 16] = [
    // 0b000_0000 to 0b000_1111 are unreachable
    // due to the offset of 16.

    // activeLion: 0b001_0000 to 0b111_1111
    action_handlers::active_lion::r00_c00,
    action_handlers::active_lion::r00_c01,
    action_handlers::active_lion::r00_c10,
    action_handlers::handle_bad_action,
    action_handlers::active_lion::r01_c00,
    action_handlers::active_lion::r01_c01,
    action_handlers::active_lion::r01_c10,
    action_handlers::handle_bad_action,
    action_handlers::active_lion::r10_c00,
    action_handlers::active_lion::r10_c01,
    action_handlers::active_lion::r10_c10,
    action_handlers::handle_bad_action,
    action_handlers::active_lion::r11_c00,
    action_handlers::active_lion::r11_c01,
    action_handlers::active_lion::r11_c10,
    action_handlers::handle_bad_action,
    // chick0: 0b010_0000 to 0b010_1111
    action_handlers::chick0::r00_c00,
    action_handlers::chick0::r00_c01,
    action_handlers::chick0::r00_c10,
    action_handlers::handle_bad_action,
    action_handlers::chick0::r01_c00,
    action_handlers::chick0::r01_c01,
    action_handlers::chick0::r01_c10,
    action_handlers::handle_bad_action,
    action_handlers::chick0::r10_c00,
    action_handlers::chick0::r10_c01,
    action_handlers::chick0::r10_c10,
    action_handlers::handle_bad_action,
    action_handlers::chick0::r11_c00,
    action_handlers::chick0::r11_c01,
    action_handlers::chick0::r11_c10,
    action_handlers::handle_bad_action,
    // chick1: 0b011_0000 to 0b011_1111
    action_handlers::chick1::r00_c00,
    action_handlers::chick1::r00_c01,
    action_handlers::chick1::r00_c10,
    action_handlers::handle_bad_action,
    action_handlers::chick1::r01_c00,
    action_handlers::chick1::r01_c01,
    action_handlers::chick1::r01_c10,
    action_handlers::handle_bad_action,
    action_handlers::chick1::r10_c00,
    action_handlers::chick1::r10_c01,
    action_handlers::chick1::r10_c10,
    action_handlers::handle_bad_action,
    action_handlers::chick1::r11_c00,
    action_handlers::chick1::r11_c01,
    action_handlers::chick1::r11_c10,
    action_handlers::handle_bad_action,
    // elephant0: 0b100_0000 to 0b100_1111
    action_handlers::elephant0::r00_c00,
    action_handlers::elephant0::r00_c01,
    action_handlers::elephant0::r00_c10,
    action_handlers::handle_bad_action,
    action_handlers::elephant0::r01_c00,
    action_handlers::elephant0::r01_c01,
    action_handlers::elephant0::r01_c10,
    action_handlers::handle_bad_action,
    action_handlers::elephant0::r10_c00,
    action_handlers::elephant0::r10_c01,
    action_handlers::elephant0::r10_c10,
    action_handlers::handle_bad_action,
    action_handlers::elephant0::r11_c00,
    action_handlers::elephant0::r11_c01,
    action_handlers::elephant0::r11_c10,
    action_handlers::handle_bad_action,
    // elephant1: 0b101_0000 to 0b101_1111
    action_handlers::elephant1::r00_c00,
    action_handlers::elephant1::r00_c01,
    action_handlers::elephant1::r00_c10,
    action_handlers::handle_bad_action,
    action_handlers::elephant1::r01_c00,
    action_handlers::elephant1::r01_c01,
    action_handlers::elephant1::r01_c10,
    action_handlers::handle_bad_action,
    action_handlers::elephant1::r10_c00,
    action_handlers::elephant1::r10_c01,
    action_handlers::elephant1::r10_c10,
    action_handlers::handle_bad_action,
    action_handlers::elephant1::r11_c00,
    action_handlers::elephant1::r11_c01,
    action_handlers::elephant1::r11_c10,
    action_handlers::handle_bad_action,
    // giraffe0: 0b110_0000 to 0b110_1111
    action_handlers::giraffe0::r00_c00,
    action_handlers::giraffe0::r00_c01,
    action_handlers::giraffe0::r00_c10,
    action_handlers::handle_bad_action,
    action_handlers::giraffe0::r01_c00,
    action_handlers::giraffe0::r01_c01,
    action_handlers::giraffe0::r01_c10,
    action_handlers::handle_bad_action,
    action_handlers::giraffe0::r10_c00,
    action_handlers::giraffe0::r10_c01,
    action_handlers::giraffe0::r10_c10,
    action_handlers::handle_bad_action,
    action_handlers::giraffe0::r11_c00,
    action_handlers::giraffe0::r11_c01,
    action_handlers::giraffe0::r11_c10,
    action_handlers::handle_bad_action,
    // giraffe1: 0b111_0000 to 0b111_1111
    action_handlers::giraffe1::r00_c00,
    action_handlers::giraffe1::r00_c01,
    action_handlers::giraffe1::r00_c10,
    action_handlers::handle_bad_action,
    action_handlers::giraffe1::r01_c00,
    action_handlers::giraffe1::r01_c01,
    action_handlers::giraffe1::r01_c10,
    action_handlers::handle_bad_action,
    action_handlers::giraffe1::r10_c00,
    action_handlers::giraffe1::r10_c01,
    action_handlers::giraffe1::r10_c10,
    action_handlers::handle_bad_action,
    action_handlers::giraffe1::r11_c00,
    action_handlers::giraffe1::r11_c01,
    action_handlers::giraffe1::r11_c10,
    action_handlers::handle_bad_action,
];

macro_rules! define_piece_action_handlers {
    ($name:ident, $piece:literal) => {
        pub mod $name {
            use super::*;

            pub const fn r00_c00(state: SearchNode) -> (OptionalNodeBuilder, OptionalAction) {
                state
                    .into_builder()
                    .handle_action(Action(($piece << 4) | 0b0000))
            }

            pub const fn r00_c01(state: SearchNode) -> (OptionalNodeBuilder, OptionalAction) {
                state
                    .into_builder()
                    .handle_action(Action(($piece << 4) | 0b0001))
            }

            pub const fn r00_c10(state: SearchNode) -> (OptionalNodeBuilder, OptionalAction) {
                state
                    .into_builder()
                    .handle_action(Action(($piece << 4) | 0b0010))
            }

            pub const fn r01_c00(state: SearchNode) -> (OptionalNodeBuilder, OptionalAction) {
                state
                    .into_builder()
                    .handle_action(Action(($piece << 4) | 0b0100))
            }

            pub const fn r01_c01(state: SearchNode) -> (OptionalNodeBuilder, OptionalAction) {
                state
                    .into_builder()
                    .handle_action(Action(($piece << 4) | 0b0101))
            }

            pub const fn r01_c10(state: SearchNode) -> (OptionalNodeBuilder, OptionalAction) {
                state
                    .into_builder()
                    .handle_action(Action(($piece << 4) | 0b0110))
            }

            pub const fn r10_c00(state: SearchNode) -> (OptionalNodeBuilder, OptionalAction) {
                state
                    .into_builder()
                    .handle_action(Action(($piece << 4) | 0b1000))
            }

            pub const fn r10_c01(state: SearchNode) -> (OptionalNodeBuilder, OptionalAction) {
                state
                    .into_builder()
                    .handle_action(Action(($piece << 4) | 0b1001))
            }

            pub const fn r10_c10(state: SearchNode) -> (OptionalNodeBuilder, OptionalAction) {
                state
                    .into_builder()
                    .handle_action(Action(($piece << 4) | 0b1010))
            }

            pub const fn r11_c00(state: SearchNode) -> (OptionalNodeBuilder, OptionalAction) {
                state
                    .into_builder()
                    .handle_action(Action(($piece << 4) | 0b1100))
            }

            pub const fn r11_c01(state: SearchNode) -> (OptionalNodeBuilder, OptionalAction) {
                state
                    .into_builder()
                    .handle_action(Action(($piece << 4) | 0b1101))
            }

            pub const fn r11_c10(state: SearchNode) -> (OptionalNodeBuilder, OptionalAction) {
                state
                    .into_builder()
                    .handle_action(Action(($piece << 4) | 0b1110))
            }
        }
    };
}
mod action_handlers {
    use super::*;

    define_piece_action_handlers!(active_lion, 0b001);
    define_piece_action_handlers!(chick0, 0b010);
    define_piece_action_handlers!(chick1, 0b011);
    define_piece_action_handlers!(elephant0, 0b100);
    define_piece_action_handlers!(elephant1, 0b101);
    define_piece_action_handlers!(giraffe0, 0b110);
    define_piece_action_handlers!(giraffe1, 0b111);

    pub fn handle_bad_action(_: SearchNode) -> (OptionalNodeBuilder, OptionalAction) {
        panic!("Illegal action");
    }
}

impl NodeBuilder {
    #[inline(always)]
    const fn handle_action(self, action: Action) -> (OptionalNodeBuilder, OptionalAction) {
        if self.0 & action.allegiance_mask() != 0 {
            return (OptionalNodeBuilder::NONE, action.next_species_action());
        }

        if self.0 & action.hand_mask() == action.hand_mask() {
            return self.handle_drop_assuming_actor_is_active_and_in_hand(action);
        }

        if self.is_actor_out_of_range_of_dest_square(action) {
            return (
                OptionalNodeBuilder::NONE,
                self.next_action_with_nonactive_dest_square_in_current_actor_range(
                    action,
                    self.board(),
                ),
            );
        }

        return self.handle_move_assuming_actor_is_active_and_in_range_of_dest_square(action);
    }

    #[inline(always)]
    const fn handle_drop_assuming_actor_is_active_and_in_hand(
        self,
        action: Action,
    ) -> (OptionalNodeBuilder, OptionalAction) {
        let original_state = self;
        let original_board = original_state.board();

        if original_board.is_dest_square_occupied(action) {
            return (OptionalNodeBuilder::NONE, action.next_species_action());
        }

        let state = original_state.move_acting_piece_to_dest_square(action);
        let next_action = original_state.next_empty_square_action(action);
        (state.into_optional(), next_action)
    }

    #[inline(always)]
    const fn move_acting_piece_to_dest_square(self, action: Action) -> NodeBuilder {
        todo!()
    }

    #[inline(always)]
    const fn next_empty_square_action(self, action: Action) -> OptionalAction {
        todo!()
    }

    #[inline(always)]
    const fn is_actor_out_of_range_of_dest_square(self, action: Action) -> bool {
        todo!()
    }

    #[inline(always)]
    const fn next_action_with_nonactive_dest_square_in_current_actor_range(
        self,
        action: Action,
        board: Board,
    ) -> OptionalAction {
        todo!()
    }

    #[inline(always)]
    const fn handle_move_assuming_actor_is_active_and_in_range_of_dest_square(
        self,
        action: Action,
    ) -> (OptionalNodeBuilder, OptionalAction) {
        let original_state = self;
        let original_board = original_state.board();
        let state = original_state.vacate_passive_dest_square(action, original_board);

        // If the destination square is occupied by an active piece,
        // then the move is illegal.
        if state.is_none() {
            return (
                OptionalNodeBuilder::NONE,
                original_state.next_action_with_nonactive_dest_square_in_current_actor_range(
                    action,
                    original_board,
                ),
            );
        }

        let state = state.unchecked_unwrap();
        let state = state.move_acting_piece_to_dest_square(action);
        let state = state.promote_actor_if_needed(action);
        let next_action = original_state
            .next_action_with_nonactive_dest_square_in_current_actor_range(action, original_board);
        (state.into_optional(), next_action)
    }

    /// - If the destination square is empty, this returns the original state.
    /// - If the destination square is occupied by a passive piece,
    ///   this returns the state with the passive piece moved to the active player's hand.
    /// - If the destination square is occupied by an active piece,
    ///   this returns `OptionalNodeBuilder::NONE`.
    #[inline(always)]
    const fn vacate_passive_dest_square(self, action: Action, board: Board) -> OptionalNodeBuilder {
        todo!()
    }

    #[inline(always)]
    const fn promote_actor_if_needed(self, action: Action) -> NodeBuilder {
        todo!()
    }
}

impl Board {
    #[inline(always)]
    const fn is_dest_square_occupied(self, action: Action) -> bool {
        // TODO
        false
    }
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

    // Returns a mask `m` that can be `&`ed with the state to produce
    // some number `n` such that `n == m` if and only if the actor is in hand.
    const fn hand_mask(self) -> u64 {
        let offset = match self.0 >> 4 {
            0b001 => offsets::ACTIVE_LION_COLUMN,
            0b010 => offsets::CHICK0_COLUMN,
            0b011 => offsets::CHICK1_COLUMN,
            0b100 => offsets::ELEPHANT0_COLUMN,
            0b101 => offsets::ELEPHANT1_COLUMN,
            0b110 => offsets::GIRAFFE0_COLUMN,
            0b111 => offsets::GIRAFFE1_COLUMN,

            _ => return 0,
        };

        0b1111 << offset
    }
}

/// All offsets are given relative to the right (i.e., least significant) bit.
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

    pub mod square {
        pub const SPECIES: u64 = 0;
        pub const PIECE_NUMBER: u64 = SPECIES + 3;
        pub const ALLEGIANCE: u64 = PIECE_NUMBER + 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positive1_i9_converts_to_i16_correctly() {
        let actual = i16::from_zero_padded_i9(1);
        let expected: i16 = 1;
        assert_eq!(expected, actual);
    }

    #[test]
    fn zero_i9_converts_to_i16_correctly() {
        let actual = i16::from_zero_padded_i9(0);
        let expected: i16 = 0;
        assert_eq!(expected, actual);
    }

    #[test]
    fn negative1_i9_converts_to_i16_correctly() {
        let actual = i16::from_zero_padded_i9(0b1_1111_1111);
        let expected: i16 = -1;
        assert_eq!(expected, actual);
    }

    #[test]
    fn negative201_i9_converts_to_i16_correctly() {
        let actual = i16::from_zero_padded_i9(NEGATIVE_201_I9);
        let expected: i16 = -201;
        assert_eq!(expected, actual);
    }

    #[test]
    fn positive1_i16_converts_to_i9_correctly() {
        let actual = 1i16.into_zero_padded_i9_unchecked();
        let expected: u64 = 1;
        assert_eq!(expected, actual);
    }

    #[test]
    fn zero_i16_converts_to_i9_correctly() {
        let actual = 0i16.into_zero_padded_i9_unchecked();
        let expected: u64 = 0;
        assert_eq!(expected, actual);
    }

    #[test]
    fn negative1_i16_converts_to_i9_correctly() {
        let actual = (-1i16).into_zero_padded_i9_unchecked();
        let expected: u64 = 0b1_1111_1111;
        assert_eq!(expected, actual);
    }

    #[test]
    fn negative201_i16_converts_to_i9_correctly() {
        let actual = (-201i16).into_zero_padded_i9_unchecked();
        let expected: u64 = NEGATIVE_201_I9;
        assert_eq!(expected, actual);
    }
}
