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
