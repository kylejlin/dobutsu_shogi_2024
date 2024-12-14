use super::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct CoordSet(u16);

#[test]
fn every_start_square_lists_original_square_as_dest_square() {
    every_start_square_lists_original_square_as_dest_square_for_actor(Actor::LION, false);
    every_start_square_lists_original_square_as_dest_square_for_actor(Actor::LION, true);

    every_start_square_lists_original_square_as_dest_square_for_actor(Actor::CHICK0, false);
    every_start_square_lists_original_square_as_dest_square_for_actor(Actor::CHICK0, true);

    every_start_square_lists_original_square_as_dest_square_for_actor(Actor::CHICK1, false);
    every_start_square_lists_original_square_as_dest_square_for_actor(Actor::CHICK1, true);

    every_start_square_lists_original_square_as_dest_square_for_actor(Actor::ELEPHANT0, false);
    every_start_square_lists_original_square_as_dest_square_for_actor(Actor::ELEPHANT0, true);

    every_start_square_lists_original_square_as_dest_square_for_actor(Actor::ELEPHANT1, false);
    every_start_square_lists_original_square_as_dest_square_for_actor(Actor::ELEPHANT1, true);

    every_start_square_lists_original_square_as_dest_square_for_actor(Actor::GIRAFFE0, false);
    every_start_square_lists_original_square_as_dest_square_for_actor(Actor::GIRAFFE0, true);

    every_start_square_lists_original_square_as_dest_square_for_actor(Actor::GIRAFFE1, false);
    every_start_square_lists_original_square_as_dest_square_for_actor(Actor::GIRAFFE1, true);
}

fn every_start_square_lists_original_square_as_dest_square_for_actor(
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

#[test]
fn every_dest_square_lists_original_square_as_start_square() {
    every_dest_square_lists_original_square_as_start_square_for_actor(Actor::LION, false);
    every_dest_square_lists_original_square_as_start_square_for_actor(Actor::LION, true);

    every_dest_square_lists_original_square_as_start_square_for_actor(Actor::CHICK0, false);
    every_dest_square_lists_original_square_as_start_square_for_actor(Actor::CHICK0, true);

    every_dest_square_lists_original_square_as_start_square_for_actor(Actor::CHICK1, false);
    every_dest_square_lists_original_square_as_start_square_for_actor(Actor::CHICK1, true);

    every_dest_square_lists_original_square_as_start_square_for_actor(Actor::ELEPHANT0, false);
    every_dest_square_lists_original_square_as_start_square_for_actor(Actor::ELEPHANT0, true);

    every_dest_square_lists_original_square_as_start_square_for_actor(Actor::ELEPHANT1, false);
    every_dest_square_lists_original_square_as_start_square_for_actor(Actor::ELEPHANT1, true);

    every_dest_square_lists_original_square_as_start_square_for_actor(Actor::GIRAFFE0, false);
    every_dest_square_lists_original_square_as_start_square_for_actor(Actor::GIRAFFE0, true);

    every_dest_square_lists_original_square_as_start_square_for_actor(Actor::GIRAFFE1, false);
    every_dest_square_lists_original_square_as_start_square_for_actor(Actor::GIRAFFE1, true);
}

fn every_dest_square_lists_original_square_as_start_square_for_actor(
    actor: Actor,
    is_promoted: bool,
) {
    for original_square in CoordVec::ALL_BOARD_SQUARES {
        let dest_squares = actor.legal_dest_squares(is_promoted, original_square);
        for dest_square in dest_squares {
            let start_squares = actor
                .legal_starting_squares(is_promoted, dest_square)
                .into_set();
            if !start_squares.contains(original_square) {
                panic!(
                    "When {actor:?} was at {original_square:?}, it listed {dest_square:?} as a destination square, but when it was at said destination square (i.e., {dest_square:?}), it did not list {original_square:?} as a starting square.",
                );
            }
        }
    }
}

/// It's redundant to test chick0 and chick1, as well as elephant0 and elephant1, etc.
/// So, we only test chick0, elephant0, giraffe0, and the lion.
/// In another test, we ensure that chick0 and chick1 (as well as elephant0 and elephant1, etc., respectively) have the same legal moves.
#[test]
fn test_piece0_legal_destination_square_snapshots() {
    let mut out = String::new();

    // Test promotables.
    out += "# PROMOTABLES:\n\n";
    for actor in [Actor::CHICK0] {
        // Add nonpromoted destinations.
        out += "## NONPROMOTED:\n\n";
        for start_square in CoordVec::ALL_BOARD_SQUARES {
            let dest_squares = actor.legal_dest_squares(false, start_square);
            let mut board: [char; 16] = ['.'; 16];
            board[start_square.0 as usize] = actor.char_();
            for d in dest_squares {
                board[d.0 as usize] = 'X';
            }
            let [r0c0, r0c1, r0c2, _, r1c0, r1c1, r1c2, _, r2c0, r2c1, r2c2, _, r3c0, r3c1, r3c2, _] =
                board;
            out.push_str(&format!(
                "|---|\n|{r3c0}{r3c1}{r3c2}|\n|{r2c0}{r2c1}{r2c2}|\n|{r1c0}{r1c1}{r1c2}|\n|{r0c0}{r0c1}{r0c2}|\n|---|\n\n",
            ));
        }

        // Add promoted destinations.
        out += "## PROMOTED:\n\n";
        for start_square in CoordVec::ALL_BOARD_SQUARES {
            let dest_squares = actor.legal_dest_squares(true, start_square);
            let mut board: [char; 16] = ['.'; 16];
            board[start_square.0 as usize] = actor.char_();
            for d in dest_squares {
                board[d.0 as usize] = 'X';
            }
            let [r0c0, r0c1, r0c2, _, r1c0, r1c1, r1c2, _, r2c0, r2c1, r2c2, _, r3c0, r3c1, r3c2, _] =
                board;
            out.push_str(&format!(
                "|---|\n|{r3c0}{r3c1}{r3c2}|\n|{r2c0}{r2c1}{r2c2}|\n|{r1c0}{r1c1}{r1c2}|\n|{r0c0}{r0c1}{r0c2}|\n|---|\n\n",
            ));
        }
    }

    // Test nonpromotables.
    out += "# NONPROMOTABLES:\n\n";
    for actor in [Actor::LION, Actor::ELEPHANT0, Actor::GIRAFFE0] {
        for start_square in CoordVec::ALL_BOARD_SQUARES {
            // Add nonpromoted destinations.

            let nonpromoted_dest_squares = actor.legal_dest_squares(false, start_square);
            let mut board: [char; 16] = ['.'; 16];
            board[start_square.0 as usize] = actor.char_();
            for d in nonpromoted_dest_squares {
                board[d.0 as usize] = 'X';
            }
            let [r0c0, r0c1, r0c2, _, r1c0, r1c1, r1c2, _, r2c0, r2c1, r2c2, _, r3c0, r3c1, r3c2, _] =
                board;
            out.push_str(&format!(
                "|---|\n|{r3c0}{r3c1}{r3c2}|\n|{r2c0}{r2c1}{r2c2}|\n|{r1c0}{r1c1}{r1c2}|\n|{r0c0}{r0c1}{r0c2}|\n|---|\n\n",
            ));

            // There should not be any promoted destinations.
            let promoted_dest_squares = actor.legal_dest_squares(true, start_square);
            assert!(promoted_dest_squares.is_empty());
        }
    }

    insta::assert_snapshot!(out);
}

#[test]
fn piece0_and_piece1_have_the_same_set_of_moves() {
    for (actor0, actor1) in [
        (Actor::CHICK0, Actor::CHICK1),
        (Actor::ELEPHANT0, Actor::ELEPHANT1),
        (Actor::GIRAFFE0, Actor::GIRAFFE1),
    ] {
        let actor0_nonpromoted = actor0.legal_dest_squares(false, Coords::R0C0).into_set();
        let actor0_promoted = actor0.legal_dest_squares(true, Coords::R0C0).into_set();
        let actor1_nonpromoted = actor1.legal_dest_squares(false, Coords::R0C0).into_set();
        let actor1_promoted = actor1.legal_dest_squares(true, Coords::R0C0).into_set();

        assert_eq!(actor0_nonpromoted, actor1_nonpromoted);
        assert_eq!(actor0_promoted, actor1_promoted);
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

    fn is_empty(self) -> bool {
        (self.0 & 0b1111) == 0
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

impl Actor {
    fn char_(self) -> char {
        match self {
            Actor::LION => 'l',
            Actor::CHICK0 => 'c',
            Actor::CHICK1 => 'c',
            Actor::ELEPHANT0 => 'e',
            Actor::ELEPHANT1 => 'e',
            Actor::GIRAFFE0 => 'g',
            Actor::GIRAFFE1 => 'g',
            _ => '?',
        }
    }
}
