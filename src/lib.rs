#![warn(clippy::all)]
#![allow(clippy::unusual_byte_groupings)]
#![allow(clippy::type_complexity)]

#[cfg(test)]
mod tests;

pub mod backward_pass;
pub mod best_child_map;
pub mod forward_pass;
pub mod pretty;
pub mod prune;
pub mod state_map;
pub mod state_set;

pub use backward_pass::solve;
pub use forward_pass::reachable_states;
pub use prune::prune_assuming_one_player_plays_optimally;
pub use state_map::StateMap;
pub use state_set::StateSet;

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

///  - `0` represents a draw.
///
///  - A positive number `n` represents a win for the active player
///    in `201 - n` plies.
///
///  - A negative number `-n` represents a win for the passive player
///    in `201 + n` plies.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Outcome(pub i16);

/// The **least** significant 56 bits are used.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

/// The **least** significant 48 bits are used.
#[derive(Clone, Copy, Debug)]
struct Board(u64);

/// This is a C-string-inspired vector of
/// up to 13 board coordinates.
/// The first (i.e., least significant) 4 bits hold the length.
/// The next 4 bits hold the first coordinate.
/// The next 4 bits hold the second coordinate, and so on.
#[derive(Clone, Copy, Debug)]
struct CoordVec(u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Piece(u8);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Actor(Piece);

struct PassiveLion;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Nonlion(Piece);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Chick(Piece);

#[derive(Clone, Copy, Debug)]
struct Offset(u8);

/// WARNING: This struct may possibly be `Coords(0b1111)`,
/// which represents the location of the hand.
/// Thus, it is important not to assume that the coordinates
/// are on the board.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Coords(u8);

#[derive(Clone, Copy, Debug)]
struct ChildCalculator {
    node: SearchNode,
    board: Board,
    empty_squares: CoordVec,
}

#[derive(Clone, Copy, Debug)]
struct ParentCalculator {
    inverted_node: NodeBuilder,
    inverted_board: Board,
}

#[derive(Clone, Copy, Debug)]
struct ShouldDemoteActorInParent(bool);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Player {
    Sente,
    Gote,
}

// TODO: Delete after we are done debugging.
#[derive(Debug, Clone, Copy, Default)]
pub struct Progress {
    pub already_solved_parent_visits: usize,
    pub unsolved_parent_visits: usize,
    pub unreachable_parent_visits: usize,

    pub winning_parent_conclusions: usize,
    pub uncertain_parent_conclusions: usize,
    pub losing_parent_conclusions: usize,

    pub queue_pushes: usize,
}

impl Terminality {
    const fn is_terminal(self) -> bool {
        (self as i8) != (Terminality::Nonterminal as i8)
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

impl std::ops::Not for Player {
    type Output = Self;

    fn not(self) -> Self {
        match self {
            Player::Sente => Player::Gote,
            Player::Gote => Player::Sente,
        }
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

        Self(
            (chick0 << Offset::CHICK0.0)
                | (chick1 << Offset::CHICK1.0)
                | (elephant0 << Offset::ELEPHANT0.0)
                | (elephant1 << Offset::ELEPHANT1.0)
                | (giraffe0 << Offset::GIRAFFE0.0)
                | (giraffe1 << Offset::GIRAFFE1.0)
                | (active_lion << Offset::ACTIVE_LION.0)
                | (passive_lion << Offset::PASSIVE_LION.0),
        )
    }

    const fn is_terminal(self) -> bool {
        self.into_builder().is_terminal()
    }

    const fn terminality(self) -> Terminality {
        self.into_builder().terminality()
    }

    const fn into_builder(self) -> NodeBuilder {
        NodeBuilder(self.0)
    }

    pub fn children(self) -> Vec<SearchNode> {
        let mut out = vec![];
        self.visit_children(|child| out.push(child));
        out
    }

    pub fn parents(self) -> Vec<SearchNode> {
        let mut out = vec![];
        self.visit_parents(|parent| out.push(parent));
        out
    }

    /// This returns the state of the node,
    /// it its original (i.e., nonshifted) position.
    pub const fn state(self) -> u64 {
        const STATE_MASK: u64 = 0xFF_FFFF_FFFF << Offset::PASSIVE_LION.0;
        self.0 & STATE_MASK
    }

    pub const fn required_child_report_count(self) -> u8 {
        ((self.0 >> Offset::REQUIRED_CHILD_REPORT_COUNT.0) & 0b111_1111) as u8
    }

    pub fn best_known_outcome(self) -> Outcome {
        Outcome(i16::from_zero_padded_i9(
            (self.0 >> Offset::BEST_KNOWN_OUTCOME.0) & 0b1_1111_1111,
        ))
    }

    pub fn best_outcome(self) -> Option<Outcome> {
        if self.required_child_report_count() > 0 {
            return None;
        }

        Some(Outcome(i16::from_zero_padded_i9(
            (self.0 >> Offset::BEST_KNOWN_OUTCOME.0) & 0b1_1111_1111,
        )))
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

    /// Ensures that `chick0 <= chick1`, `elephant0 <= elephant1`, and `giraffe0 <= giraffe1`.
    const fn build(self) -> SearchNode {
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
}

impl Nonlion {
    #[inline(always)]
    const fn coords_mask(self) -> u64 {
        0b1111 << self.coords_offset().0
    }

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

    #[inline(always)]
    const fn is_in_hand(self, node: NodeBuilder) -> bool {
        node.0 & self.coords_mask() == self.coords_mask()
    }

    #[must_use]
    #[inline(always)]
    const fn set_coords(self, node: NodeBuilder, coords: Coords) -> NodeBuilder {
        let coords_offset = self.coords_offset().0;
        NodeBuilder((node.0 & !(0b1111 << coords_offset)) | ((coords.0 as u64) << coords_offset))
    }

    #[inline(always)]
    const fn allegiance_mask(self) -> u64 {
        let allegiance_bit_offset = match self {
            Nonlion::CHICK0 => Offset::CHICK0_ALLEGIANCE,
            Nonlion::CHICK1 => Offset::CHICK1_ALLEGIANCE,
            Nonlion::ELEPHANT0 => Offset::ELEPHANT0_ALLEGIANCE,
            Nonlion::ELEPHANT1 => Offset::ELEPHANT1_ALLEGIANCE,
            Nonlion::GIRAFFE0 => Offset::GIRAFFE0_ALLEGIANCE,
            Nonlion::GIRAFFE1 => Offset::GIRAFFE1_ALLEGIANCE,

            _ => return 0,
        };
        1 << allegiance_bit_offset.0
    }

    #[must_use]
    #[inline(always)]
    const fn make_passive(self, node: NodeBuilder) -> NodeBuilder {
        NodeBuilder(node.0 | self.allegiance_mask())
    }

    #[inline(always)]
    const fn is_active(self, node: NodeBuilder) -> bool {
        let allegiance_bit_offset = match self {
            Nonlion::CHICK0 => Offset::CHICK0_ALLEGIANCE,
            Nonlion::CHICK1 => Offset::CHICK1_ALLEGIANCE,
            Nonlion::ELEPHANT0 => Offset::ELEPHANT0_ALLEGIANCE,
            Nonlion::ELEPHANT1 => Offset::ELEPHANT1_ALLEGIANCE,
            Nonlion::GIRAFFE0 => Offset::GIRAFFE0_ALLEGIANCE,
            Nonlion::GIRAFFE1 => Offset::GIRAFFE1_ALLEGIANCE,

            _ => return false,
        };
        node.0 & (1 << allegiance_bit_offset.0) == 0
    }

    #[inline(always)]
    const fn is_bird(self) -> bool {
        self.0.is_bird()
    }

    #[must_use]
    #[inline(always)]
    const fn promote(self, node: NodeBuilder) -> NodeBuilder {
        let promotion_status_offset = match self {
            Nonlion::CHICK0 => Offset::CHICK0_PROMOTION,
            Nonlion::CHICK1 => Offset::CHICK1_PROMOTION,

            _ => return node,
        };
        NodeBuilder(node.0 | (1 << promotion_status_offset.0))
    }

    #[inline(always)]
    const fn is_piece1(self) -> bool {
        self.0.is_piece1()
    }

    #[inline(always)]
    const fn is_piece0_in_active_hand(self, node: NodeBuilder) -> bool {
        let piece0 = self.piece0();
        piece0.is_active(node) && piece0.is_in_hand(node)
    }

    #[inline(always)]
    const fn piece0(self) -> Self {
        Self(self.0.piece0())
    }
}

impl Coords {
    #[inline(always)]
    const fn board_offset(self: Coords) -> u8 {
        self.0 * 4
    }

    #[inline(always)]
    const fn is_in_last_row(self) -> bool {
        const LOOKUP_TABLE: u16 = 0b0111_0000_0000_0000;
        (LOOKUP_TABLE >> self.0) & 1 != 0
    }
}

impl CoordVec {
    const EMPTY: Self = Self(0);
    const MAX_ELEMENTS: usize = 13;

    /// If the vector is already full, then this function
    /// behaves as the identity function.
    #[must_use]
    #[inline(always)]
    const fn push(self, coords: Coords) -> Self {
        let len = self.0 & 0b1111;
        if len >= (Self::MAX_ELEMENTS as u64) {
            return self;
        }

        Self((self.0 | ((coords.0 as u64) << (len * 4 + 4))) + 1)
    }

    #[inline(always)]
    const fn singleton(coords: Coords) -> Self {
        Self(((coords.0 as u64) << 4) | 1)
    }
}

impl Iterator for CoordVec {
    type Item = Coords;

    fn next(&mut self) -> Option<Self::Item> {
        let len = self.0 & 0b1111;
        if len == 0 {
            return None;
        }

        let shifted = self.0 >> 4;
        let out = Coords((shifted & 0b1111) as u8);

        self.0 = (shifted & !(0b1111)) | (len - 1);

        Some(out)
    }
}

impl SearchNode {
    fn visit_children(self, visitor: impl FnMut(SearchNode)) {
        ChildCalculator::new(self).visit_children(visitor);
    }
}

impl ChildCalculator {
    const fn new(node: SearchNode) -> Self {
        let board = node.into_builder().board();
        let empty_squares = board.empty_squares();
        Self {
            node,
            board,
            empty_squares,
        }
    }

    #[inline(always)]
    fn visit_children(self, mut visitor: impl FnMut(SearchNode)) {
        if self.node.is_terminal() {
            return;
        }

        self.visit_children_with_actor(Actor::LION, &mut visitor);
        self.visit_children_with_actor(Actor::CHICK0, &mut visitor);
        self.visit_children_with_actor(Actor::CHICK1, &mut visitor);
        self.visit_children_with_actor(Actor::ELEPHANT0, &mut visitor);
        self.visit_children_with_actor(Actor::ELEPHANT1, &mut visitor);
        self.visit_children_with_actor(Actor::GIRAFFE0, &mut visitor);
        self.visit_children_with_actor(Actor::GIRAFFE1, &mut visitor);
    }

    #[inline(always)]
    fn visit_children_with_actor(self, actor: Actor, visitor: impl FnMut(SearchNode)) {
        let node = self.node.into_builder();

        if actor.is_passive(node) {
            return;
        }

        let start = actor.coords(node);

        if start == Coords::HAND {
            self.visit_dropping_children(actor, visitor);
        } else {
            self.visit_moving_children(actor, start, visitor);
        }
    }

    #[inline(always)]
    fn visit_dropping_children(self, actor: Actor, mut visitor: impl FnMut(SearchNode)) {
        let node = self.node.into_builder();

        // If two of the same species are in the active hand,
        // we must be careful to avoid double counting the associated child.
        if actor.is_piece1() && actor.is_piece0_in_active_hand(node) {
            return;
        }

        for dest in self.empty_squares {
            let node = actor.set_coords(node, dest);
            visitor(node.invert_active_player().build());
        }
    }

    #[inline(always)]
    fn visit_moving_children(
        self,
        actor: Actor,
        start: Coords,
        mut visitor: impl FnMut(SearchNode),
    ) {
        let node = self.node.into_builder();
        let is_promoted = actor.is_promoted(node);
        let dest_candidates = actor.legal_dest_squares(is_promoted, start);

        for dest in dest_candidates {
            let optional_node = node.vacate_passive(dest, self.board);
            if optional_node.is_none() {
                continue;
            }
            let node = optional_node.unchecked_unwrap();
            let node = actor.set_coords_and_promote_if_in_last_row(node, dest);
            visitor(node.invert_active_player().build());
        }
    }
}

impl SearchNode {
    fn visit_parents(self, visitor: impl FnMut(SearchNode)) {
        ParentCalculator::new(self).visit_parents(visitor);
    }
}

impl ParentCalculator {
    fn new(node: SearchNode) -> Self {
        let inverted_node = node.into_builder().invert_active_player();
        Self {
            inverted_node,
            inverted_board: inverted_node.board(),
        }
    }

    fn visit_parents(self, mut visitor: impl FnMut(SearchNode)) {
        self.visit_parents_with_actor(Actor::LION, &mut visitor);
        self.visit_parents_with_actor(Actor::CHICK0, &mut visitor);
        self.visit_parents_with_actor(Actor::CHICK1, &mut visitor);
        self.visit_parents_with_actor(Actor::ELEPHANT0, &mut visitor);
        self.visit_parents_with_actor(Actor::ELEPHANT1, &mut visitor);
        self.visit_parents_with_actor(Actor::GIRAFFE0, &mut visitor);
        self.visit_parents_with_actor(Actor::GIRAFFE1, &mut visitor);
    }

    #[inline(always)]
    fn visit_parents_with_actor(self, actor: Actor, mut visitor: impl FnMut(SearchNode)) {
        let node = self.inverted_node;
        if !(actor.is_active(node) && actor.is_on_board(node)) {
            return;
        }

        if !actor.is_hen(node) && !actor.is_lion() {
            self.visit_dropping_parent(actor, &mut visitor);
        }

        if actor.is_chick(node) && actor.is_in_last_row(node) {
            return;
        }

        self.visit_moving_parents(
            actor,
            ShouldDemoteActorInParent(false),
            actor.legal_starting_squares_in_state(node),
            &mut visitor,
        );

        if actor.is_hen(node) && actor.is_in_last_row(node) {
            let legal_starting_squares = CoordVec::singleton(Coords(actor.coords(node).0 - 0b0100));
            self.visit_moving_parents(
                actor,
                ShouldDemoteActorInParent(true),
                legal_starting_squares,
                &mut visitor,
            );
        }
    }

    #[inline(always)]
    fn visit_dropping_parent(self, actor: Actor, mut visitor: impl FnMut(SearchNode)) {
        let node = self.inverted_node;
        let coords = Coords::HAND;

        let out = node;
        let out = actor.set_coords(out, coords);
        if !out.is_terminal() {
            visitor(out.build());
        }
    }

    #[inline(always)]
    fn visit_moving_parents(
        self,
        actor: Actor,
        should_demote: ShouldDemoteActorInParent,
        starting_squares: CoordVec,
        mut visitor: impl FnMut(SearchNode),
    ) {
        let node = self.inverted_node;
        let board = self.inverted_board;

        let dest_square = actor.coords(node);

        for starting_square in starting_squares {
            if !board.is_square_empty(starting_square) {
                continue;
            }

            if PassiveLion.is_in_hand(node) {
                let out = node;
                let out = actor.set_coords(out, starting_square);
                let out = if should_demote.0 {
                    actor.demote(out)
                } else {
                    out
                };
                let out = PassiveLion.set_coords(out, dest_square);

                if !out.is_terminal() {
                    visitor(out.build());
                }

                // If the passive lion is in hand in the inverted current node,
                // then it must be on the board for all parent nodes.
                // Therefore, we should not consider parents where a non-lion is captured
                // or where no piece is captured.
                continue;
            }

            self.visit_noncapturing_moving_parent(
                actor,
                should_demote,
                starting_square,
                &mut visitor,
            );

            self.visit_capturing_parents(
                actor,
                should_demote,
                starting_square,
                dest_square,
                Nonlion::CHICK0,
                &mut visitor,
            );
            self.visit_capturing_parents(
                actor,
                should_demote,
                starting_square,
                dest_square,
                Nonlion::CHICK1,
                &mut visitor,
            );
            self.visit_capturing_parents(
                actor,
                should_demote,
                starting_square,
                dest_square,
                Nonlion::ELEPHANT0,
                &mut visitor,
            );
            self.visit_capturing_parents(
                actor,
                should_demote,
                starting_square,
                dest_square,
                Nonlion::ELEPHANT1,
                &mut visitor,
            );
            self.visit_capturing_parents(
                actor,
                should_demote,
                starting_square,
                dest_square,
                Nonlion::GIRAFFE0,
                &mut visitor,
            );
            self.visit_capturing_parents(
                actor,
                should_demote,
                starting_square,
                dest_square,
                Nonlion::GIRAFFE1,
                &mut visitor,
            );
        }
    }

    #[inline(always)]
    fn visit_noncapturing_moving_parent(
        self,
        actor: Actor,
        should_demote: ShouldDemoteActorInParent,
        starting_square: Coords,
        mut visitor: impl FnMut(SearchNode),
    ) {
        let node = self.inverted_node;

        let out = node;
        let out = actor.set_coords(out, starting_square);
        let out = if should_demote.0 {
            actor.demote(out)
        } else {
            out
        };

        if !out.is_terminal() {
            visitor(out.build());
        }
    }

    #[inline(always)]
    fn visit_capturing_parents(
        self,
        actor: Actor,
        should_demote: ShouldDemoteActorInParent,
        starting_square: Coords,
        dest_square: Coords,
        captive: Nonlion,
        mut visitor: impl FnMut(SearchNode),
    ) {
        let node = self.inverted_node;

        // If a piece was captured, then it would be moved to the active hand.
        if !(captive.is_active(node) && captive.is_in_hand(node)) {
            return;
        }

        // If two of the same species are in the active hand,
        // we must be careful to avoid double counting the associated parent.
        if captive.is_piece1() && captive.is_piece0_in_active_hand(node) {
            return;
        }

        if captive.is_bird() {
            let out = node;
            let out = actor.set_coords(out, starting_square);
            let out = if should_demote.0 {
                actor.demote(out)
            } else {
                out
            };
            let out = captive.set_coords(out, dest_square);
            let out = captive.make_passive(out);
            let out = captive.promote(out);

            if !out.is_terminal() {
                visitor(out.build());
            }
        }

        let out = node;
        let out = actor.set_coords(out, starting_square);
        let out = if should_demote.0 {
            actor.demote(out)
        } else {
            out
        };
        let out = captive.set_coords(out, dest_square);
        let out = captive.make_passive(out);
        if !out.is_terminal() {
            visitor(out.build());
        }
    }
}

impl NodeBuilder {
    /// - If the destination square is empty, this returns the original state.
    /// - If the destination square is occupied by a passive piece,
    ///   this returns the state with the passive piece moved to the active player's hand.
    /// - If the destination square is occupied by an active piece,
    ///   this returns `OptionalNodeBuilder::NONE`.
    #[inline(always)]
    const fn vacate_passive(self, dest: Coords, board: Board) -> OptionalNodeBuilder {
        let board_offset = dest.board_offset();

        let dest_square = (board.0 >> board_offset) & 0b1111;
        if dest_square == 0 {
            return self.into_optional();
        }

        // We cannot vacate an active piece.
        if dest_square & 0b1000 == 0 {
            return OptionalNodeBuilder::NONE;
        }

        let occupant = dest_square & 0b111;

        let occupant_coords_offset = [
            Offset::PASSIVE_LION_COLUMN,
            Offset::CHICK0_COLUMN,
            Offset::CHICK1_COLUMN,
            Offset::ELEPHANT0_COLUMN,
            Offset::ELEPHANT1_COLUMN,
            Offset::GIRAFFE0_COLUMN,
            Offset::GIRAFFE1_COLUMN,
        ][(occupant - 1) as usize];

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
}

impl Actor {
    #[inline(always)]
    const fn is_passive(self, node: NodeBuilder) -> bool {
        let allegiance_bit_offset = match self {
            // The active lion is never passive.
            Actor::LION => return false,

            Actor::CHICK0 => Offset::CHICK0_ALLEGIANCE,
            Actor::CHICK1 => Offset::CHICK1_ALLEGIANCE,
            Actor::ELEPHANT0 => Offset::ELEPHANT0_ALLEGIANCE,
            Actor::ELEPHANT1 => Offset::ELEPHANT1_ALLEGIANCE,
            Actor::GIRAFFE0 => Offset::GIRAFFE0_ALLEGIANCE,
            Actor::GIRAFFE1 => Offset::GIRAFFE1_ALLEGIANCE,

            _ => return false,
        };

        (node.0 & (1 << allegiance_bit_offset.0)) != 0
    }

    #[inline(always)]
    const fn is_active(self, node: NodeBuilder) -> bool {
        let allegiance_bit_offset = match self {
            // The active lion is always active.
            Actor::LION => return true,

            Actor::CHICK0 => Offset::CHICK0_ALLEGIANCE,
            Actor::CHICK1 => Offset::CHICK1_ALLEGIANCE,
            Actor::ELEPHANT0 => Offset::ELEPHANT0_ALLEGIANCE,
            Actor::ELEPHANT1 => Offset::ELEPHANT1_ALLEGIANCE,
            Actor::GIRAFFE0 => Offset::GIRAFFE0_ALLEGIANCE,
            Actor::GIRAFFE1 => Offset::GIRAFFE1_ALLEGIANCE,

            _ => return false,
        };

        (node.0 & (1 << allegiance_bit_offset.0)) == 0
    }

    #[inline(always)]
    const fn coords_mask(self) -> u64 {
        0b1111 << self.coords_offset().0
    }

    #[inline(always)]
    const fn coords_offset(self) -> Offset {
        match self {
            Actor::LION => Offset::ACTIVE_LION_COLUMN,
            Actor::CHICK0 => Offset::CHICK0_COLUMN,
            Actor::CHICK1 => Offset::CHICK1_COLUMN,
            Actor::ELEPHANT0 => Offset::ELEPHANT0_COLUMN,
            Actor::ELEPHANT1 => Offset::ELEPHANT1_COLUMN,
            Actor::GIRAFFE0 => Offset::GIRAFFE0_COLUMN,
            Actor::GIRAFFE1 => Offset::GIRAFFE1_COLUMN,

            _ => Offset(0),
        }
    }

    #[inline(always)]
    const fn coords(self, node: NodeBuilder) -> Coords {
        Coords(((node.0 >> self.coords_offset().0) & 0b1111) as u8)
    }

    #[must_use]
    #[inline(always)]
    const fn set_coords(self, node: NodeBuilder, coords: Coords) -> NodeBuilder {
        let coords_offset = self.coords_offset().0;
        NodeBuilder((node.0 & !(0b1111 << coords_offset)) | ((coords.0 as u64) << coords_offset))
    }

    #[must_use]
    #[inline(always)]
    const fn set_coords_and_promote_if_in_last_row(
        self,
        node: NodeBuilder,
        coords: Coords,
    ) -> NodeBuilder {
        let node = self.set_coords(node, coords);

        if self.is_bird() && coords.is_in_last_row() {
            return Chick(self.0).promote(node);
        }

        node
    }

    #[inline(always)]
    const fn is_in_last_row(self, node: NodeBuilder) -> bool {
        self.coords(node).is_in_last_row()
    }

    #[inline(always)]
    const fn is_on_board(self, node: NodeBuilder) -> bool {
        node.0 & self.coords_mask() != self.coords_mask()
    }

    #[inline(always)]
    const fn is_in_hand(self, node: NodeBuilder) -> bool {
        node.0 & self.coords_mask() == self.coords_mask()
    }

    #[inline(always)]
    const fn is_bird(self) -> bool {
        self.0.is_bird()
    }

    #[inline(always)]
    const fn is_chick(self, node: NodeBuilder) -> bool {
        let promotion_status_offset = match self {
            Actor::CHICK0 => Offset::CHICK0_PROMOTION,
            Actor::CHICK1 => Offset::CHICK1_PROMOTION,

            _ => return false,
        };
        node.0 & (1 << promotion_status_offset.0) == 0
    }

    #[inline(always)]
    const fn is_hen(self, node: NodeBuilder) -> bool {
        // Since hens are the only promoted pieces, it follows that
        // a piece is a hen if and only if it is promoted.
        self.is_promoted(node)
    }

    #[inline(always)]
    const fn is_lion(self) -> bool {
        self.0.is_lion()
    }

    #[inline(always)]
    const fn is_piece1(self) -> bool {
        self.0.is_piece1()
    }

    #[inline(always)]
    const fn is_piece0_in_active_hand(self, node: NodeBuilder) -> bool {
        let piece0 = self.piece0();
        piece0.is_active(node) && piece0.is_in_hand(node)
    }

    #[inline(always)]
    const fn piece0(self) -> Self {
        Self(self.0.piece0())
    }

    #[inline(always)]
    const fn is_promoted(self, node: NodeBuilder) -> bool {
        let promotion_status_offset = match self {
            Actor::CHICK0 => Offset::CHICK0_PROMOTION,
            Actor::CHICK1 => Offset::CHICK1_PROMOTION,

            _ => return false,
        };
        node.0 & (1 << promotion_status_offset.0) != 0
    }

    #[must_use]
    #[inline(always)]
    const fn demote(self, node: NodeBuilder) -> NodeBuilder {
        let promotion_status_offset = match self {
            Actor::CHICK0 => Offset::CHICK0_PROMOTION,
            Actor::CHICK1 => Offset::CHICK1_PROMOTION,

            _ => return node,
        };
        NodeBuilder(node.0 & !(1 << promotion_status_offset.0))
    }

    #[inline(always)]
    const fn legal_starting_squares_in_state(self, node: NodeBuilder) -> CoordVec {
        let is_promoted = self.is_promoted(node);
        let dest = self.coords(node);
        self.legal_starting_squares(is_promoted, dest)
    }
}

impl Piece {
    /// A bird is a chick or a hen.
    #[inline(always)]
    const fn is_bird(self) -> bool {
        self.0 & !1 == 0b010
    }

    #[inline(always)]
    const fn is_lion(self) -> bool {
        self.0 == 0b001
    }

    #[inline(always)]
    const fn is_piece1(self) -> bool {
        const LOOKUP_TABLE: u16 =
            (1 << Piece::CHICK1.0) | (1 << Piece::ELEPHANT1.0) | (1 << Piece::GIRAFFE1.0);
        LOOKUP_TABLE & (1 << self.0) != 0
    }

    #[inline(always)]
    const fn piece0(self) -> Self {
        match self {
            Self::CHICK1 => Self::CHICK0,
            Self::ELEPHANT1 => Self::ELEPHANT0,
            Self::GIRAFFE1 => Self::GIRAFFE0,

            _ => self,
        }
    }
}

impl Chick {
    #[inline(always)]
    const fn promote(self, node: NodeBuilder) -> NodeBuilder {
        let promotion_status_offset = match self {
            Chick::CHICK0 => Offset::CHICK0_PROMOTION,
            Chick::CHICK1 => Offset::CHICK1_PROMOTION,

            _ => return node,
        };
        NodeBuilder(node.0 | (1 << promotion_status_offset.0))
    }
}

impl PassiveLion {
    const COORDS_OFFSET: Offset = Offset::PASSIVE_LION_COLUMN;
    const COORDS_MASK: u64 = 0b1111 << Self::COORDS_OFFSET.0;

    #[must_use]
    #[inline(always)]
    const fn set_coords(self, node: NodeBuilder, coords: Coords) -> NodeBuilder {
        let coords_offset = Self::COORDS_OFFSET.0;
        NodeBuilder((node.0 & !(0b1111 << coords_offset)) | ((coords.0 as u64) << coords_offset))
    }

    #[inline(always)]
    const fn is_in_hand(self, node: NodeBuilder) -> bool {
        node.0 & Self::COORDS_MASK == Self::COORDS_MASK
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

impl NodeBuilder {
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
    const fn empty_squares(self) -> CoordVec {
        let mut buffer = 0;
        let mut buffer_offset = 4;

        macro_rules! check_square {
            ($coords:expr) => {{
                const MASK: u64 = 0b1111 << $coords.board_offset();
                if self.0 & MASK == 0 {
                    buffer |= ($coords.0 as u64) << buffer_offset;
                    buffer_offset += 4;
                }
            }};
        }

        check_square!(Coords::R0C0);
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

        let len = buffer_offset / 4 - 1;
        buffer |= len;

        CoordVec(buffer)
    }

    #[inline(always)]
    const fn is_square_empty(self, square: Coords) -> bool {
        let mask = 0b1111 << square.board_offset();
        self.0 & mask == 0
    }
}

mod piece_movement_directions {
    use super::*;

    #[derive(Copy, Clone, Debug)]
    struct IsPromoted(bool);

    /// In accordance with DRY,
    /// instead of implementing separate functions for
    /// finding the legal starting squares and the legal destination squares,
    /// we write a generic function that can do both.
    /// This generic function takes a parameter `unknown: Unknown` that specifies
    /// whether the function should return the legal starting squares or the legal destination squares.
    ///
    /// We derive the name "unknown" from known and unknown variables in elementary algebra
    /// (e.g., in the problem "Solve for `x` in `x + 3 = 5 + k` where `k = 5`", `x` is an unknown variable,
    /// and `k` is a known variable).
    #[derive(Copy, Clone, Debug)]
    enum Unknown {
        Start,
        Dest,
    }

    /// This function should only be called during compile-time.
    /// Consequently, we don't have to worry about the performance
    /// inside of it.
    /// Thus, we can use a simple struct with 8 boolean fields
    /// instead of a more efficient `u8` bitset.
    #[derive(Copy, Clone, Debug)]
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

    impl Actor {
        #[inline(always)]
        pub(crate) const fn legal_dest_squares(self, is_promoted: bool, start: Coords) -> CoordVec {
            const LOOKUP_TABLE: [[[CoordVec; 16]; 8]; 2] =
                Actor::slowly_compute_combined_lookup_table(Unknown::Dest);

            LOOKUP_TABLE[is_promoted as usize][self.0 .0 as usize][start.0 as usize]
        }

        #[inline(always)]
        pub(crate) const fn legal_starting_squares(
            self,
            is_promoted: bool,
            dest: Coords,
        ) -> CoordVec {
            const LOOKUP_TABLE: [[[CoordVec; 16]; 8]; 2] =
                Actor::slowly_compute_combined_lookup_table(Unknown::Start);

            LOOKUP_TABLE[is_promoted as usize][self.0 .0 as usize][dest.0 as usize]
        }

        #[inline(always)]
        const fn slowly_compute_combined_lookup_table(
            unknown: Unknown,
        ) -> [[[CoordVec; 16]; 8]; 2] {
            [
                Actor::slowly_compute_lookup_table(IsPromoted(false), unknown),
                Actor::slowly_compute_lookup_table(IsPromoted(true), unknown),
            ]
        }

        #[inline(always)]
        const fn slowly_compute_lookup_table(
            is_promoted: IsPromoted,
            unknown: Unknown,
        ) -> [[CoordVec; 16]; 8] {
            [
                [CoordVec::EMPTY; 16],
                Actor::LION.slowly_compute_lookup_table_row(is_promoted, unknown),
                Actor::CHICK0.slowly_compute_lookup_table_row(is_promoted, unknown),
                Actor::CHICK1.slowly_compute_lookup_table_row(is_promoted, unknown),
                Actor::ELEPHANT0.slowly_compute_lookup_table_row(is_promoted, unknown),
                Actor::ELEPHANT1.slowly_compute_lookup_table_row(is_promoted, unknown),
                Actor::GIRAFFE0.slowly_compute_lookup_table_row(is_promoted, unknown),
                Actor::GIRAFFE1.slowly_compute_lookup_table_row(is_promoted, unknown),
            ]
        }

        #[inline(always)]
        const fn slowly_compute_lookup_table_row(
            self,
            is_promoted: IsPromoted,
            unknown: Unknown,
        ) -> [CoordVec; 16] {
            macro_rules! check_square {
                ($coords:expr) => {
                    self.slowly_compute_legal_squares(is_promoted, unknown, $coords)
                };
            }

            [
                check_square!(Coords::R0C0),
                check_square!(Coords::R0C1),
                check_square!(Coords::R0C2),
                CoordVec::EMPTY,
                check_square!(Coords::R1C0),
                check_square!(Coords::R1C1),
                check_square!(Coords::R1C2),
                CoordVec::EMPTY,
                check_square!(Coords::R2C0),
                check_square!(Coords::R2C1),
                check_square!(Coords::R2C2),
                CoordVec::EMPTY,
                check_square!(Coords::R3C0),
                check_square!(Coords::R3C1),
                check_square!(Coords::R3C2),
                CoordVec::EMPTY,
            ]
        }

        const fn slowly_compute_legal_squares(
            self,
            is_promoted: IsPromoted,
            unknown: Unknown,
            known: Coords,
        ) -> CoordVec {
            let mut out = CoordVec::EMPTY;

            let dirset = if is_promoted.0 {
                self.promoted_dirset()
            } else {
                self.nonpromoted_dirset()
            };

            macro_rules! check_candidate {
                ($candidate:expr) => {
                    match unknown {
                        Unknown::Start => {
                            if dirset.connects($candidate, known) {
                                out = out.push($candidate);
                            }
                        }
                        Unknown::Dest => {
                            if dirset.connects(known, $candidate) {
                                out = out.push($candidate);
                            }
                        }
                    }
                };
            }

            check_candidate!(Coords::R0C0);
            check_candidate!(Coords::R0C1);
            check_candidate!(Coords::R0C2);

            check_candidate!(Coords::R1C0);
            check_candidate!(Coords::R1C1);
            check_candidate!(Coords::R1C2);

            check_candidate!(Coords::R2C0);
            check_candidate!(Coords::R2C1);
            check_candidate!(Coords::R2C2);

            check_candidate!(Coords::R3C0);
            check_candidate!(Coords::R3C1);
            check_candidate!(Coords::R3C2);

            out
        }

        #[inline(always)]
        const fn nonpromoted_dirset(self) -> DirectionSet {
            match self {
                Actor::LION => LION,
                Actor::CHICK0 => CHICK,
                Actor::CHICK1 => CHICK,
                Actor::ELEPHANT0 => ELEPHANT,
                Actor::ELEPHANT1 => ELEPHANT,
                Actor::GIRAFFE0 => GIRAFFE,
                Actor::GIRAFFE1 => GIRAFFE,

                _ => EMPTY,
            }
        }

        #[inline(always)]
        const fn promoted_dirset(self) -> DirectionSet {
            match self {
                Actor::CHICK0 => HEN,
                Actor::CHICK1 => HEN,

                _ => EMPTY,
            }
        }
    }
}

impl Offset {
    const BEST_KNOWN_OUTCOME: Self = Self(0);
    const REQUIRED_CHILD_REPORT_COUNT: Self = Self(Self::BEST_KNOWN_OUTCOME.0 + 9);
    const PASSIVE_LION: Self = Self(Self::REQUIRED_CHILD_REPORT_COUNT.0 + 7);
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

impl Actor {
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

impl Chick {
    const CHICK0: Self = Self(Piece::CHICK0);
    const CHICK1: Self = Self(Piece::CHICK1);
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

/// `-200`` in 9-bit two's complement, left-padded with zeros
/// to fill the 64-bit integer.
const NEGATIVE_201_I9: u64 = 0b1_0011_0111;

/// `200` in 9-bit two's complement, left-padded with zeros
/// to fill the 64-bit integer.
const POSITIVE_201_I9: u64 = 0b0_1100_1001;
