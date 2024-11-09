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

    while let Some(top) = known_stack.pop() {
        let outcome = top.best_known_outcome();

        top.visit_parents(|parent| {
            let Ok(parent_index) = states.binary_search(&parent) else {
                // It's possible that a theoretical parent is actually unreachable.
                return;
            };

            let parent_mut = &mut states[parent_index];
            *parent_mut = parent_mut.record_child_outcome(outcome);
            if parent_mut.unknown_child_count() == 0 {
                known_stack.push(*parent_mut);
            }
        });
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
    const DELETION_MASK: u64 = !((0b111_1111 << Offset::UNKNOWN_CHILD_COUNT.0)
        | (0b1_1111_1111 << Offset::BEST_KNOWN_OUTCOME.0));

    for state in states {
        match state.terminality() {
            Terminality::Nonterminal => {
                state.0 = (state.0 & DELETION_MASK)
                    | ((state.total_child_count() as u64) << Offset::UNKNOWN_CHILD_COUNT.0)
                    | (NEGATIVE_201_I9 << Offset::BEST_KNOWN_OUTCOME.0);
            }

            Terminality::Win => {
                state.0 = (state.0 & DELETION_MASK)
                    | (0 << Offset::UNKNOWN_CHILD_COUNT.0)
                    | (POSITIVE_201_I9 << Offset::BEST_KNOWN_OUTCOME.0);
            }

            Terminality::Loss => {
                state.0 = (state.0 & DELETION_MASK)
                    | (0 << Offset::UNKNOWN_CHILD_COUNT.0)
                    | (NEGATIVE_201_I9 << Offset::BEST_KNOWN_OUTCOME.0);
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
        ((self.0 >> Offset::UNKNOWN_CHILD_COUNT.0) & 0b111_1111) as u8
    }

    fn best_known_outcome(self) -> Outcome {
        Outcome(i16::from_zero_padded_i9(
            (self.0 >> Offset::BEST_KNOWN_OUTCOME.0) & 0b1_1111_1111,
        ))
    }

    fn record_child_outcome(self, child_outcome: Outcome) -> Self {
        let incumbent = self.best_known_outcome();
        let challenger = child_outcome.invert().delay_by_one();
        if challenger > incumbent {
            Self(
                self.0 & !(0b1_1111_1111 << Offset::BEST_KNOWN_OUTCOME.0)
                    | (challenger.0.into_zero_padded_i9_unchecked()
                        << Offset::BEST_KNOWN_OUTCOME.0),
            )
        } else {
            self
        }
    }
}

impl SearchNode {
    fn visit_parents(self, mut visitor: impl FnMut(SearchNode)) {
        let inverted = self.into_builder().invert_active_player();
        inverted.visit_parents_with_actor(ActivePiece::LION, &mut visitor);
        inverted.visit_parents_with_actor(ActivePiece::CHICK0, &mut visitor);
        inverted.visit_parents_with_actor(ActivePiece::CHICK1, &mut visitor);
        inverted.visit_parents_with_actor(ActivePiece::ELEPHANT0, &mut visitor);
        inverted.visit_parents_with_actor(ActivePiece::ELEPHANT1, &mut visitor);
        inverted.visit_parents_with_actor(ActivePiece::GIRAFFE0, &mut visitor);
        inverted.visit_parents_with_actor(ActivePiece::GIRAFFE1, &mut visitor);
    }
}

impl NodeBuilder {
    fn visit_parents_with_actor(self, actor: ActivePiece, mut visitor: impl FnMut(SearchNode)) {
        if self.is_actor_passive(actor) || self.is_in_hand(actor) {
            return;
        }

        // You cannot directly drop a hen.
        if self.is_nonpromoted(actor) {
            visitor(self.dropping_parent_of_nonpromoted_actor(actor).build());
        }

        self.visit_moving_parents_assuming_actor_is_active_and_on_board(actor, visitor);
    }

    #[inline(always)]
    const fn is_in_hand(self, actor: ActivePiece) -> bool {
        self.actor_coords(actor).0 == Coords::HAND.0
    }

    #[inline(always)]
    const fn is_nonpromoted(self, actor: ActivePiece) -> bool {
        let promotion_bit_offset = match actor {
            ActivePiece::CHICK0 => Offset::CHICK0_PROMOTION,
            ActivePiece::CHICK1 => Offset::CHICK1_PROMOTION,

            _ => return false,
        };

        (self.0 & (1 << promotion_bit_offset.0)) == 0
    }

    #[inline(always)]
    const fn dropping_parent_of_nonpromoted_actor(self, actor: ActivePiece) -> Self {
        self.set_coords_without_demoting(actor, Coords::HAND)
    }

    #[inline(always)]
    const fn set_coords_without_demoting(self, actor: ActivePiece, coords: Coords) -> Self {
        Self((self.0 & !actor.coords_mask()) | (coords.0 as u64))
    }

    #[inline(always)]
    fn visit_moving_parents_assuming_actor_is_active_and_on_board(
        self,
        actor: ActivePiece,
        visitor: impl FnMut(SearchNode),
    ) {
        if self.is_nonpromoted(actor) {
            return self
                .visit_moving_parents_assuming_actor_is_active_and_on_board_and_nonpromoted(
                    actor, visitor,
                );
        }

        self.visit_moving_parents_assuming_actor_is_active_and_on_board_and_promoted(
            actor, visitor,
        );
    }

    #[inline(always)]
    fn visit_moving_parents_assuming_actor_is_active_and_on_board_and_nonpromoted(
        self,
        actor: ActivePiece,
        mut visitor: impl FnMut(SearchNode),
    ) {
        let starting_squares = actor.legal_starting_squares(self.actor_coords(actor))[0];
        for starting_square in starting_squares {
            self.visit_noncapturing_moving_parent_assuming_actor_is_active_and_on_board_and_non_promoted(
                actor,
                starting_square,
                &mut visitor,
            );

            macro_rules! visit {
                ($captive:expr) => {
                    self.visit_capturing_moving_parents_assuming_actor_is_active_and_on_board_and_non_promoted(
                        actor,
                        starting_square,
                        $captive,
                        &mut visitor,
                    );
                };
            }

            visit!(PassivePiece::LION);
            visit!(PassivePiece::CHICK0);
            visit!(PassivePiece::CHICK1);
            visit!(PassivePiece::ELEPHANT0);
            visit!(PassivePiece::ELEPHANT1);
            visit!(PassivePiece::GIRAFFE0);
            visit!(PassivePiece::GIRAFFE1);
        }
    }

    #[inline(always)]
    fn visit_noncapturing_moving_parent_assuming_actor_is_active_and_on_board_and_non_promoted(
        self,
        actor: ActivePiece,
        starting_square: Coords,
        mut visitor: impl FnMut(SearchNode),
    ) {
        visitor(
            self.set_coords_without_demoting(actor, starting_square)
                .build(),
        );
    }

    #[inline(always)]
    fn visit_capturing_moving_parents_assuming_actor_is_active_and_on_board_and_non_promoted(
        self,
        actor: ActivePiece,
        starting_square: Coords,
        captive: PassivePiece,
        mut visitor: impl FnMut(SearchNode),
    ) {
        // TODO: If the captive is a chick,
        // then in the parent state,
        // the captive could have been a chick
        // but it also could have been a hen.

        todo!()
    }

    #[inline(always)]
    fn visit_moving_parents_assuming_actor_is_active_and_on_board_and_promoted(
        self,
        actor: ActivePiece,
        visitor: impl FnMut(SearchNode),
    ) {
        // TODO: Consider the case where a hen is on the last row.
        // We need to visit up to 6 parents (instead of 1):
        // 1. The parent where a chick moved onto the last row.
        // 2-6. The parents where the hen moved onto the last row.

        todo!()
    }
}

impl Outcome {
    const fn invert(self) -> Self {
        Self(-self.0)
    }

    const fn delay_by_one(self) -> Self {
        Self(self.0 - self.0.signum())
    }
}

/// `-200`` in 9-bit two's complement, left-padded with zeros
/// to fill the 64-bit integer.
const NEGATIVE_201_I9: u64 = 0b1_0011_0111;

/// `200` in 9-bit two's complement, left-padded with zeros
/// to fill the 64-bit integer.
const POSITIVE_201_I9: u64 = 0b0_1100_1001;
