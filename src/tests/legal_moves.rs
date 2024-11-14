use super::*;

#[derive(Clone, Copy)]
struct CoordSet(u16);

#[test]
fn every_start_square_is_lists_original_square_as_dest_square() {
    every_actor_start_square_is_lists_original_square_as_dest_square(Actor::LION, false);
    every_actor_start_square_is_lists_original_square_as_dest_square(Actor::LION, true);

    every_actor_start_square_is_lists_original_square_as_dest_square(Actor::CHICK0, false);
    every_actor_start_square_is_lists_original_square_as_dest_square(Actor::CHICK0, true);

    every_actor_start_square_is_lists_original_square_as_dest_square(Actor::CHICK1, false);
    every_actor_start_square_is_lists_original_square_as_dest_square(Actor::CHICK1, true);

    every_actor_start_square_is_lists_original_square_as_dest_square(Actor::ELEPHANT0, false);
    every_actor_start_square_is_lists_original_square_as_dest_square(Actor::ELEPHANT0, true);

    every_actor_start_square_is_lists_original_square_as_dest_square(Actor::ELEPHANT1, false);
    every_actor_start_square_is_lists_original_square_as_dest_square(Actor::ELEPHANT1, true);

    every_actor_start_square_is_lists_original_square_as_dest_square(Actor::GIRAFFE0, false);
    every_actor_start_square_is_lists_original_square_as_dest_square(Actor::GIRAFFE0, true);

    every_actor_start_square_is_lists_original_square_as_dest_square(Actor::GIRAFFE1, false);
    every_actor_start_square_is_lists_original_square_as_dest_square(Actor::GIRAFFE1, true);
}

fn every_actor_start_square_is_lists_original_square_as_dest_square(
    actor: Actor,
    is_promoted: bool,
) {
    for original_square in CoordVec::ALL_BOARD_SQUARES {
        let start_squares = actor.legal_starting_squares(is_promoted, original_square);
        for start_square in start_squares {
            let dest_squares = actor
                .legal_dest_squares(is_promoted, start_square)
                .into_set();
            if !dest_squares.contains(original_square) {
                panic!(
                    "When {actor:?} was at {original_square:?}, it listed {start_square:?} as a starting square, but when it was at said starting square (i.e., {start_square:?}), it did not list {original_square:?} as a destination square.",
                );
            }
        }
    }
}

impl CoordVec {
    const ALL_BOARD_SQUARES: Self = CoordVec::EMPTY
        .push(Coords::R0C0)
        .push(Coords::R0C1)
        .push(Coords::R0C2)
        .push(Coords::R1C0)
        .push(Coords::R1C1)
        .push(Coords::R1C2)
        .push(Coords::R2C0)
        .push(Coords::R2C1)
        .push(Coords::R2C2)
        .push(Coords::R3C0)
        .push(Coords::R3C1)
        .push(Coords::R3C2);

    fn into_set(self) -> CoordSet {
        let mut set = CoordSet::EMPTY;
        for coord in self {
            set = set.add(coord);
        }
        set
    }
}

impl CoordSet {
    const EMPTY: Self = Self(0);

    #[must_use]
    fn add(self, coords: Coords) -> Self {
        Self(self.0 | (1 << coords.0))
    }

    fn contains(self, coords: Coords) -> bool {
        (self.0 & (1 << coords.0)) != 0
    }
}
