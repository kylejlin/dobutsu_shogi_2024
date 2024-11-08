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

    let mut stack = vec![];
    add_terminal_nodes(states, &mut stack);

    while let Some(top) = stack.pop() {
        todo!()
    }
}

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
                    | (state.total_child_count() << offsets::UNKNOWN_CHILD_COUNT)
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
    fn total_child_count(self) -> u64 {
        todo!()
    }
}

/// `-200`` in 9-bit two's complement, left-padded with zeros
/// to fill the 64-bit integer.
const NEGATIVE_201_I9: u64 = 0b1_0011_0111;

/// `200` in 9-bit two's complement, left-padded with zeros
/// to fill the 64-bit integer.
const POSITIVE_201_I9: u64 = 0b0_1100_1001;
