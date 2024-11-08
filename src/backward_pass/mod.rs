use super::*;

#[cfg(test)]
mod tests;

/// This function will solve the game when provided
/// with a slice of all reachable states.
///
/// The slice of states will be sorted.
pub fn solve(states: &mut [SearchNode]) {
    states.sort_unstable();

    init_unknown_child_count_and_best_known_outcome(states);

    let mut known_stack = vec![];
    add_terminal_nodes(states, &mut known_stack);

    let mut parent_buffer = vec![];

    while let Some(top) = known_stack.pop() {
        let outcome = top.best_known_outcome();

        top.write_parents(&mut parent_buffer);

        for parent in parent_buffer.iter().copied() {
            let Ok(parent_index) = states.binary_search(&parent) else {
                // It's possible that a theoretical parent is actually unreachable.
                continue;
            };

            let parent_mut = &mut states[parent_index];
            *parent_mut = parent_mut.record_child_outcome(outcome);
            if parent_mut.unknown_child_count() == 0 {
                known_stack.push(*parent_mut);
            }
        }
    }
}

///  - `0` represents a draw.
///
///  - A positive number `n` represents a win for the active player
///    in `201 - n` plies.
///
///  - A negative number `-n` represents a win for the passive player
///    in `201 + n` plies.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Outcome(i16);

trait FromZeroPaddedI9<T> {
    fn from_zero_padded_i9(value: T) -> Self;
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

fn init_unknown_child_count_and_best_known_outcome(states: &mut [SearchNode]) {
    const DELETION_MASK: u64 = !((0b111_1111 << offsets::UNKNOWN_CHILD_COUNT)
        | (0b1_1111_1111 << offsets::BEST_KNOWN_OUTCOME));

    for state in states {
        match state.terminality() {
            Terminality::Nonterminal => {
                state.0 = (state.0 & DELETION_MASK)
                    | ((state.total_child_count() as u64) << offsets::UNKNOWN_CHILD_COUNT)
                    | (NEGATIVE_201_I9 << offsets::BEST_KNOWN_OUTCOME);
            }

            Terminality::Win => {
                state.0 = (state.0 & DELETION_MASK)
                    | (0 << offsets::UNKNOWN_CHILD_COUNT)
                    | (POSITIVE_201_I9 << offsets::BEST_KNOWN_OUTCOME);
            }

            Terminality::Loss => {
                state.0 = (state.0 & DELETION_MASK)
                    | (0 << offsets::UNKNOWN_CHILD_COUNT)
                    | (NEGATIVE_201_I9 << offsets::BEST_KNOWN_OUTCOME);
            }
        }
    }
}

fn add_terminal_nodes(states: &[SearchNode], stack: &mut Vec<SearchNode>) {
    for state in states {
        if state.is_terminal() {
            stack.push(*state);
        }
    }
}

impl SearchNode {
    fn total_child_count(self) -> u8 {
        let mut current_action = Action(0b001_0000);
        let mut count = 0;

        loop {
            let (child, next_action) = self.apply_action(current_action);
            if child.is_some() {
                count += 1;
            }

            if next_action.is_none() {
                return count;
            }

            current_action = next_action.unchecked_unwrap();
        }
    }

    fn unknown_child_count(self) -> u8 {
        ((self.0 >> offsets::UNKNOWN_CHILD_COUNT) & 0b111_1111) as u8
    }

    fn best_known_outcome(self) -> Outcome {
        Outcome(i16::from_zero_padded_i9(
            (self.0 >> offsets::BEST_KNOWN_OUTCOME) & 0b1_1111_1111,
        ))
    }

    fn record_child_outcome(self, child_outcome: Outcome) -> Self {
        let incumbent = self.best_known_outcome();
        let challenger = child_outcome.invert().delay_by_one();
        if challenger > incumbent {
            Self(
                self.0 & !(0b1_1111_1111 << offsets::BEST_KNOWN_OUTCOME)
                    | (challenger.0.into_zero_padded_i9_unchecked() << offsets::BEST_KNOWN_OUTCOME),
            )
        } else {
            self
        }
    }
}

impl SearchNode {
    fn write_parents(self, out: &mut Vec<SearchNode>) {
        out.clear();

        let board = self.into_builder().board();

        self.write_parents_with_dest_coords(out, board, 0b0000);
        self.write_parents_with_dest_coords(out, board, 0b0001);
        self.write_parents_with_dest_coords(out, board, 0b0010);

        self.write_parents_with_dest_coords(out, board, 0b0100);
        self.write_parents_with_dest_coords(out, board, 0b0101);
        self.write_parents_with_dest_coords(out, board, 0b0110);

        self.write_parents_with_dest_coords(out, board, 0b1000);
        self.write_parents_with_dest_coords(out, board, 0b1001);
        self.write_parents_with_dest_coords(out, board, 0b1010);

        self.write_parents_with_dest_coords(out, board, 0b1100);
        self.write_parents_with_dest_coords(out, board, 0b1101);
        self.write_parents_with_dest_coords(out, board, 0b1110);
    }

    #[inline(always)]
    fn write_parents_with_dest_coords(
        self,
        out: &mut Vec<SearchNode>,
        board: Board,
        dest_coords: u64,
    ) {
        let dest_offset = coords_to_board_offset(dest_coords);

        if board.is_square_nonpassive_at_board_offset(dest_offset) {
            return;
        }

        const ROW_MASK: u64 = 0b1100;
        const COLUMN_MASK: u64 = 0b0011;
        const MAX_ROW: u64 = 0b1100;
        const MAX_COLUMN: u64 = 0b0010;
        const ROW_DELTA: u64 = 0b0100;
        const COLUMN_DELTA: u64 = 0b0001;

        // start = dest.north()
        if (dest_coords & ROW_MASK) < MAX_ROW {
            let start_coords = dest_coords + ROW_DELTA;
            self.write_parents_with_dest_and_start_coords(out, board, dest_coords, start_coords)
        }

        // start = dest.northeast()
        if (dest_coords & ROW_MASK) < MAX_ROW && (dest_coords & COLUMN_MASK) < MAX_COLUMN {
            let start_coords = dest_coords + ROW_DELTA + COLUMN_DELTA;
            self.write_parents_with_dest_and_start_coords(out, board, dest_coords, start_coords)
        }

        // start = dest.east()
        if (dest_coords & COLUMN_MASK) < MAX_COLUMN {
            let start_coords = dest_coords + COLUMN_DELTA;
            self.write_parents_with_dest_and_start_coords(out, board, dest_coords, start_coords)
        }

        // start = dest.southeast()
        if (dest_coords & ROW_MASK) > 0 && (dest_coords & COLUMN_MASK) < MAX_COLUMN {
            let start_coords = dest_coords - ROW_DELTA + COLUMN_DELTA;
            self.write_parents_with_dest_and_start_coords(out, board, dest_coords, start_coords)
        }

        // start = dest.south()
        if (dest_coords & ROW_MASK) > 0 {
            let start_coords = dest_coords - ROW_DELTA;
            self.write_parents_with_dest_and_start_coords(out, board, dest_coords, start_coords)
        }

        // start = dest.southwest()
        if (dest_coords & ROW_MASK) > 0 && (dest_coords & COLUMN_MASK) > 0 {
            let start_coords = dest_coords - ROW_DELTA - COLUMN_DELTA;
            self.write_parents_with_dest_and_start_coords(out, board, dest_coords, start_coords)
        }

        // start = dest.west()
        if (dest_coords & COLUMN_MASK) > 0 {
            let start_coords = dest_coords - COLUMN_DELTA;
            self.write_parents_with_dest_and_start_coords(out, board, dest_coords, start_coords)
        }

        // start = dest.northwest()
        if (dest_coords & ROW_MASK) < MAX_ROW && (dest_coords & COLUMN_MASK) > 0 {
            let start_coords = dest_coords + ROW_DELTA - COLUMN_DELTA;
            self.write_parents_with_dest_and_start_coords(out, board, dest_coords, start_coords)
        }
    }

    #[inline(always)]
    fn write_parents_with_dest_and_start_coords(
        self,
        out: &mut Vec<SearchNode>,
        board: Board,
        dest_coords: u64,
        start_coords: u64,
    ) {
        todo!()
    }
}

impl Outcome {
    const fn invert(self) -> Self {
        Self(-self.0)
    }

    const fn delay_by_one(self) -> Self {
        Self(self.0 + -self.0.signum())
    }
}

/// `-200`` in 9-bit two's complement, left-padded with zeros
/// to fill the 64-bit integer.
const NEGATIVE_201_I9: u64 = 0b1_0011_0111;

/// `200` in 9-bit two's complement, left-padded with zeros
/// to fill the 64-bit integer.
const POSITIVE_201_I9: u64 = 0b0_1100_1001;
