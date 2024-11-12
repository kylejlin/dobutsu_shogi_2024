use super::*;

#[cfg(test)]
mod tests;

/// This function will solve the game when provided
/// with a slice of all reachable states.
///
/// The slice of states will be sorted.
pub fn solve(states: &mut [SearchNode], mut on_node_processed: impl FnMut(SearchNode)) {
    states.sort_unstable();

    init_required_child_report_count_and_best_known_outcome(states);

    let mut known_stack = vec![];
    add_terminal_nodes(states, &mut known_stack);

    while let Some(top) = known_stack.pop() {
        let outcome = top.best_known_outcome();

        top.visit_parents(|parent| {
            let parent_state = parent.state();
            let Ok(parent_index) = states.binary_search_by(|other| {
                let other_state = other.state();
                other_state.cmp(&parent_state)
            }) else {
                // It's possible that a theoretical parent is actually unreachable.
                return;
            };

            let parent_mut = &mut states[parent_index];
            *parent_mut = parent_mut
                .record_child_outcome(outcome)
                .decrement_required_child_report_count();
            if parent_mut.required_child_report_count() == 0 {
                known_stack.push(*parent_mut);
            }
        });

        on_node_processed(top);
    }
}

#[derive(Clone, Copy, Debug)]
struct RequireLionCapture(bool);

fn init_required_child_report_count_and_best_known_outcome(states: &mut [SearchNode]) {
    const DELETION_MASK: u64 = !((0b111_1111 << Offset::REQUIRED_CHILD_REPORT_COUNT.0)
        | (0b1_1111_1111 << Offset::BEST_KNOWN_OUTCOME.0));

    for state in states {
        match state.terminality() {
            Terminality::Nonterminal => {
                state.0 = (state.0 & DELETION_MASK)
                    | ((state.total_child_count() as u64) << Offset::REQUIRED_CHILD_REPORT_COUNT.0)
                    | (NEGATIVE_201_I9 << Offset::BEST_KNOWN_OUTCOME.0);
            }

            Terminality::Win => {
                state.0 = (state.0 & DELETION_MASK)
                    | (0 << Offset::REQUIRED_CHILD_REPORT_COUNT.0)
                    | (POSITIVE_201_I9 << Offset::BEST_KNOWN_OUTCOME.0);
            }

            Terminality::Loss => {
                state.0 = (state.0 & DELETION_MASK)
                    | (0 << Offset::REQUIRED_CHILD_REPORT_COUNT.0)
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
        let mut count = 0;
        self.visit_children(|_| count += 1);
        count
    }

    #[must_use]
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

    #[must_use]
    fn decrement_required_child_report_count(self) -> Self {
        Self(self.0 - (1 << Offset::REQUIRED_CHILD_REPORT_COUNT.0))
    }
}

impl SearchNode {
    fn visit_parents(self, mut visitor: impl FnMut(SearchNode)) {
        let inverted = self.into_builder().invert_active_player();
        inverted.visit_parents_with_actor(Actor::LION, &mut visitor);
        inverted.visit_parents_with_actor(Actor::CHICK0, &mut visitor);
        inverted.visit_parents_with_actor(Actor::CHICK1, &mut visitor);
        inverted.visit_parents_with_actor(Actor::ELEPHANT0, &mut visitor);
        inverted.visit_parents_with_actor(Actor::ELEPHANT1, &mut visitor);
        inverted.visit_parents_with_actor(Actor::GIRAFFE0, &mut visitor);
        inverted.visit_parents_with_actor(Actor::GIRAFFE1, &mut visitor);
    }
}

impl NodeBuilder {
    fn visit_parents_with_actor(self, actor: Actor, mut visitor: impl FnMut(SearchNode)) {
        if self.is_actor_passive(actor) || self.is_actor_in_hand(actor) {
            return;
        }

        if self.is_passive_lion_in_hand() {
            self.visit_moving_parents(actor, RequireLionCapture(true), visitor);
            return;
        }

        // You cannot drop a hen or a lion.
        if self.is_nonpromoted(actor) && actor.0 != Actor::LION.0 {
            visitor(self.dropping_parent_of_nonpromoted_actor(actor).build());
        }

        self.visit_moving_parents(actor, RequireLionCapture(false), visitor);
    }

    #[inline(always)]
    const fn is_actor_in_hand(self, actor: Actor) -> bool {
        let mask = actor.coords_mask();
        self.0 & mask == mask
    }

    #[inline(always)]
    const fn is_passive_lion_in_hand(self) -> bool {
        const MASK: u64 = Captive::LION.coords_mask();
        self.0 & MASK == MASK
    }

    #[inline(always)]
    const fn is_nonpromoted(self, actor: Actor) -> bool {
        let promotion_bit_offset = match actor {
            Actor::CHICK0 => Offset::CHICK0_PROMOTION,
            Actor::CHICK1 => Offset::CHICK1_PROMOTION,

            _ => return true,
        };

        (self.0 & (1 << promotion_bit_offset.0)) == 0
    }

    #[inline(always)]
    const fn dropping_parent_of_nonpromoted_actor(self, actor: Actor) -> Self {
        self.set_actor_coords_without_demoting(actor, Coords::HAND)
    }

    #[inline(always)]
    const fn set_actor_coords_without_demoting(self, actor: Actor, coords: Coords) -> Self {
        Self((self.0 & !actor.coords_mask()) | ((coords.0 as u64) << actor.coords_offset().0))
    }

    /// Precondition: The actor is has active allegiance and is on the board.
    #[inline(always)]
    fn visit_moving_parents(
        self,
        actor: Actor,
        require_lion_capture: RequireLionCapture,
        visitor: impl FnMut(SearchNode),
    ) {
        if self.is_nonpromoted(actor) {
            return self.visit_moving_parents_assuming_nonpromoted_actor(
                actor,
                require_lion_capture,
                visitor,
            );
        }

        // If the actor is promoted, it must be a chick.
        let actor = Chick(actor.0);

        self.visit_moving_parents_assuming_promoted_actor(actor, require_lion_capture, visitor);
    }

    #[inline(always)]
    fn visit_moving_parents_assuming_nonpromoted_actor(
        self,
        actor: Actor,
        require_lion_capture: RequireLionCapture,
        visitor: impl FnMut(SearchNode),
    ) {
        // A nonpromoted chick can only be on the last row
        // if it was dropped there.
        // Had it moved there, it would have been promoted.
        if actor.is_chick() && self.actor_coords(actor).in_last_row() {
            return;
        }

        let starting_squares = actor.legal_starting_squares(false, self.actor_coords(actor));
        self.visit_moving_parents_assuming_no_promotion(
            actor,
            require_lion_capture,
            starting_squares,
            visitor,
        );
    }

    #[inline(always)]
    fn visit_moving_parents_assuming_no_promotion(
        self,
        actor: Actor,
        require_lion_capture: RequireLionCapture,
        starting_squares: CoordVec,
        mut visitor: impl FnMut(SearchNode),
    ) {
        for starting_square in starting_squares {
            macro_rules! visit {
                ($captive_candidate:expr) => {
                    self.visit_capturing_moving_parents_assuming_no_promotion(
                        actor,
                        starting_square,
                        $captive_candidate,
                        &mut visitor,
                    )
                };
            }

            visit!(Captive::LION);

            if require_lion_capture.0 {
                continue;
            }

            // We must take care to avoid visiting duplicate parents.
            // This can happen if two pieces of the same species
            // are both in the hand.
            // This is because capturing, say, chick0, and capturing
            // chick1 would get counted as two separate parents,
            // when they are actually the same parent.
            //
            // To avoid double counting, we skip the second piece
            // of every species if the first piece has already been visited.

            if !visit!(Captive::CHICK0) {
                visit!(Captive::CHICK1);
            }

            if !visit!(Captive::ELEPHANT0) {
                visit!(Captive::ELEPHANT1);
            }

            if !visit!(Captive::GIRAFFE0) {
                visit!(Captive::GIRAFFE1);
            }

            self.visit_noncapturing_moving_parent_assuming_no_promotion(
                actor,
                starting_square,
                &mut visitor,
            );
        }
    }

    #[inline(always)]
    fn visit_noncapturing_moving_parent_assuming_no_promotion(
        self,
        actor: Actor,
        starting_square: Coords,
        mut visitor: impl FnMut(SearchNode),
    ) {
        visitor(
            self.set_actor_coords_without_demoting(actor, starting_square)
                .build(),
        );
    }

    /// Returns whether the captive candidate is in the active player's hand.
    #[inline(always)]
    fn visit_capturing_moving_parents_assuming_no_promotion(
        self,
        actor: Actor,
        starting_square: Coords,
        captive_candidate: Captive,
        mut visitor: impl FnMut(SearchNode),
    ) -> bool {
        if !self.is_in_active_hand(captive_candidate) {
            return false;
        }

        let captive = captive_candidate;

        let dest_square = self.actor_coords(actor);
        visitor(
            self.set_actor_coords_without_demoting(actor, starting_square)
                .set_captive_coords_without_demoting(captive, dest_square)
                .build(),
        );

        if captive.is_chick() {
            visitor(
                self.set_actor_coords_without_demoting(actor, starting_square)
                    .set_captive_coords_without_demoting(captive, dest_square)
                    .promote(Chick(captive.0))
                    .build(),
            );
        }

        true
    }

    #[inline(always)]
    fn is_in_active_hand(self, piece: Captive) -> bool {
        self.is_captive_candidate_in_hand(piece) && self.is_captive_candidate_active(piece)
    }

    #[inline(always)]
    const fn is_captive_candidate_in_hand(self, piece: Captive) -> bool {
        let mask = piece.coords_mask();
        self.0 & mask == mask
    }

    #[inline(always)]
    const fn is_captive_candidate_active(self, piece: Captive) -> bool {
        let allegiance_bit_offset = match piece {
            Captive::LION => return self.is_captive_candidate_in_hand(Captive::LION),

            Captive::CHICK0 => Offset::CHICK0_ALLEGIANCE,
            Captive::CHICK1 => Offset::CHICK1_ALLEGIANCE,
            Captive::ELEPHANT0 => Offset::ELEPHANT0_ALLEGIANCE,
            Captive::ELEPHANT1 => Offset::ELEPHANT1_ALLEGIANCE,
            Captive::GIRAFFE0 => Offset::GIRAFFE0_ALLEGIANCE,
            Captive::GIRAFFE1 => Offset::GIRAFFE1_ALLEGIANCE,

            _ => return false,
        };
        self.0 & (1 << allegiance_bit_offset.0) == 0
    }

    #[inline(always)]
    const fn set_captive_coords_without_demoting(self, piece: Captive, coords: Coords) -> Self {
        Self((self.0 & !piece.coords_mask()) | ((coords.0 as u64) << piece.coords_offset().0))
    }

    #[inline(always)]
    const fn promote(self, chick: Chick) -> Self {
        let promotion_bit_offset = match chick {
            Chick::CHICK0 => Offset::CHICK0_PROMOTION,
            Chick::CHICK1 => Offset::CHICK1_PROMOTION,

            _ => return self,
        };

        Self(self.0 | (1 << promotion_bit_offset.0))
    }

    #[inline(always)]
    fn visit_moving_parents_assuming_promoted_actor(
        self,
        actor: Chick,
        require_lion_capture: RequireLionCapture,
        mut visitor: impl FnMut(SearchNode),
    ) {
        let dest = self.actor_coords(Actor(actor.0));
        let starting_squares = Actor(actor.0).legal_starting_squares(true, dest);
        self.visit_moving_parents_assuming_no_promotion(
            Actor(actor.0),
            require_lion_capture,
            starting_squares,
            &mut visitor,
        );

        if dest.in_last_row() {
            let starting_squares = Actor(actor.0).legal_starting_squares(false, dest);
            // A chick only has one legal move, so we can skip the loop.
            let starting_square = Coords((starting_squares.0 & 0b1111) as u8);

            self.visit_moving_parents_assuming_promotion(
                actor,
                require_lion_capture,
                starting_square,
                &mut visitor,
            );
        }
    }

    #[inline(always)]
    fn visit_moving_parents_assuming_promotion(
        self,
        actor: Chick,
        require_lion_capture: RequireLionCapture,
        starting_square: Coords,
        mut visitor: impl FnMut(SearchNode),
    ) {
        macro_rules! visit {
            ($captive_candidate:expr) => {
                self.visit_capturing_moving_parents_assuming_promotion(
                    actor,
                    starting_square,
                    $captive_candidate,
                    &mut visitor,
                )
            };
        }

        visit!(Captive::LION);

        if require_lion_capture.0 {
            return;
        }

        // We must take care to avoid visiting duplicate parents.
        // This can happen if two pieces of the same species
        // are both in the hand.
        // This is because capturing, say, chick0, and capturing
        // chick1 would get counted as two separate parents,
        // when they are actually the same parent.
        //
        // To avoid double counting, we skip the second piece
        // of every species if the first piece has already been visited.

        if !visit!(Captive::CHICK0) {
            visit!(Captive::CHICK1);
        }

        if !visit!(Captive::ELEPHANT0) {
            visit!(Captive::ELEPHANT1);
        }

        if !visit!(Captive::GIRAFFE0) {
            visit!(Captive::GIRAFFE1);
        }

        self.visit_noncapturing_moving_parent_assuming_promotion(
            actor,
            starting_square,
            &mut visitor,
        );
    }

    #[inline(always)]
    fn visit_noncapturing_moving_parent_assuming_promotion(
        self,
        actor: Chick,
        starting_square: Coords,
        mut visitor: impl FnMut(SearchNode),
    ) {
        visitor(
            self.set_actor_coords_without_demoting(Actor(actor.0), starting_square)
                .demote(actor)
                .build(),
        );
    }

    #[inline(always)]
    const fn demote(self, chick: Chick) -> Self {
        let promotion_bit_offset = match chick {
            Chick::CHICK0 => Offset::CHICK0_PROMOTION,
            Chick::CHICK1 => Offset::CHICK1_PROMOTION,

            _ => return self,
        };

        Self(self.0 & !(1 << promotion_bit_offset.0))
    }

    /// Returns whether the captive candidate is in the active player's hand.
    #[inline(always)]
    fn visit_capturing_moving_parents_assuming_promotion(
        self,
        actor: Chick,
        starting_square: Coords,
        captive_candidate: Captive,
        mut visitor: impl FnMut(SearchNode),
    ) -> bool {
        if !self.is_in_active_hand(captive_candidate) {
            return false;
        }

        let captive = captive_candidate;

        let upcast_actor = Actor(actor.0);

        let dest_square = self.actor_coords(upcast_actor);
        visitor(
            self.set_actor_coords_without_demoting(upcast_actor, starting_square)
                .demote(actor)
                .set_captive_coords_without_demoting(captive, dest_square)
                .build(),
        );

        if captive.is_chick() {
            visitor(
                self.set_actor_coords_without_demoting(upcast_actor, starting_square)
                    .demote(actor)
                    .set_captive_coords_without_demoting(captive, dest_square)
                    .promote(Chick(captive.0))
                    .build(),
            );
        }

        true
    }
}

impl Captive {
    #[inline(always)]
    const fn is_chick(self) -> bool {
        self.0.is_chick()
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
