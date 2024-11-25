use super::*;

use std::fmt::{self, Debug, Display, Formatter};

#[derive(Clone, Copy)]
pub struct Pretty<T>(pub T);

#[derive(Clone, Copy)]
struct Indented<'a> {
    s: &'a str,
    space_count: usize,
}

#[derive(Clone, Copy)]
struct Hands {
    active: Hand,
    passive: Hand,
}

#[derive(Clone, Copy)]
struct Hand {
    lion: u8,
    chick: u8,
    elephant: u8,
    giraffe: u8,
}

#[derive(Clone, Copy)]
struct BoardWithPromotionData {
    board: Board,
    is_chick0_promoted: bool,
    is_chick1_promoted: bool,
}

const GAP: &str = "                ";

pub trait IntoPretty: Sized {
    fn pretty(self) -> Pretty<Self> {
        Pretty(self)
    }
}

trait Indent {
    fn indented(&self, spaces: usize) -> Indented<'_>;
}

impl IntoPretty for State {}
impl IntoPretty for StateBuilder {}
impl IntoPretty for Hands {}
impl IntoPretty for BoardWithPromotionData {}
impl IntoPretty for Outcome {}
impl IntoPretty for Vec<State> {}

impl Indent for str {
    fn indented(&self, spaces: usize) -> Indented<'_> {
        Indented {
            s: self,
            space_count: spaces,
        }
    }
}

impl Display for Pretty<State> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0.into_builder().pretty(), f)
    }
}

impl Debug for Pretty<State> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self, f)
    }
}

impl Display for Pretty<StateBuilder> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let hands = self.0.hands().pretty();
        let board = BoardWithPromotionData {
            board: self.0.board(),
            is_chick0_promoted: Actor::CHICK0.is_promoted(self.0),
            is_chick1_promoted: Actor::CHICK1.is_promoted(self.0),
        }
        .pretty();
        write!(f, "{hands}\n{board}",)
    }
}

impl Display for Pretty<StateAndStats> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let state = self.0.state().into_builder();
        let hands = state.hands().pretty();
        let board = BoardWithPromotionData {
            board: state.board(),
            is_chick0_promoted: Actor::CHICK0.is_promoted(state),
            is_chick1_promoted: Actor::CHICK1.is_promoted(state),
        }
        .pretty();

        let stats = self.0.stats();
        let required_child_report_count = stats.required_child_report_count();
        let best_known_outcome = stats.best_known_outcome().pretty();

        write!(f, "{hands}\n{board}\nrequired_child_report_count: {required_child_report_count}\nbest_known_outcome: {best_known_outcome}",)
    }
}

impl Debug for Pretty<StateBuilder> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self, f)
    }
}

impl StateBuilder {
    fn hands(self) -> Hands {
        let mut active = Hand::empty();
        let mut passive = Hand::empty();

        macro_rules! check_nonlion {
            ($name:ident, $piece:expr) => {
                if self.0 & $piece.coords_mask() == $piece.coords_mask() {
                    if self.0 & $piece.allegiance_mask() == 0 {
                        active.$name += 1;
                    } else {
                        passive.$name += 1;
                    }
                }
            };
        }

        check_nonlion!(chick, Nonlion::CHICK0);
        check_nonlion!(chick, Nonlion::CHICK1);
        check_nonlion!(elephant, Nonlion::ELEPHANT0);
        check_nonlion!(elephant, Nonlion::ELEPHANT1);
        check_nonlion!(giraffe, Nonlion::GIRAFFE0);
        check_nonlion!(giraffe, Nonlion::GIRAFFE1);

        if self.0 & Actor::LION.coords_mask() == Actor::LION.coords_mask() {
            // If the active lion is in somebody's hand,
            // it must be in the passive player's hand.
            passive.lion += 1;
        }

        if self.0 & PassiveLion::COORDS_MASK == PassiveLion::COORDS_MASK {
            // If the passive lion is in somebody's hand,
            // it must be in the active player's hand.
            active.lion += 1;
        }

        Hands { active, passive }
    }
}

impl Hand {
    fn empty() -> Self {
        Self {
            lion: 0,
            chick: 0,
            elephant: 0,
            giraffe: 0,
        }
    }
}

impl Display for Pretty<BoardWithPromotionData> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let [[r0c0, r0c1, r0c2], [r1c0, r1c1, r1c2], [r2c0, r2c1, r2c2], [r3c0, r3c1, r3c2]]: [[char;
            3];
            4] = self.into_array();
        let [[i_r0c0, i_r0c1, i_r0c2], [i_r1c0, i_r1c1, i_r1c2], [i_r2c0, i_r2c1, i_r2c2], [i_r3c0, i_r3c1, i_r3c2]] =
            self.0.invert_active_player().pretty().into_array();
        write!(
        f,
        "|---|{GAP}|---|\n|{r3c0}{r3c1}{r3c2}|{GAP}|{i_r3c0}{i_r3c1}{i_r3c2}|\n|{r2c0}{r2c1}{r2c2}|{GAP}|{i_r2c0}{i_r2c1}{i_r2c2}|\n|{r1c0}{r1c1}{r1c2}|{GAP}|{i_r1c0}{i_r1c1}{i_r1c2}|\n|{r0c0}{r0c1}{r0c2}|{GAP}|{i_r0c0}{i_r0c1}{i_r0c2}|\n|---|{GAP}|---|",
    )
    }
}

impl Debug for Pretty<BoardWithPromotionData> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self, f)
    }
}

impl Pretty<BoardWithPromotionData> {
    fn into_array(self) -> [[char; 3]; 4] {
        let mut array = [['*'; 3]; 4];
        for (row, row_array) in array.iter_mut().enumerate() {
            for (col, cell) in row_array.iter_mut().enumerate() {
                let coords = (row << 2) | col;
                *cell = self.char_at_offset(Coords(coords as u8).board_offset());
            }
        }
        array
    }

    fn char_at_offset(self, offset: u8) -> char {
        let square = (self.0.board.0 >> offset) & 0b1111;
        match square {
            0b0_000 => '*',
            0b0_001 => 'l',
            0b0_010 => {
                if self.0.is_chick0_promoted {
                    'h'
                } else {
                    'c'
                }
            }
            0b0_011 => {
                if self.0.is_chick1_promoted {
                    'h'
                } else {
                    'c'
                }
            }
            0b0_100 => 'e',
            0b0_101 => 'e',
            0b0_110 => 'g',
            0b0_111 => 'g',

            0b1_001 => 'L',
            0b1_010 => {
                if self.0.is_chick0_promoted {
                    'H'
                } else {
                    'C'
                }
            }
            0b1_011 => {
                if self.0.is_chick1_promoted {
                    'H'
                } else {
                    'C'
                }
            }
            0b1_100 => 'E',
            0b1_101 => 'E',
            0b1_110 => 'G',
            0b1_111 => 'G',

            // Bad square
            _ => '!',
        }
    }
}

impl BoardWithPromotionData {
    fn invert_active_player(self) -> Self {
        Self {
            board: self.board.invert_active_player(),
            is_chick0_promoted: self.is_chick0_promoted,
            is_chick1_promoted: self.is_chick1_promoted,
        }
    }
}

impl Board {
    fn invert_active_player(self) -> Self {
        let mut out = 0;

        for row in 0..=3 {
            for col in 0..=2 {
                let coords = (row << 2) | col;
                let offset = Coords(coords).board_offset();
                let square = (self.0 >> offset) & 0b1111;
                let square_with_inverted_allegiance = if square == 0b0_000 {
                    square
                } else {
                    square ^ 0b1_000
                };

                let inv_row = 3 - row;
                let inv_col = 2 - col;
                let inv_coords = (inv_row << 2) | inv_col;
                let inv_offset = Coords(inv_coords).board_offset();

                out |= square_with_inverted_allegiance << inv_offset;
            }
        }

        Self(out)
    }
}

impl Display for Pretty<Hands> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let [h0, h1, h2, h3, h4, h5] = self.into_array();
        let [i_h0, i_h1, i_h2, i_h3, i_h4, i_h5] = self.invert().into_array();
        write!(
        f,
        "|===|{GAP}|===|\n|{h0}{h1}{h2}|{GAP}|{i_h0}{i_h1}{i_h2}|\n|{h3}{h4}{h5}|{GAP}|{i_h3}{i_h4}{i_h5}|\n|===|{GAP}|===|",
    )
    }
}

impl Debug for Pretty<Hands> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self, f)
    }
}

impl Pretty<Hands> {
    fn into_array(self) -> [char; 6] {
        let mut array = ['.'; 6];
        let mut i = 0;

        macro_rules! check {
            ($allegiance:ident, $field:ident, $char_:literal) => {{
                for _ in 0..self.0.$allegiance.$field {
                    array[i] = $char_;
                    i += 1;
                }
            }};
        }

        check!(active, lion, 'l');
        check!(active, chick, 'c');
        check!(active, elephant, 'e');
        check!(active, giraffe, 'g');

        check!(passive, lion, 'L');
        check!(passive, chick, 'C');
        check!(passive, elephant, 'E');
        check!(passive, giraffe, 'G');

        array
    }

    #[must_use]
    fn invert(self) -> Self {
        Pretty(self.0.invert())
    }
}

impl Hands {
    #[must_use]
    fn invert(self) -> Self {
        Self {
            active: self.passive,
            passive: self.active,
        }
    }
}

impl Display for Pretty<Outcome> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.0 .0 < 0 {
            let delay = 201 + self.0 .0;
            return write!(f, "{} (loss in {delay})", self.0 .0);
        }

        if self.0 .0 > 0 {
            let delay = 201 - self.0 .0;
            return write!(f, "{} (win in {delay})", self.0 .0);
        }

        write!(f, "0 (draw)")
    }
}

impl Display for Pretty<Vec<State>> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let divider = "=".repeat("|---|".len() * 2 + GAP.len());

        let len = self.0.len();

        writeln!(f, "SearchNodeSet(len = {len}) [")?;

        for (i, node) in self.0.iter().enumerate() {
            let node = node.pretty();
            let node = format!("{i}:\n{node}\n{divider}\n");
            let indented = node.indented(4);
            write!(f, "{indented}")?;
        }

        write!(f, "]")
    }
}

impl Display for Indented<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let indent = " ".repeat(self.space_count);

        let mut start = 0;
        loop {
            if start >= self.s.len() {
                break Ok(());
            }

            let end = self.s[start..].find('\n').map(|n| start + n);
            let Some(end) = end else {
                let line = &self.s[start..];
                break write!(f, "{indent}{line}");
            };

            let line = &self.s[start..end];
            write!(f, "{indent}{line}\n")?;
            start = end + 1;
        }
    }
}
