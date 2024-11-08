#![warn(clippy::all)]
#![allow(clippy::unusual_byte_groupings)]
#![allow(clippy::type_complexity)]

// A note about fileds with the comment "This must be non-zero":
// I know we _could_ use a `NonZeroU64` (or another respective `NonZero*` type),
// but that would clutter the code with a bunch of unwraps,
// which hurts readability and performance.

pub mod backward_pass;
pub mod forward_pass;

pub use backward_pass::solve;
pub use forward_pass::reachable_states;

/// The **least** significant 4 bits are used.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SearchNode(
    // Must be non-zero.
    pub u64,
);

#[repr(i8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Terminality {
    Loss = -1,
    Nonterminal = 0,
    Win = 1,
}

impl Terminality {
    const fn is_terminal(self) -> bool {
        (self as i8) != (Terminality::Nonterminal as i8)
    }
}

impl SearchNode {
    const fn is_terminal(self) -> bool {
        self.terminality().is_terminal()
    }

    const fn terminality(self) -> Terminality {
        const ACTIVE_LION_COORDS_MASK: u64 = 0b1111 << offsets::ACTIVE_LION;
        if self.0 & ACTIVE_LION_COORDS_MASK == ACTIVE_LION_COORDS_MASK {
            return Terminality::Loss;
        }

        const ACTIVE_LION_TRY_MASK: u64 = 0b11 << offsets::ACTIVE_LION_ROW;
        if self.0 & ACTIVE_LION_TRY_MASK == ACTIVE_LION_TRY_MASK {
            return Terminality::Win;
        }

        Terminality::Nonterminal
    }
}

/// All offsets are given relative to the right (i.e., least significant) bit.
mod offsets {
    pub const BEST_KNOWN_OUTCOME: u64 = 0;
    pub const NEXT_ACTION: u64 = BEST_KNOWN_OUTCOME + 9;
    pub const UNKNOWN_CHILD_COUNT: u64 = NEXT_ACTION;
    pub const PASSIVE_LION: u64 = NEXT_ACTION + 7;
    pub const ACTIVE_LION: u64 = PASSIVE_LION + 4;
    pub const GIRAFFE1: u64 = ACTIVE_LION + 4;
    pub const GIRAFFE0: u64 = GIRAFFE1 + 5;
    pub const ELEPHANT1: u64 = GIRAFFE0 + 5;
    pub const ELEPHANT0: u64 = ELEPHANT1 + 5;
    pub const CHICK1: u64 = ELEPHANT0 + 5;
    pub const CHICK0: u64 = CHICK1 + 6;

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
}
