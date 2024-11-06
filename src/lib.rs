// A note about fileds with the comment "This must be non-zero":
// I know we _could_ use a `NonZeroU64` (or another respective `NonZero*` type),
// but that would clutter the code with a bunch of unwraps,
// which hurts readability and performance.

#[cfg(test)]
mod tests;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SquareSet(u16);

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

        let next_action: u64 = 0b001_0000;

        let best_discovered_outcome: u64 = NEGATIVE_201_I9;

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
                | (next_action << offsets::NEXT_ACTION)
                | (best_discovered_outcome << offsets::BEST_DISCOVERED_OUTCOME),
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

    const fn next_action(self) -> Result<Action, Solution> {
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

    const fn set_next_action(self, next_action: OptionalAction) -> Self {
        let raw = next_action.0 as u64;
        Self((self.0 & !(0b111_1111 << 9)) | (raw << 9))
    }
}

impl NodeBuilder {
    const fn invert_active_player(self) -> Self {
        const NONLION_ALLEGIANCE_INVERSION_MASK: u64 = (1 << offsets::CHICK0_ALLEGIANCE)
            | (1 << offsets::CHICK1_ALLEGIANCE)
            | (1 << offsets::ELEPHANT0_ALLEGIANCE)
            | (1 << offsets::ELEPHANT1_ALLEGIANCE)
            | (1 << offsets::GIRAFFE0_ALLEGIANCE)
            | (1 << offsets::GIRAFFE1_ALLEGIANCE);

        let chick0_inverted_coords = self.invert_coords_at_offset(offsets::CHICK0_COLUMN);
        let chick1_inverted_coords = self.invert_coords_at_offset(offsets::CHICK1_COLUMN);
        let elephant0_inverted_coords = self.invert_coords_at_offset(offsets::ELEPHANT0_COLUMN);
        let elephant1_inverted_coords = self.invert_coords_at_offset(offsets::ELEPHANT1_COLUMN);
        let giraffe0_inverted_coords = self.invert_coords_at_offset(offsets::GIRAFFE0_COLUMN);
        let giraffe1_inverted_coords = self.invert_coords_at_offset(offsets::GIRAFFE1_COLUMN);
        let active_lion_inverted_coords = self.invert_coords_at_offset(offsets::ACTIVE_LION_COLUMN);
        let passive_lion_inverted_coords =
            self.invert_coords_at_offset(offsets::PASSIVE_LION_COLUMN);

        const ALL_COORDS_MASK: u64 = (0b1111 << offsets::CHICK0_COLUMN)
            | (0b1111 << offsets::CHICK1_COLUMN)
            | (0b1111 << offsets::ELEPHANT0_COLUMN)
            | (0b1111 << offsets::ELEPHANT1_COLUMN)
            | (0b1111 << offsets::GIRAFFE0_COLUMN)
            | (0b1111 << offsets::GIRAFFE1_COLUMN)
            | (0b1111 << offsets::ACTIVE_LION_COLUMN)
            | (0b1111 << offsets::PASSIVE_LION_COLUMN);

        Self(
            (
                (self.0 & !ALL_COORDS_MASK)
                // First, we invert the allegiance bits of non-lions.
                ^ NONLION_ALLEGIANCE_INVERSION_MASK
            )
            // Then, we invert the coordinates of the non-lions.
            | chick0_inverted_coords
            | chick1_inverted_coords
            | elephant0_inverted_coords
            | elephant1_inverted_coords
            | giraffe0_inverted_coords
            | giraffe1_inverted_coords
            // Finally, we invert the coordinates of the lions,
            // while simultaneously swapping their positions
            // (i.e., putting the active lion in the passive lion location,
            // and vice-versa).
            | (active_lion_inverted_coords
                >> (offsets::ACTIVE_LION - offsets::PASSIVE_LION))
            | (passive_lion_inverted_coords
                << (offsets::ACTIVE_LION - offsets::PASSIVE_LION)),
        )
    }

    #[inline(always)]
    const fn invert_coords_at_offset(self, coords_offset: u64) -> u64 {
        let coords_mask = 0b1111 << coords_offset;
        let r3c2 = 0b1110 << coords_offset;
        let coords = self.0 & coords_mask;
        if coords == coords_mask {
            return coords;
        }
        r3c2 - coords
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
            let chick0_col = self.0 & (0b11 << offsets::CHICK0_COLUMN);
            let chick0_col_flipped = (0b10 << offsets::CHICK0_COLUMN) - chick0_col;
            const CHICK0_ROW_MASK: u64 = 0b11 << offsets::CHICK0_ROW;
            (chick0_coords & CHICK0_ROW_MASK) | chick0_col_flipped
        };

        const CHICK1_COORDS_MASK: u64 = 0b1111 << offsets::CHICK1_COLUMN;
        let chick1_coords = self.0 & CHICK1_COORDS_MASK;
        let chick1_coords_flipped = if chick1_coords == CHICK1_COORDS_MASK {
            chick1_coords
        } else {
            let chick1_col = self.0 & (0b11 << offsets::CHICK1_COLUMN);
            let chick1_col_flipped = (0b10 << offsets::CHICK1_COLUMN) - chick1_col;
            const CHICK1_ROW_MASK: u64 = 0b11 << offsets::CHICK1_ROW;
            (chick1_coords & CHICK1_ROW_MASK) | chick1_col_flipped
        };

        const ELEPHANT0_COORDS_MASK: u64 = 0b1111 << offsets::ELEPHANT0_COLUMN;
        let elephant0_coords = self.0 & ELEPHANT0_COORDS_MASK;
        let elephant0_coords_flipped = if elephant0_coords == ELEPHANT0_COORDS_MASK {
            elephant0_coords
        } else {
            let elephant0_col = self.0 & (0b11 << offsets::ELEPHANT0_COLUMN);
            let elephant0_col_flipped = (0b10 << offsets::ELEPHANT0_COLUMN) - elephant0_col;
            const ELEPHANT0_ROW_MASK: u64 = 0b11 << offsets::ELEPHANT0_ROW;
            (elephant0_coords & ELEPHANT0_ROW_MASK) | elephant0_col_flipped
        };

        const ELEPHANT1_COORDS_MASK: u64 = 0b1111 << offsets::ELEPHANT1_COLUMN;
        let elephant1_coords = self.0 & ELEPHANT1_COORDS_MASK;
        let elephant1_coords_flipped = if elephant1_coords == ELEPHANT1_COORDS_MASK {
            elephant1_coords
        } else {
            let elephant1_col = self.0 & (0b11 << offsets::ELEPHANT1_COLUMN);
            let elephant1_col_flipped = (0b10 << offsets::ELEPHANT1_COLUMN) - elephant1_col;
            const ELEPHANT1_ROW_MASK: u64 = 0b11 << offsets::ELEPHANT1_ROW;
            (elephant1_coords & ELEPHANT1_ROW_MASK) | elephant1_col_flipped
        };

        const GIRAFFE0_COORDS_MASK: u64 = 0b1111 << offsets::GIRAFFE0_COLUMN;
        let giraffe0_coords = self.0 & GIRAFFE0_COORDS_MASK;
        let giraffe0_coords_flipped = if giraffe0_coords == GIRAFFE0_COORDS_MASK {
            giraffe0_coords
        } else {
            let giraffe0_col = self.0 & (0b11 << offsets::GIRAFFE0_COLUMN);
            let giraffe0_col_flipped = (0b10 << offsets::GIRAFFE0_COLUMN) - giraffe0_col;
            const GIRAFFE0_ROW_MASK: u64 = 0b11 << offsets::GIRAFFE0_ROW;
            (giraffe0_coords & GIRAFFE0_ROW_MASK) | giraffe0_col_flipped
        };

        const GIRAFFE1_COORDS_MASK: u64 = 0b1111 << offsets::GIRAFFE1_COLUMN;
        let giraffe1_coords = self.0 & GIRAFFE1_COORDS_MASK;
        let giraffe1_coords_flipped = if giraffe1_coords == GIRAFFE1_COORDS_MASK {
            giraffe1_coords
        } else {
            let giraffe1_col = self.0 & (0b11 << offsets::GIRAFFE1_COLUMN);
            let giraffe1_col_flipped = (0b10 << offsets::GIRAFFE1_COLUMN) - giraffe1_col;
            const GIRAFFE1_ROW_MASK: u64 = 0b11 << offsets::GIRAFFE1_ROW;
            (giraffe1_coords & GIRAFFE1_ROW_MASK) | giraffe1_col_flipped
        };

        const ACTIVE_LION_COORDS_MASK: u64 = 0b11 << offsets::ACTIVE_LION_COLUMN;
        let active_lion_coords = self.0 & ACTIVE_LION_COORDS_MASK;
        let active_lion_coords_flipped = if active_lion_coords == ACTIVE_LION_COORDS_MASK {
            active_lion_coords
        } else {
            let active_lion_col = self.0 & (0b11 << offsets::ACTIVE_LION_COLUMN);
            let active_lion_col_flipped = (0b10 << offsets::ACTIVE_LION_COLUMN) - active_lion_col;
            const ACTIVE_LION_ROW_MASK: u64 = 0b11 << offsets::ACTIVE_LION_ROW;
            (active_lion_coords & ACTIVE_LION_ROW_MASK) | active_lion_col_flipped
        };

        const PASSIVE_LION_COORDS_MASK: u64 = 0b11 << offsets::PASSIVE_LION_COLUMN;
        let passive_lion_coords = self.0 & PASSIVE_LION_COORDS_MASK;
        let passive_lion_coords_flipped = if passive_lion_coords == PASSIVE_LION_COORDS_MASK {
            passive_lion_coords
        } else {
            let passive_lion_col = self.0 & (0b11 << offsets::PASSIVE_LION_COLUMN);
            let passive_lion_col_flipped =
                (0b10 << offsets::PASSIVE_LION_COLUMN) - passive_lion_col;
            const PASSIVE_LION_ROW_MASK: u64 = 0b11 << offsets::PASSIVE_LION_ROW;
            (passive_lion_coords & PASSIVE_LION_ROW_MASK) | passive_lion_col_flipped
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

macro_rules! define_piece_action_handler {
    ($piece:literal, $name:ident, $dest_coords:literal) => {
        pub const fn $name(state: SearchNode) -> (OptionalNodeBuilder, OptionalAction) {
            state
                .into_builder()
                .handle_action(Action(($piece << 4) | $dest_coords))
        }
    };
}

macro_rules! define_piece_action_handlers {
    ($name:ident, $piece:literal) => {
        pub mod $name {
            use super::*;

            define_piece_action_handler!($piece, r00_c00, 0b0000);
            define_piece_action_handler!($piece, r00_c01, 0b0001);
            define_piece_action_handler!($piece, r00_c10, 0b0010);

            define_piece_action_handler!($piece, r01_c00, 0b0100);
            define_piece_action_handler!($piece, r01_c01, 0b0101);
            define_piece_action_handler!($piece, r01_c10, 0b0110);

            define_piece_action_handler!($piece, r10_c00, 0b1000);
            define_piece_action_handler!($piece, r10_c01, 0b1001);
            define_piece_action_handler!($piece, r10_c10, 0b1010);

            define_piece_action_handler!($piece, r11_c00, 0b1100);
            define_piece_action_handler!($piece, r11_c01, 0b1101);
            define_piece_action_handler!($piece, r11_c10, 0b1110);
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

        if self.0 & action.coords_mask() == action.coords_mask() {
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
        let next_action = original_state.next_empty_square_action(action, original_board);

        if original_board.is_dest_square_occupied(action) {
            return (OptionalNodeBuilder::NONE, next_action);
        }

        let state = original_state.move_actor_to_dest_square(action);
        (state.into_optional(), next_action)
    }

    #[inline(always)]
    const fn move_actor_to_dest_square(self, action: Action) -> NodeBuilder {
        Self(
            (self.0 & !action.coords_mask())
                | action.dest_square_coords_shifted_by_actor_coords_offset(),
        )
    }

    #[inline(always)]
    const fn next_empty_square_action(self, action: Action, board: Board) -> OptionalAction {
        macro_rules! check_square {
            ($coords:literal) => {
                if action.0 & 0b1111 < $coords
                    && board.is_square_empty_at_board_offset(coords_to_board_offset($coords))
                {
                    return action.set_dest_square($coords).into_optional();
                }
            };
        }

        check_square!(0b0001);
        check_square!(0b0010);

        check_square!(0b0100);
        check_square!(0b0101);
        check_square!(0b0110);

        check_square!(0b1000);
        check_square!(0b1001);
        check_square!(0b1010);

        check_square!(0b1100);
        check_square!(0b1101);
        check_square!(0b1110);

        action.next_piece_action()
    }

    #[inline(always)]
    const fn is_actor_out_of_range_of_dest_square(self, action: Action) -> bool {
        let actor_coords = (self.0 >> action.actor_coords_offset()) & 0b1111;

        let legal_squares = action.legal_starting_squares();

        // Only pieces on the board can be in range.
        // Therefore, if the actor is in hand, it is out of range.
        // However, we don't have to explicitly check this case,
        // because if the actor is in hand, then `actor_coords == 15`,
        // and bit 15 of `legal_squares` is guaranteed to be 0.
        legal_squares[self.is_actor_promoted(action) as usize].0 & (1 << actor_coords) == 0
    }

    #[inline(always)]
    const fn is_actor_promoted(self, action: Action) -> bool {
        let offset = match action.0 >> 4 {
            0b010 => offsets::CHICK0_PROMOTION,
            0b011 => offsets::CHICK1_PROMOTION,

            _ => return false,
        };
        self.0 & (1 << offset) != 0
    }

    #[inline(always)]
    const fn is_actor_in_range_of_dest_square(self, action: Action) -> bool {
        !self.is_actor_out_of_range_of_dest_square(action)
    }

    #[inline(always)]
    const fn next_action_with_nonactive_dest_square_in_current_actor_range(
        self,
        action: Action,
        board: Board,
    ) -> OptionalAction {
        macro_rules! check_square {
            ($coords:literal) => {{
                let candidate = action.set_dest_square($coords);

                if action.0 & 0b1111 < $coords
                    && board.is_dest_square_nonactive(candidate)
                    && self.is_actor_in_range_of_dest_square(candidate)
                {
                    return candidate.into_optional();
                }
            }};
        }

        check_square!(0b0001);
        check_square!(0b0010);

        check_square!(0b0100);
        check_square!(0b0101);
        check_square!(0b0110);

        check_square!(0b1000);
        check_square!(0b1001);
        check_square!(0b1010);

        check_square!(0b1100);
        check_square!(0b1101);
        check_square!(0b1110);

        action.next_piece_action()
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
        let state = state.move_actor_to_dest_square(action);
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
        if board.is_dest_square_empty(action) {
            return self.into_optional();
        }

        // If the destination square is non-empty
        // and non-passive, then it must be active.
        // Therefore, we return `None`.
        if board.is_dest_square_nonpassive(action) {
            return OptionalNodeBuilder::NONE;
        }

        let occupant = (board.0 >> action.dest_square_board_offset()) & 0b111;
        let occupant_lookup_index = (occupant - 1) as usize;

        let occupant_coords_offset = [
            offsets::ACTIVE_LION_COLUMN,
            offsets::CHICK0_COLUMN,
            offsets::CHICK1_COLUMN,
            offsets::ELEPHANT0_COLUMN,
            offsets::ELEPHANT1_COLUMN,
            offsets::GIRAFFE0_COLUMN,
            offsets::GIRAFFE1_COLUMN,
        ][occupant_lookup_index];

        let is_occupant_nonlion = occupant != 0b001;
        // If the occupant is a non-lion, we need to set the allegiance bit to 0.
        // The allegiance bit is 4 bits left of the column offset.
        let allegiance_mask = !((is_occupant_nonlion as u64) << (occupant_coords_offset + 4));

        let is_occupant_chick = occupant & !1 == 0b010;
        // If the occupant is a chick, we need to set the promotion bit to 0.
        // The promotion bit is 1 bit right of the column offset.
        let demotion_mask = !((is_occupant_chick as u64) << (occupant_coords_offset - 1));

        Self((self.0 | (0b1111 << occupant_coords_offset)) & allegiance_mask & demotion_mask)
            .into_optional()
    }

    #[inline(always)]
    const fn promote_actor_if_needed(self, action: Action) -> NodeBuilder {
        if action.is_actor_chick0() {
            let coords = self.0 & action.coords_mask();
            let promotion_bit = (((coords != action.coords_mask())
                & (coords >= (0b1100 << offsets::CHICK0_COLUMN)))
                as u64)
                << offsets::CHICK0_PROMOTION;
            return Self(self.0 | promotion_bit);
        }

        if action.is_actor_chick1() {
            let coords = self.0 & action.coords_mask();
            let promotion_bit = (((coords != action.coords_mask())
                & (coords >= (0b1100 << offsets::CHICK1_COLUMN)))
                as u64)
                << offsets::CHICK1_PROMOTION;
            return Self(self.0 | promotion_bit);
        }

        self
    }
}

impl Board {
    #[inline(always)]
    const fn is_dest_square_empty(self, action: Action) -> bool {
        self.is_square_empty_at_board_offset(action.dest_square_board_offset())
    }

    #[inline(always)]
    const fn is_square_empty_at_board_offset(self, board_offset: u64) -> bool {
        self.0 & (0b111 << board_offset) == 0
    }

    #[inline(always)]
    const fn is_dest_square_occupied(self, action: Action) -> bool {
        self.0 & (0b111 << action.dest_square_board_offset()) != 0
    }

    #[inline(always)]
    const fn is_dest_square_nonpassive(self, action: Action) -> bool {
        self.0 & (0b1_000 << action.dest_square_board_offset()) == 0
    }

    #[inline(always)]
    const fn is_dest_square_nonactive(self, action: Action) -> bool {
        let is_passive = self.0 & (0b1_000 << action.dest_square_board_offset()) != 0;
        self.is_dest_square_empty(action) | is_passive
    }
}

impl Action {
    #[inline(always)]
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

    #[inline(always)]
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

    #[inline(always)]
    const fn next_piece_action(self) -> OptionalAction {
        OptionalAction(match self.0 >> 4 {
            0b001 => 0b010_0000,
            0b010 => 0b011_0000,
            0b011 => 0b100_0000,
            0b100 => 0b101_0000,
            0b101 => 0b110_0000,
            0b110 => 0b111_0000,
            0b111 => 0,

            _ => 0,
        })
    }

    #[inline(always)]
    const fn coords_mask(self) -> u64 {
        0b1111 << self.actor_coords_offset()
    }

    #[inline(always)]
    const fn dest_square_coords_shifted_by_actor_coords_offset(self) -> u64 {
        ((self.0 as u64) & 0b1111) << self.actor_coords_offset()
    }

    #[inline(always)]
    const fn actor_coords_offset(self) -> u64 {
        match self.0 >> 4 {
            0b001 => offsets::ACTIVE_LION_COLUMN,
            0b010 => offsets::CHICK0_COLUMN,
            0b011 => offsets::CHICK1_COLUMN,
            0b100 => offsets::ELEPHANT0_COLUMN,
            0b101 => offsets::ELEPHANT1_COLUMN,
            0b110 => offsets::GIRAFFE0_COLUMN,
            0b111 => offsets::GIRAFFE1_COLUMN,

            _ => 0,
        }
    }

    #[inline(always)]
    const fn dest_square_board_offset(self) -> u64 {
        coords_to_board_offset((self.0 as u64) & 0b1111)
    }

    #[inline(always)]
    const fn is_actor_chick0(self) -> bool {
        self.0 >> 4 == 0b010
    }

    #[inline(always)]
    const fn is_actor_chick1(self) -> bool {
        self.0 >> 4 == 0b011
    }

    #[inline(always)]
    const fn set_dest_square(self, coords: u8) -> Action {
        Self((self.0 & !0b1111) | coords)
    }

    #[inline(always)]
    const fn into_optional(self) -> OptionalAction {
        OptionalAction(self.0)
    }

    /// The set of legal starting squares depends on whether the
    /// actor is promoted.
    /// We cannot determine this from the action alone.
    ///
    /// So, we return an array of two sets.
    /// The first set is for the non-promoted actor,
    /// and the second set is for the promoted actor.
    ///
    /// It is the consumer's responsibility to select the correct
    /// set to use.
    #[inline(always)]
    const fn legal_starting_squares(self) -> [SquareSet; 2] {
        /// This function should only be called during compile-time.
        /// Consequently, we don't have to worry about the performance
        /// inside of it.
        /// Thus, we can use a simple struct with 8 boolean fields
        /// instead of a more efficient `u8` bitset.
        #[derive(Copy, Clone)]
        struct DirectionSet {
            n: bool,
            ne: bool,
            e: bool,
            se: bool,
            s: bool,
            sw: bool,
            w: bool,
            nw: bool,
        }

        impl DirectionSet {
            const fn union(self, rhs: Self) -> Self {
                Self {
                    n: self.n | rhs.n,
                    ne: self.ne | rhs.ne,
                    e: self.e | rhs.e,
                    se: self.se | rhs.se,
                    s: self.s | rhs.s,
                    sw: self.sw | rhs.sw,
                    w: self.w | rhs.w,
                    nw: self.nw | rhs.nw,
                }
            }

            /// Returns whether there is a way to move from `from` to `to`
            /// by taking one step in some direction contained in this set.
            const fn connects(self, from: u8, to: u8) -> bool {
                let from_column = (from & 0b11) as i8;
                let from_row = (from >> 2) as i8;
                let to_column = (to & 0b11) as i8;
                let to_row = (to >> 2) as i8;

                (self.n && from_row + 1 == to_row && from_column == to_column)
                    || (self.ne && from_row + 1 == to_row && from_column + 1 == to_column)
                    || (self.e && from_row == to_row && from_column + 1 == to_column)
                    || (self.se && from_row - 1 == to_row && from_column + 1 == to_column)
                    || (self.s && from_row - 1 == to_row && from_column == to_column)
                    || (self.sw && from_row - 1 == to_row && from_column - 1 == to_column)
                    || (self.w && from_row == to_row && from_column - 1 == to_column)
                    || (self.nw && from_row + 1 == to_row && from_column - 1 == to_column)
            }
        }

        const EMPTY: DirectionSet = DirectionSet {
            n: false,
            ne: false,
            e: false,
            se: false,
            s: false,
            sw: false,
            w: false,
            nw: false,
        };
        const N: DirectionSet = DirectionSet { n: true, ..EMPTY };
        const NE: DirectionSet = DirectionSet { ne: true, ..EMPTY };
        const E: DirectionSet = DirectionSet { e: true, ..EMPTY };
        const SE: DirectionSet = DirectionSet { se: true, ..EMPTY };
        const S: DirectionSet = DirectionSet { s: true, ..EMPTY };
        const SW: DirectionSet = DirectionSet { sw: true, ..EMPTY };
        const W: DirectionSet = DirectionSet { w: true, ..EMPTY };
        const NW: DirectionSet = DirectionSet { nw: true, ..EMPTY };

        const CARDINAL: DirectionSet = N.union(E).union(S).union(W);
        const DIAGONAL: DirectionSet = NE.union(SE).union(SW).union(NW);

        const CHICK: DirectionSet = N;
        const HEN: DirectionSet = CARDINAL.union(NE).union(NW);
        const ELEPHANT: DirectionSet = DIAGONAL;
        const GIRAFFE: DirectionSet = CARDINAL;
        const LION: DirectionSet = CARDINAL.union(DIAGONAL);

        let [nonpromoted_dirset, promoted_dirset] = match self.0 >> 4 {
            0b001 => [LION, EMPTY],
            0b010 => [CHICK, HEN],
            0b011 => [CHICK, HEN],
            0b100 => [ELEPHANT, EMPTY],
            0b101 => [ELEPHANT, EMPTY],
            0b110 => [GIRAFFE, EMPTY],
            0b111 => [GIRAFFE, EMPTY],

            _ => [EMPTY, EMPTY],
        };

        let dest_coords = self.0 & 0b1111;

        let nonpromoted_squares = {
            let mut out: u16 = 0;

            macro_rules! check_start_square {
                ($start_square:literal) => {
                    if nonpromoted_dirset.connects($start_square, dest_coords) {
                        out |= 1 << $start_square;
                    }
                };
            }

            check_start_square!(0b0000);
            check_start_square!(0b0001);
            check_start_square!(0b0010);

            check_start_square!(0b0100);
            check_start_square!(0b0101);
            check_start_square!(0b0110);

            check_start_square!(0b1000);
            check_start_square!(0b1001);
            check_start_square!(0b1010);

            check_start_square!(0b1100);
            check_start_square!(0b1101);
            check_start_square!(0b1110);

            out
        };
        let promoted_squares = {
            let mut out: u16 = 0;

            macro_rules! check_start_square {
                ($start_square:literal) => {
                    if promoted_dirset.connects($start_square, dest_coords) {
                        out |= 1 << $start_square;
                    }
                };
            }

            check_start_square!(0b0000);
            check_start_square!(0b0001);
            check_start_square!(0b0010);

            check_start_square!(0b0100);
            check_start_square!(0b0101);
            check_start_square!(0b0110);

            check_start_square!(0b1000);
            check_start_square!(0b1001);
            check_start_square!(0b1010);

            check_start_square!(0b1100);
            check_start_square!(0b1101);
            check_start_square!(0b1110);

            out
        };

        [SquareSet(nonpromoted_squares), SquareSet(promoted_squares)]
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
    pub const ELEPHANT0_ALLEGIANCE: u64 = ELEPHANT0_ROW + 2;

    pub const ELEPHANT1_COLUMN: u64 = ELEPHANT1;
    pub const ELEPHANT1_ROW: u64 = ELEPHANT1_COLUMN + 2;
    pub const ELEPHANT1_ALLEGIANCE: u64 = ELEPHANT1_ROW + 2;

    pub const GIRAFFE0_COLUMN: u64 = GIRAFFE0;
    pub const GIRAFFE0_ROW: u64 = GIRAFFE0_COLUMN + 2;
    pub const GIRAFFE0_ALLEGIANCE: u64 = GIRAFFE0_ROW + 2;

    pub const GIRAFFE1_COLUMN: u64 = GIRAFFE1;
    pub const GIRAFFE1_ROW: u64 = GIRAFFE1_COLUMN + 2;
    pub const GIRAFFE1_ALLEGIANCE: u64 = GIRAFFE1_ROW + 2;

    pub const ACTIVE_LION_COLUMN: u64 = ACTIVE_LION;
    pub const ACTIVE_LION_ROW: u64 = ACTIVE_LION_COLUMN + 2;

    pub const PASSIVE_LION_COLUMN: u64 = PASSIVE_LION;
    pub const PASSIVE_LION_ROW: u64 = PASSIVE_LION_COLUMN + 2;
}
