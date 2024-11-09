#![warn(clippy::all)]
#![allow(clippy::unusual_byte_groupings)]
#![allow(clippy::type_complexity)]

#[cfg(test)]
mod tests;

pub mod backward_pass;
pub mod forward_pass;

pub use backward_pass::solve;
pub use forward_pass::reachable_states;

// A note about fileds with the comment "Must be non-zero":
//
// I know we _could_ use a `NonZeroU64` (or another respective `NonZero*` type),
// but that would clutter the code with a bunch of unwraps,
// which hurts readability and performance.

#[repr(i8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Terminality {
    Loss = -1,
    Nonterminal = 0,
    Win = 1,
}

/// The **least** significant 4 bits are used.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SearchNode(
    // Must be non-zero.
    pub u64,
);

/// This is like a `SearchNode`,
/// but with the `chick0 <= chick1` invariant
/// (and all similar invariants) removed.
/// In other words, `NodeBuilder` represents a
/// possibly "corrupted" forward node,
/// and `SearchNode` is the subset of `NodeBuilder`
/// representing "valid" forward nodes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct NodeBuilder(
    /// This must be non-zero.
    u64,
);

/// An optional node builder `o` represents None if and only if `o.0 == 0`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct OptionalNodeBuilder(u64);

#[derive(Clone, Copy, Debug)]
pub struct OptionalSearchNode(
    // This is zero if and only if
    // the option is `NONE`.
    u64,
);

/// The **least** significant 7 bits are used.
#[derive(Clone, Copy, Debug)]
struct Action(
    /// This must be non-zero.
    u8,
);

/// The **least** significant 7 bits are used.
#[derive(Clone, Copy, Debug)]
struct OptionalAction(
    /// This is zero if and only if
    /// the option is `NONE`.
    u8,
);

/// The **least** significant 48 bits are used.
#[derive(Clone, Copy, Debug)]
struct Board(u64);

/// This is a C-string-inspired vector of
/// up to 8 board coordinates.
/// The first coordinate is in the first (i.e., least significant) 4 bits,
/// the second coordinate is in the second 4 bits, and so on.
/// If a coordinate is `0b1111`, this is a "null terminator"
/// (analogous `'\0'` in C strings) which indicates the end of the vector.
///
/// The advantage of `CoordVec` over `CoordSet` is that
/// iteration is fast.
///
/// The disadvantage is that inclusion lookups are O(n).
#[derive(Clone, Copy, Debug)]
struct CoordVec(u64);

/// This is a bitset of up to 12 board coordinates.
///
/// The advantage of `CoordSet` over `CoordVec` is that
/// inclusion lookups are O(1).
///
/// The disadvantage is that iteration is slow.
#[derive(Clone, Copy, Debug)]
struct CoordSet(u16);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Piece(u8);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ActivePiece(Piece);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PassivePiece(Piece);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Nonlion(Piece);

#[derive(Clone, Copy, Debug)]
struct Offset(u8);

/// WARNING: This struct may possibly be `Coords(0b1111)`,
/// which represents the location of the hand.
/// Thus, it is important not to assume that the coordinates
/// are on the board.
#[derive(Clone, Copy, Debug)]
struct Coords(u8);

type ActionHandler = fn(SearchNode) -> (OptionalNodeBuilder, OptionalAction);

impl Terminality {
    const fn is_terminal(self) -> bool {
        (self as i8) != (Terminality::Nonterminal as i8)
    }
}

impl OptionalSearchNode {
    const NONE: Self = Self(0);

    const fn is_some(self) -> bool {
        self.0 != 0
    }

    const fn unchecked_unwrap(self) -> SearchNode {
        SearchNode(self.0)
    }
}

impl OptionalNodeBuilder {
    const NONE: Self = Self(0);

    const fn is_none(self) -> bool {
        self.0 == 0
    }

    const fn unchecked_unwrap(self) -> NodeBuilder {
        NodeBuilder(self.0)
    }
}

impl OptionalAction {
    const NONE: Self = Self(0);

    const fn is_none(self) -> bool {
        self.0 == 0
    }

    const fn unchecked_unwrap(self) -> Action {
        Action(self.0)
    }
}

impl SearchNode {
    pub const fn initial() -> Self {
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

        let next_action: u64 = 0b001_0000;

        Self(
            (chick0 << Offset::CHICK0.0)
                | (chick1 << Offset::CHICK1.0)
                | (elephant0 << Offset::ELEPHANT0.0)
                | (elephant1 << Offset::ELEPHANT1.0)
                | (giraffe0 << Offset::GIRAFFE0.0)
                | (giraffe1 << Offset::GIRAFFE1.0)
                | (active_lion << Offset::ACTIVE_LION.0)
                | (passive_lion << Offset::PASSIVE_LION.0)
                | (next_action << Offset::NEXT_ACTION.0),
        )
    }

    fn next_child(mut self) -> (Self, OptionalSearchNode) {
        loop {
            let raw = ((self.0 >> Offset::NEXT_ACTION.0) & 0b111_1111) as u8;
            if raw == 0 {
                return (self, OptionalSearchNode::NONE);
            }

            let (new_self, new_child) = self.explore(Action(raw));

            if new_child.is_some() {
                return (new_self, new_child);
            }

            self = new_self;
        }
    }

    fn explore(self, action: Action) -> (Self, OptionalSearchNode) {
        let (child, next_action) = self.apply_action(action);
        let new_self = self.set_next_action(next_action);
        (new_self, child)
    }

    fn apply_action(self, action: Action) -> (OptionalSearchNode, OptionalAction) {
        let (child_builder, next_action) = ACTION_HANDLERS[(action.0 - 16) as usize](self);

        let child = if child_builder.is_none() {
            OptionalSearchNode::NONE
        } else {
            child_builder
                .unchecked_unwrap()
                .invert_active_player()
                .init_next_action()
                .build()
                .into_optional()
        };

        (child, next_action)
    }

    const fn set_next_action(self, next_action: OptionalAction) -> Self {
        let raw = next_action.0 as u64;
        Self((self.0 & !(0b111_1111 << Offset::NEXT_ACTION.0)) | (raw << Offset::NEXT_ACTION.0))
    }

    const fn is_terminal(self) -> bool {
        self.terminality().is_terminal()
    }

    const fn terminality(self) -> Terminality {
        const ACTIVE_LION_COORDS_MASK: u64 = 0b1111 << Offset::ACTIVE_LION.0;
        if self.0 & ACTIVE_LION_COORDS_MASK == ACTIVE_LION_COORDS_MASK {
            return Terminality::Loss;
        }

        const ACTIVE_LION_TRY_MASK: u64 = 0b11 << Offset::ACTIVE_LION_ROW.0;
        if self.0 & ACTIVE_LION_TRY_MASK == ACTIVE_LION_TRY_MASK {
            return Terminality::Win;
        }

        Terminality::Nonterminal
    }

    const fn into_builder(self) -> NodeBuilder {
        NodeBuilder(self.0)
    }

    const fn into_optional(self) -> OptionalSearchNode {
        OptionalSearchNode(self.0)
    }
}

impl NodeBuilder {
    const fn invert_active_player(self) -> Self {
        const NONLION_ALLEGIANCE_INVERSION_MASK: u64 = (1 << Offset::CHICK0_ALLEGIANCE.0)
            | (1 << Offset::CHICK1_ALLEGIANCE.0)
            | (1 << Offset::ELEPHANT0_ALLEGIANCE.0)
            | (1 << Offset::ELEPHANT1_ALLEGIANCE.0)
            | (1 << Offset::GIRAFFE0_ALLEGIANCE.0)
            | (1 << Offset::GIRAFFE1_ALLEGIANCE.0);

        let chick0_inverted_coords = self.invert_coords_at_offset(Offset::CHICK0_COLUMN);
        let chick1_inverted_coords = self.invert_coords_at_offset(Offset::CHICK1_COLUMN);
        let elephant0_inverted_coords = self.invert_coords_at_offset(Offset::ELEPHANT0_COLUMN);
        let elephant1_inverted_coords = self.invert_coords_at_offset(Offset::ELEPHANT1_COLUMN);
        let giraffe0_inverted_coords = self.invert_coords_at_offset(Offset::GIRAFFE0_COLUMN);
        let giraffe1_inverted_coords = self.invert_coords_at_offset(Offset::GIRAFFE1_COLUMN);
        let active_lion_inverted_coords = self.invert_coords_at_offset(Offset::ACTIVE_LION_COLUMN);
        let passive_lion_inverted_coords =
            self.invert_coords_at_offset(Offset::PASSIVE_LION_COLUMN);

        const ALL_COORDS_MASK: u64 = (0b1111 << Offset::CHICK0_COLUMN.0)
            | (0b1111 << Offset::CHICK1_COLUMN.0)
            | (0b1111 << Offset::ELEPHANT0_COLUMN.0)
            | (0b1111 << Offset::ELEPHANT1_COLUMN.0)
            | (0b1111 << Offset::GIRAFFE0_COLUMN.0)
            | (0b1111 << Offset::GIRAFFE1_COLUMN.0)
            | (0b1111 << Offset::ACTIVE_LION_COLUMN.0)
            | (0b1111 << Offset::PASSIVE_LION_COLUMN.0);

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
                >> (Offset::ACTIVE_LION.0 - Offset::PASSIVE_LION.0))
            | (passive_lion_inverted_coords
                << (Offset::ACTIVE_LION.0 - Offset::PASSIVE_LION.0)),
        )
    }

    #[inline(always)]
    const fn invert_coords_at_offset(self, coords_offset: Offset) -> u64 {
        let coords_mask = 0b1111 << coords_offset.0;
        let r3c2 = 0b1110 << coords_offset.0;
        let coords = self.0 & coords_mask;
        if coords == coords_mask {
            return coords;
        }
        r3c2 - coords
    }

    /// If the this is terminal, then we set the next action to `None`.
    /// Otherwise, we set the next action `to Action(0b001_0000)`.
    const fn init_next_action(self) -> Self {
        if self.is_terminal() {
            return Self(
                (self.0 & !(0b111_1111 << Offset::NEXT_ACTION.0))
                    | ((OptionalAction::NONE.0 as u64) << Offset::NEXT_ACTION.0),
            );
        }

        const DEFAULT_FIRST_ACTION: Action = Action(0b001_0000);
        Self(
            (self.0 & !(0b111_1111 << Offset::NEXT_ACTION.0))
                | ((DEFAULT_FIRST_ACTION.0 as u64) << Offset::NEXT_ACTION.0),
        )
    }

    const fn is_terminal(self) -> bool {
        // At this point, the node is not guaranteed to be normalized.
        // However, this does not matter for terminality checks.
        // Thus, we can safely cast this to a `SearchNode`.
        SearchNode(self.0).is_terminal()
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
        const CHICK0_MASK: u64 = 0b11_1111 << Offset::CHICK0.0;
        const CHICK1_MASK: u64 = 0b11_1111 << Offset::CHICK1.0;
        const ELEPHANT0_MASK: u64 = 0b1_1111 << Offset::ELEPHANT0.0;
        const ELEPHANT1_MASK: u64 = 0b1_1111 << Offset::ELEPHANT1.0;
        const GIRAFFE0_MASK: u64 = 0b1_1111 << Offset::GIRAFFE0.0;
        const GIRAFFE1_MASK: u64 = 0b1_1111 << Offset::GIRAFFE1.0;

        let chick0 = self.0 & CHICK0_MASK;
        let chick1 = self.0 & CHICK1_MASK;
        let chick1_shifted = chick1 << (Offset::CHICK0.0 - Offset::CHICK1.0);
        let (chick0, chick1) = if chick0 <= chick1_shifted {
            (chick0, chick1)
        } else {
            (
                chick1_shifted,
                chick0 >> (Offset::CHICK0.0 - Offset::CHICK1.0),
            )
        };

        let elephant0 = self.0 & ELEPHANT0_MASK;
        let elephant1 = self.0 & ELEPHANT1_MASK;
        let elephant1_shifted = elephant1 << (Offset::ELEPHANT0.0 - Offset::ELEPHANT1.0);
        let (elephant0, elephant1) = if elephant0 <= elephant1_shifted {
            (elephant0, elephant1)
        } else {
            (
                elephant1_shifted,
                elephant0 >> (Offset::ELEPHANT0.0 - Offset::ELEPHANT1.0),
            )
        };

        let giraffe0 = self.0 & GIRAFFE0_MASK;
        let giraffe1 = self.0 & GIRAFFE1_MASK;
        let giraffe1_shifted = giraffe1 << (Offset::GIRAFFE0.0 - Offset::GIRAFFE1.0);
        let (giraffe0, giraffe1) = if giraffe0 <= giraffe1_shifted {
            (giraffe0, giraffe1)
        } else {
            (
                giraffe1_shifted,
                giraffe0 >> (Offset::GIRAFFE0.0 - Offset::GIRAFFE1.0),
            )
        };

        const NONLION_MASK: u64 = 0xFFFF_FFFF << Offset::GIRAFFE1.0;
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
        macro_rules! flip_column {
            ($column_offset:expr) => {{
                const COL_MASK: u64 = 0b11 << $column_offset.0;
                let col = self.0 & COL_MASK;

                // A piece is in hand if and only if the column is `0b11`.
                if col == COL_MASK {
                    col
                } else {
                    (0b10 << $column_offset.0) - col
                }
            }};
        }

        let chick0_col_flipped = flip_column!(Offset::CHICK0_COLUMN);
        let chick1_col_flipped = flip_column!(Offset::CHICK1_COLUMN);
        let elephant0_col_flipped = flip_column!(Offset::ELEPHANT0_COLUMN);
        let elephant1_col_flipped = flip_column!(Offset::ELEPHANT1_COLUMN);
        let giraffe0_col_flipped = flip_column!(Offset::GIRAFFE0_COLUMN);
        let giraffe1_col_flipped = flip_column!(Offset::GIRAFFE1_COLUMN);
        let active_lion_col_flipped = flip_column!(Offset::ACTIVE_LION_COLUMN);
        let passive_lion_col_flipped = flip_column!(Offset::PASSIVE_LION_COLUMN);

        const ALL_COLUMNS_MASK: u64 = (0b11 << Offset::CHICK0_COLUMN.0)
            | (0b11 << Offset::CHICK1_COLUMN.0)
            | (0b11 << Offset::ELEPHANT0_COLUMN.0)
            | (0b11 << Offset::ELEPHANT1_COLUMN.0)
            | (0b11 << Offset::GIRAFFE0_COLUMN.0)
            | (0b11 << Offset::GIRAFFE1_COLUMN.0)
            | (0b11 << Offset::ACTIVE_LION_COLUMN.0)
            | (0b11 << Offset::PASSIVE_LION_COLUMN.0);

        Self(
            (self.0 & !ALL_COLUMNS_MASK)
                | chick0_col_flipped
                | chick1_col_flipped
                | elephant0_col_flipped
                | elephant1_col_flipped
                | giraffe0_col_flipped
                | giraffe1_col_flipped
                | active_lion_col_flipped
                | passive_lion_col_flipped,
        )
    }
}

impl ActivePiece {
    #[inline(always)]
    const fn coords_mask(self) -> u64 {
        0b1111 << self.coords_offset().0
    }

    #[inline(always)]
    const fn coords_offset(self) -> Offset {
        match self {
            ActivePiece::LION => Offset::ACTIVE_LION_COLUMN,
            ActivePiece::CHICK0 => Offset::CHICK0_COLUMN,
            ActivePiece::CHICK1 => Offset::CHICK1_COLUMN,
            ActivePiece::ELEPHANT0 => Offset::ELEPHANT0_COLUMN,
            ActivePiece::ELEPHANT1 => Offset::ELEPHANT1_COLUMN,
            ActivePiece::GIRAFFE0 => Offset::GIRAFFE0_COLUMN,
            ActivePiece::GIRAFFE1 => Offset::GIRAFFE1_COLUMN,

            _ => Offset(0),
        }
    }
}

impl Nonlion {
    #[inline(always)]
    const fn coords_offset(self) -> Offset {
        match self {
            Nonlion::CHICK0 => Offset::CHICK0_COLUMN,
            Nonlion::CHICK1 => Offset::CHICK1_COLUMN,
            Nonlion::ELEPHANT0 => Offset::ELEPHANT0_COLUMN,
            Nonlion::ELEPHANT1 => Offset::ELEPHANT1_COLUMN,
            Nonlion::GIRAFFE0 => Offset::GIRAFFE0_COLUMN,
            Nonlion::GIRAFFE1 => Offset::GIRAFFE1_COLUMN,

            _ => Offset(0),
        }
    }
}

impl Coords {
    #[inline(always)]
    const fn board_offset(self: Coords) -> u8 {
        self.0 * 4
    }
}

impl CoordVec {
    const EMPTY: Self = Self(0b1111);
    const MAX_ELEMENTS: usize = 8;
    const TERMINATOR: Coords = Coords(0b1111);

    /// If the vector is already full, then this function
    /// behaves as the identity function.
    #[inline(always)]
    const fn push(self, coords: Coords) -> Self {
        let mut i = 0;
        while i < Self::MAX_ELEMENTS {
            let mask = 0b1111 << (i * 4);

            if self.0 & mask == mask {
                return Self(
                    (self.0 & !mask) | ((coords.0 as u64) << (i * 4)) | (0b1111 << ((i + 1) * 4)),
                );
            }

            i += 1;
        }

        self
    }

    #[inline(always)]
    const fn into_coord_set(self) -> CoordSet {
        let mut out = CoordSet(0);

        let mut i = 0;
        while i < Self::MAX_ELEMENTS {
            let coords = (self.0 >> (i * 4)) & 0b1111;

            if coords == Self::TERMINATOR.0 as u64 {
                break;
            }

            out.0 |= 1 << coords;

            i += 1;
        }

        return out;
    }
}

impl Iterator for CoordVec {
    type Item = Coords;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.0 & 0b1111;
        if next == Self::TERMINATOR.0.into() {
            return None;
        }

        self.0 >>= 4;

        Some(Coords(next as u8))
    }
}

macro_rules! action_handlers_for_piece {
    ($piece:ident) => {
        [
            action_handlers::$piece::r00_c00,
            action_handlers::$piece::r00_c01,
            action_handlers::$piece::r00_c10,
            action_handlers::handle_bad_action,
            action_handlers::$piece::r01_c00,
            action_handlers::$piece::r01_c01,
            action_handlers::$piece::r01_c10,
            action_handlers::handle_bad_action,
            action_handlers::$piece::r10_c00,
            action_handlers::$piece::r10_c01,
            action_handlers::$piece::r10_c10,
            action_handlers::handle_bad_action,
            action_handlers::$piece::r11_c00,
            action_handlers::$piece::r11_c01,
            action_handlers::$piece::r11_c10,
            action_handlers::handle_bad_action,
        ]
    };
}

macro_rules! concat_action_handlers {
    ($left:expr, $right:expr) => {{
        let left = $left;
        let right = $right;

        let mut arr: [ActionHandler; $left.len() + $right.len()] =
            [dummy_action_handler; $left.len() + $right.len()];

        let left_len = left.len();
        let mut i = 0;
        while i < left_len {
            arr[i] = left[i];
            i += 1;
        }

        let right_len = right.len();
        let mut i = 0;
        while i < right_len {
            arr[left.len() + i] = right[i];
            i += 1;
        }

        arr
    }};
}

const fn dummy_action_handler(_: SearchNode) -> (OptionalNodeBuilder, OptionalAction) {
    (OptionalNodeBuilder::NONE, OptionalAction::NONE)
}

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
const ACTION_HANDLERS: [ActionHandler; 7 * 16] = concat_action_handlers!(
    concat_action_handlers!(
        concat_action_handlers!(
            concat_action_handlers!(
                concat_action_handlers!(
                    concat_action_handlers!(
                        action_handlers_for_piece!(active_lion),
                        action_handlers_for_piece!(chick0)
                    ),
                    action_handlers_for_piece!(chick1)
                ),
                action_handlers_for_piece!(elephant0)
            ),
            action_handlers_for_piece!(elephant1)
        ),
        action_handlers_for_piece!(giraffe0)
    ),
    action_handlers_for_piece!(giraffe1)
);

macro_rules! define_action_handler {
    ($piece:literal, $name:ident, $dest_coords:literal) => {
        pub const fn $name(state: SearchNode) -> (OptionalNodeBuilder, OptionalAction) {
            state
                .into_builder()
                .handle_action(Action(($piece << 4) | $dest_coords))
        }
    };
}

macro_rules! define_all_action_handlers_for_piece {
    ($name:ident, $piece:literal) => {
        pub mod $name {
            use super::*;

            define_action_handler!($piece, r00_c00, 0b0000);
            define_action_handler!($piece, r00_c01, 0b0001);
            define_action_handler!($piece, r00_c10, 0b0010);

            define_action_handler!($piece, r01_c00, 0b0100);
            define_action_handler!($piece, r01_c01, 0b0101);
            define_action_handler!($piece, r01_c10, 0b0110);

            define_action_handler!($piece, r10_c00, 0b1000);
            define_action_handler!($piece, r10_c01, 0b1001);
            define_action_handler!($piece, r10_c10, 0b1010);

            define_action_handler!($piece, r11_c00, 0b1100);
            define_action_handler!($piece, r11_c01, 0b1101);
            define_action_handler!($piece, r11_c10, 0b1110);
        }
    };
}
mod action_handlers {
    use super::*;

    define_all_action_handlers_for_piece!(active_lion, 0b001);
    define_all_action_handlers_for_piece!(chick0, 0b010);
    define_all_action_handlers_for_piece!(chick1, 0b011);
    define_all_action_handlers_for_piece!(elephant0, 0b100);
    define_all_action_handlers_for_piece!(elephant1, 0b101);
    define_all_action_handlers_for_piece!(giraffe0, 0b110);
    define_all_action_handlers_for_piece!(giraffe1, 0b111);

    pub fn handle_bad_action(_: SearchNode) -> (OptionalNodeBuilder, OptionalAction) {
        panic!("Illegal action");
    }
}

impl NodeBuilder {
    #[inline(always)]
    const fn handle_action(self, action: Action) -> (OptionalNodeBuilder, OptionalAction) {
        if self.is_actor_passive(action.actor()) {
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

        self.handle_move_assuming_actor_is_active_and_in_range_of_dest_square(action)
    }

    #[inline(always)]
    const fn is_actor_passive(self, actor: ActivePiece) -> bool {
        let allegiance_bit_offset = match actor {
            // The active lion can never be passive.
            ActivePiece::LION => return false,

            ActivePiece::CHICK0 => Offset::CHICK0_ALLEGIANCE,
            ActivePiece::CHICK1 => Offset::CHICK1_ALLEGIANCE,
            ActivePiece::ELEPHANT0 => Offset::ELEPHANT0_ALLEGIANCE,
            ActivePiece::ELEPHANT1 => Offset::ELEPHANT1_ALLEGIANCE,
            ActivePiece::GIRAFFE0 => Offset::GIRAFFE0_ALLEGIANCE,
            ActivePiece::GIRAFFE1 => Offset::GIRAFFE1_ALLEGIANCE,

            _ => return false,
        };

        self.0 & (1 << allegiance_bit_offset.0) != 0
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
            ($coords:expr) => {
                if action.dest_coords().0 < $coords.0
                    && board.is_square_empty_at_board_offset($coords.board_offset())
                {
                    return action.set_dest_square($coords).into_optional();
                }
            };
        }

        check_square!(Coords::R0C1);
        check_square!(Coords::R0C2);

        check_square!(Coords::R1C0);
        check_square!(Coords::R1C1);
        check_square!(Coords::R1C2);

        check_square!(Coords::R2C0);
        check_square!(Coords::R2C1);
        check_square!(Coords::R2C2);

        check_square!(Coords::R3C0);
        check_square!(Coords::R3C1);
        check_square!(Coords::R3C2);

        action.next_piece_action()
    }

    #[inline(always)]
    const fn is_actor_out_of_range_of_dest_square(self, action: Action) -> bool {
        let actor_coords = self.actor_coords(action.actor());

        let legal_squares = action.legal_starting_squares();

        // Only pieces on the board can be in range.
        // Therefore, if the actor is in hand, it is out of range.
        // However, we don't have to explicitly check this case,
        // because if the actor is in hand, then `actor_coords == 15`,
        // and bit 15 of `legal_squares` is guaranteed to be 0.
        legal_squares[self.is_actor_promoted(action) as usize].0 & (1 << actor_coords.0) == 0
    }

    #[inline(always)]
    const fn actor_coords(self, actor: ActivePiece) -> Coords {
        Coords(((self.0 >> actor.coords_offset().0) as u8) & 0b1111)
    }

    #[inline(always)]
    const fn is_actor_promoted(self, action: Action) -> bool {
        let offset = match action.actor() {
            ActivePiece::CHICK0 => Offset::CHICK0_PROMOTION,
            ActivePiece::CHICK1 => Offset::CHICK1_PROMOTION,

            _ => return false,
        };
        self.0 & (1 << offset.0) != 0
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
            ($coords:expr) => {{
                let candidate = action.set_dest_square($coords);

                if action.dest_coords().0 < $coords.0
                    && board.is_dest_square_nonactive(candidate)
                    && self.is_actor_in_range_of_dest_square(candidate)
                {
                    return candidate.into_optional();
                }
            }};
        }

        check_square!(Coords::R0C1);
        check_square!(Coords::R0C2);

        check_square!(Coords::R1C0);
        check_square!(Coords::R1C1);
        check_square!(Coords::R1C2);

        check_square!(Coords::R2C0);
        check_square!(Coords::R2C1);
        check_square!(Coords::R2C2);

        check_square!(Coords::R3C0);
        check_square!(Coords::R3C1);
        check_square!(Coords::R3C2);

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
            Offset::PASSIVE_LION_COLUMN,
            Offset::CHICK0_COLUMN,
            Offset::CHICK1_COLUMN,
            Offset::ELEPHANT0_COLUMN,
            Offset::ELEPHANT1_COLUMN,
            Offset::GIRAFFE0_COLUMN,
            Offset::GIRAFFE1_COLUMN,
        ][occupant_lookup_index];

        let is_occupant_nonlion = occupant != 0b001;
        // If the occupant is a non-lion, we need to set the allegiance bit to 0.
        // The allegiance bit is 4 bits left of the column offset.
        let allegiance_mask = !((is_occupant_nonlion as u64) << (occupant_coords_offset.0 + 4));

        let is_occupant_chick = occupant & !1 == 0b010;
        // If the occupant is a chick, we need to set the promotion bit to 0.
        // The promotion bit is 1 bit right of the column offset.
        let demotion_mask = !((is_occupant_chick as u64) << (occupant_coords_offset.0 - 1));

        Self((self.0 | (0b1111 << occupant_coords_offset.0)) & allegiance_mask & demotion_mask)
            .into_optional()
    }

    #[inline(always)]
    const fn promote_actor_if_needed(self, action: Action) -> NodeBuilder {
        if action.is_actor_chick0() {
            let coords = self.0 & action.coords_mask();
            let promotion_bit = (((coords != action.coords_mask())
                & (coords >= (0b1100 << Offset::CHICK0_COLUMN.0)))
                as u64)
                << Offset::CHICK0_PROMOTION.0;
            return Self(self.0 | promotion_bit);
        }

        if action.is_actor_chick1() {
            let coords = self.0 & action.coords_mask();
            let promotion_bit = (((coords != action.coords_mask())
                & (coords >= (0b1100 << Offset::CHICK1_COLUMN.0)))
                as u64)
                << Offset::CHICK1_PROMOTION.0;
            return Self(self.0 | promotion_bit);
        }

        self
    }

    const fn board(self) -> Board {
        let mut board: u64 = 0;

        // For each piece, we first check whether it's in the hand.
        // If so, we skip it.
        // Otherwise, we calculate the board offset and add the piece to the board.

        macro_rules! add_nonlion {
            ($piece:expr) => {{
                let board_offset = self.nonlion_coords($piece).board_offset();
                let allegiance_in_bit3 = self.nonlion_allegiance_in_bit_3($piece);
                board |= (allegiance_in_bit3 | ($piece.0 .0 as u64)) << board_offset;
            }};
        }

        add_nonlion!(Nonlion::CHICK0);
        add_nonlion!(Nonlion::CHICK1);
        add_nonlion!(Nonlion::ELEPHANT0);
        add_nonlion!(Nonlion::ELEPHANT1);
        add_nonlion!(Nonlion::GIRAFFE0);
        add_nonlion!(Nonlion::GIRAFFE1);

        {
            let board_offset = self.active_lion_coords().board_offset();
            const ALLEGIANCE_IN_BIT3: u64 = 0 << 3;
            board |= (ALLEGIANCE_IN_BIT3 | (Piece::AMBIGUOUS_LION.0 as u64)) << board_offset;
        }

        {
            let board_offset = self.passive_lion_coords().board_offset();
            const ALLEGIANCE_IN_BIT3: u64 = 1 << 3;
            board |= (ALLEGIANCE_IN_BIT3 | (Piece::AMBIGUOUS_LION.0 as u64)) << board_offset;
        }

        Board(board)
    }

    #[inline(always)]
    const fn nonlion_coords(self, piece: Nonlion) -> Coords {
        Coords(((self.0 >> piece.coords_offset().0) & 0b1111) as u8)
    }

    #[inline(always)]
    const fn nonlion_allegiance_in_bit_3(self, piece: Nonlion) -> u64 {
        let allegiance_offset = match piece {
            Nonlion::CHICK0 => Offset::CHICK0_ALLEGIANCE,
            Nonlion::CHICK1 => Offset::CHICK1_ALLEGIANCE,
            Nonlion::ELEPHANT0 => Offset::ELEPHANT0_ALLEGIANCE,
            Nonlion::ELEPHANT1 => Offset::ELEPHANT1_ALLEGIANCE,
            Nonlion::GIRAFFE0 => Offset::GIRAFFE0_ALLEGIANCE,
            Nonlion::GIRAFFE1 => Offset::GIRAFFE1_ALLEGIANCE,

            _ => return 0,
        };
        (self.0 >> (allegiance_offset.0 - 3)) & (1 << 3)
    }

    #[inline(always)]
    const fn active_lion_coords(self) -> Coords {
        Coords(((self.0 >> Offset::ACTIVE_LION_COLUMN.0) & 0b1111) as u8)
    }

    #[inline(always)]
    const fn passive_lion_coords(self) -> Coords {
        Coords(((self.0 >> Offset::PASSIVE_LION_COLUMN.0) & 0b1111) as u8)
    }

    #[inline(always)]
    const fn into_optional(self) -> OptionalNodeBuilder {
        OptionalNodeBuilder(self.0)
    }
}

impl Board {
    #[inline(always)]
    const fn is_dest_square_empty(self, action: Action) -> bool {
        self.is_square_empty_at_board_offset(action.dest_square_board_offset())
    }

    #[inline(always)]
    const fn is_square_empty_at_board_offset(self, board_offset: u8) -> bool {
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
    const fn actor(self) -> ActivePiece {
        ActivePiece(Piece(self.0 >> 4))
    }

    #[inline(always)]
    const fn next_species_action(self) -> OptionalAction {
        OptionalAction(match self.actor() {
            ActivePiece::LION => 0b010_0000,

            ActivePiece::CHICK0 => 0b100_0000,
            ActivePiece::CHICK1 => 0b100_0000,

            ActivePiece::ELEPHANT0 => 0b110_0000,
            ActivePiece::ELEPHANT1 => 0b110_0000,

            ActivePiece::GIRAFFE0 => 0,
            ActivePiece::GIRAFFE1 => 0,

            _ => 0,
        })
    }

    #[inline(always)]
    const fn next_piece_action(self) -> OptionalAction {
        OptionalAction(match self.actor() {
            ActivePiece::LION => 0b010_0000,
            ActivePiece::CHICK0 => 0b011_0000,
            ActivePiece::CHICK1 => 0b100_0000,
            ActivePiece::ELEPHANT0 => 0b101_0000,
            ActivePiece::ELEPHANT1 => 0b110_0000,
            ActivePiece::GIRAFFE0 => 0b111_0000,
            ActivePiece::GIRAFFE1 => 0,

            _ => 0,
        })
    }

    #[inline(always)]
    const fn coords_mask(self) -> u64 {
        self.actor().coords_mask()
    }

    #[inline(always)]
    const fn dest_square_coords_shifted_by_actor_coords_offset(self) -> u64 {
        (self.dest_coords().0 as u64) << self.actor().coords_offset().0
    }

    #[inline(always)]
    const fn dest_square_board_offset(self) -> u8 {
        self.dest_coords().board_offset()
    }

    #[inline(always)]
    const fn dest_coords(self) -> Coords {
        Coords((self.0 & 0b1111) as u8)
    }

    #[inline(always)]
    const fn is_actor_chick0(self) -> bool {
        self.actor().0 .0 == ActivePiece::CHICK0.0 .0
    }

    #[inline(always)]
    const fn is_actor_chick1(self) -> bool {
        self.actor().0 .0 == ActivePiece::CHICK1.0 .0
    }

    #[inline(always)]
    const fn set_dest_square(self, coords: Coords) -> Action {
        Self((self.0 & !0b1111) | coords.0)
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
    const fn legal_starting_squares(self) -> [CoordSet; 2] {
        let [nonpromoted, promoted] = self.actor().legal_starting_squares(self.dest_coords());
        [nonpromoted.into_coord_set(), promoted.into_coord_set()]
    }
}

impl ActivePiece {
    /// The set of legal starting squares depends on whether the
    /// actor is promoted.
    /// We cannot determine this from the actor and the destination coordinates alone.
    ///
    /// So, we return an array of two coordinate vectors.
    /// The first set is for the non-promoted actor,
    /// and the second set is for the promoted actor.
    ///
    /// It is the consumer's responsibility to select the correct
    /// vector to use.
    #[inline(always)]
    const fn legal_starting_squares(self, dest: Coords) -> [CoordVec; 2] {
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
            const fn connects(self, from: Coords, to: Coords) -> bool {
                let from_column = (from.0 & 0b11) as i8;
                let from_row = (from.0 >> 2) as i8;
                let to_column = (to.0 & 0b11) as i8;
                let to_row = (to.0 >> 2) as i8;

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

        let [nonpromoted_dirset, promoted_dirset] = match self {
            ActivePiece::LION => [LION, EMPTY],
            ActivePiece::CHICK0 => [CHICK, HEN],
            ActivePiece::CHICK1 => [CHICK, HEN],
            ActivePiece::ELEPHANT0 => [ELEPHANT, EMPTY],
            ActivePiece::ELEPHANT1 => [ELEPHANT, EMPTY],
            ActivePiece::GIRAFFE0 => [GIRAFFE, EMPTY],
            ActivePiece::GIRAFFE1 => [GIRAFFE, EMPTY],

            _ => [EMPTY, EMPTY],
        };

        let nonpromoted_squares = {
            let mut out = CoordVec::EMPTY;

            macro_rules! check_start_square {
                ($start_square:expr) => {
                    if nonpromoted_dirset.connects($start_square, dest) {
                        out = out.push($start_square);
                    }
                };
            }

            check_start_square!(Coords::R0C0);
            check_start_square!(Coords::R0C1);
            check_start_square!(Coords::R0C2);

            check_start_square!(Coords::R1C0);
            check_start_square!(Coords::R1C1);
            check_start_square!(Coords::R1C2);

            check_start_square!(Coords::R2C0);
            check_start_square!(Coords::R2C1);
            check_start_square!(Coords::R2C2);

            check_start_square!(Coords::R3C0);
            check_start_square!(Coords::R3C1);
            check_start_square!(Coords::R3C2);

            out
        };
        let promoted_squares = {
            let mut out = CoordVec::EMPTY;

            macro_rules! check_start_square {
                ($start_square:expr) => {
                    if promoted_dirset.connects($start_square, dest) {
                        out = out.push($start_square);
                    }
                };
            }

            check_start_square!(Coords::R0C0);
            check_start_square!(Coords::R0C1);
            check_start_square!(Coords::R0C2);

            check_start_square!(Coords::R1C0);
            check_start_square!(Coords::R1C1);
            check_start_square!(Coords::R1C2);

            check_start_square!(Coords::R2C0);
            check_start_square!(Coords::R2C1);
            check_start_square!(Coords::R2C2);

            check_start_square!(Coords::R3C0);
            check_start_square!(Coords::R3C1);
            check_start_square!(Coords::R3C2);

            out
        };

        [nonpromoted_squares, promoted_squares]
    }
}
impl Offset {
    const BEST_KNOWN_OUTCOME: Self = Self(0);
    const NEXT_ACTION: Self = Self(Self::BEST_KNOWN_OUTCOME.0 + 9);
    const UNKNOWN_CHILD_COUNT: Self = Self::NEXT_ACTION;
    const PASSIVE_LION: Self = Self(Self::NEXT_ACTION.0 + 7);
    const ACTIVE_LION: Self = Self(Self::PASSIVE_LION.0 + 4);
    const GIRAFFE1: Self = Self(Self::ACTIVE_LION.0 + 4);
    const GIRAFFE0: Self = Self(Self::GIRAFFE1.0 + 5);
    const ELEPHANT1: Self = Self(Self::GIRAFFE0.0 + 5);
    const ELEPHANT0: Self = Self(Self::ELEPHANT1.0 + 5);
    const CHICK1: Self = Self(Self::ELEPHANT0.0 + 5);
    const CHICK0: Self = Self(Self::CHICK1.0 + 6);

    const CHICK0_PROMOTION: Self = Self(Self::CHICK0.0);
    const CHICK0_COLUMN: Self = Self(Self::CHICK0_PROMOTION.0 + 1);
    const CHICK0_ROW: Self = Self(Self::CHICK0_COLUMN.0 + 2);
    const CHICK0_ALLEGIANCE: Self = Self(Self::CHICK0_ROW.0 + 2);

    const CHICK1_PROMOTION: Self = Self(Self::CHICK1.0);
    const CHICK1_COLUMN: Self = Self(Self::CHICK1_PROMOTION.0 + 1);
    const CHICK1_ROW: Self = Self(Self::CHICK1_COLUMN.0 + 2);
    const CHICK1_ALLEGIANCE: Self = Self(Self::CHICK1_ROW.0 + 2);

    const ELEPHANT0_COLUMN: Self = Self(Self::ELEPHANT0.0);
    const ELEPHANT0_ROW: Self = Self(Self::ELEPHANT0_COLUMN.0 + 2);
    const ELEPHANT0_ALLEGIANCE: Self = Self(Self::ELEPHANT0_ROW.0 + 2);

    const ELEPHANT1_COLUMN: Self = Self(Self::ELEPHANT1.0);
    const ELEPHANT1_ROW: Self = Self(Self::ELEPHANT1_COLUMN.0 + 2);
    const ELEPHANT1_ALLEGIANCE: Self = Self(Self::ELEPHANT1_ROW.0 + 2);

    const GIRAFFE0_COLUMN: Self = Self(Self::GIRAFFE0.0);
    const GIRAFFE0_ROW: Self = Self(Self::GIRAFFE0_COLUMN.0 + 2);
    const GIRAFFE0_ALLEGIANCE: Self = Self(Self::GIRAFFE0_ROW.0 + 2);

    const GIRAFFE1_COLUMN: Self = Self(Self::GIRAFFE1.0);
    const GIRAFFE1_ROW: Self = Self(Self::GIRAFFE1_COLUMN.0 + 2);
    const GIRAFFE1_ALLEGIANCE: Self = Self(Self::GIRAFFE1_ROW.0 + 2);

    const ACTIVE_LION_COLUMN: Self = Self(Self::ACTIVE_LION.0);
    const ACTIVE_LION_ROW: Self = Self(Self::ACTIVE_LION_COLUMN.0 + 2);

    const PASSIVE_LION_COLUMN: Self = Self(Self::PASSIVE_LION.0);
}

impl Piece {
    const AMBIGUOUS_LION: Self = Self(0b001);

    const CHICK0: Self = Self(0b010);
    const CHICK1: Self = Self(0b011);
    const ELEPHANT0: Self = Self(0b100);
    const ELEPHANT1: Self = Self(0b101);
    const GIRAFFE0: Self = Self(0b110);
    const GIRAFFE1: Self = Self(0b111);
}

impl ActivePiece {
    const LION: Self = Self(Piece::AMBIGUOUS_LION);
    const CHICK0: Self = Self(Piece::CHICK0);
    const CHICK1: Self = Self(Piece::CHICK1);
    const ELEPHANT0: Self = Self(Piece::ELEPHANT0);
    const ELEPHANT1: Self = Self(Piece::ELEPHANT1);
    const GIRAFFE0: Self = Self(Piece::GIRAFFE0);
    const GIRAFFE1: Self = Self(Piece::GIRAFFE1);
}

impl PassivePiece {
    const LION: Self = Self(Piece::AMBIGUOUS_LION);
    const CHICK0: Self = Self(Piece::CHICK0);
    const CHICK1: Self = Self(Piece::CHICK1);
    const ELEPHANT0: Self = Self(Piece::ELEPHANT0);
    const ELEPHANT1: Self = Self(Piece::ELEPHANT1);
    const GIRAFFE0: Self = Self(Piece::GIRAFFE0);
    const GIRAFFE1: Self = Self(Piece::GIRAFFE1);
}

impl Nonlion {
    const CHICK0: Self = Self(Piece::CHICK0);
    const CHICK1: Self = Self(Piece::CHICK1);
    const ELEPHANT0: Self = Self(Piece::ELEPHANT0);
    const ELEPHANT1: Self = Self(Piece::ELEPHANT1);
    const GIRAFFE0: Self = Self(Piece::GIRAFFE0);
    const GIRAFFE1: Self = Self(Piece::GIRAFFE1);
}

impl Coords {
    const R0C0: Self = Self(0b0000);
    const R0C1: Self = Self(0b0001);
    const R0C2: Self = Self(0b0010);

    const R1C0: Self = Self(0b0100);
    const R1C1: Self = Self(0b0101);
    const R1C2: Self = Self(0b0110);

    const R2C0: Self = Self(0b1000);
    const R2C1: Self = Self(0b1001);
    const R2C2: Self = Self(0b1010);

    const R3C0: Self = Self(0b1100);
    const R3C1: Self = Self(0b1101);
    const R3C2: Self = Self(0b1110);

    const HAND: Self = Self(0b1111);
}
