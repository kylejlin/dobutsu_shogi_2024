use super::*;

use std::fmt::{self, Debug, Display, Formatter};

#[derive(Clone, Copy)]
pub struct Pretty<T>(pub T);

#[derive(Clone)]
pub struct SearchNodeSet(pub Vec<SearchNode>);

#[derive(Clone, Copy)]
struct Indented<'a> {
    s: &'a str,
    space_count: usize,
}

const GAP: &str = "                ";

pub trait IntoPretty: Sized {
    fn pretty(self) -> Pretty<Self>;
}

trait Indent {
    fn indented(&self, spaces: usize) -> Indented<'_>;
}

impl IntoPretty for SearchNode {
    fn pretty(self) -> Pretty<Self> {
        Pretty(self)
    }
}

impl IntoPretty for NodeBuilder {
    fn pretty(self) -> Pretty<Self> {
        Pretty(self)
    }
}

impl IntoPretty for Board {
    fn pretty(self) -> Pretty<Self> {
        Pretty(self)
    }
}

impl IntoPretty for Action {
    fn pretty(self) -> Pretty<Self> {
        Pretty(self)
    }
}

impl IntoPretty for SearchNodeSet {
    fn pretty(self) -> Pretty<Self> {
        Pretty(self)
    }
}

impl Indent for str {
    fn indented(&self, spaces: usize) -> Indented<'_> {
        Indented {
            s: self,
            space_count: spaces,
        }
    }
}

impl Display for Pretty<SearchNode> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0.into_builder().pretty(), f)
    }
}

impl Debug for Pretty<SearchNode> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self, f)
    }
}

impl Display for Pretty<NodeBuilder> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let board = self.0.board().pretty();
        let ply_count = (self.0 .0 >> offsets::PLY_COUNT) & 0xFF;
        let outcome = i16::from_zero_padded_i9(
            (self.0 .0 >> offsets::BEST_DISCOVERED_OUTCOME) & 0b1_1111_1111,
        );
        let outcome = if outcome < 0 {
            let t = 201 + outcome;
            format!("{outcome} (lose in {t})")
        } else if outcome > 0 {
            let t = 201 - outcome;
            format!("+{outcome} (win in {t})")
        } else {
            "0 (draw)".to_string()
        };
        let next_action = Action(((self.0 .0 >> offsets::NEXT_ACTION) & 0b111_1111) as u8).pretty();
        write!(
            f,
            "{board}\nply_count: {ply_count}\nbest_known_outcome: {outcome}\nnext_action: {next_action}",
        )
    }
}

impl Debug for Pretty<NodeBuilder> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self, f)
    }
}

impl Display for Pretty<Board> {
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

impl Debug for Pretty<Board> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self, f)
    }
}

impl Pretty<Board> {
    fn into_array(self) -> [[char; 3]; 4] {
        let mut array = [['*'; 3]; 4];
        for (row, row_array) in array.iter_mut().enumerate() {
            for (col, cell) in row_array.iter_mut().enumerate() {
                let coords = (row << 2) | col;
                *cell = self.char_at_offset(coords_to_board_offset(coords as u64));
            }
        }
        array
    }

    fn char_at_offset(self, offset: u64) -> char {
        let square = (self.0 .0 >> offset) & 0b1111;
        match square {
            0b0_000 => '*',
            0b0_001 => 'l',
            0b0_010 => 'c',
            0b0_011 => 'c',
            0b0_100 => 'e',
            0b0_101 => 'e',
            0b0_110 => 'g',
            0b0_111 => 'g',

            0b1_001 => 'L',
            0b1_010 => 'C',
            0b1_011 => 'C',
            0b1_100 => 'E',
            0b1_101 => 'E',
            0b1_110 => 'G',
            0b1_111 => 'G',

            // Bad square
            _ => '!',
        }
    }
}

impl Board {
    fn invert_active_player(self) -> Self {
        let mut out = 0;

        for row in 0..=3 {
            for col in 0..=2 {
                let coords = (row << 2) | col;
                let offset = coords_to_board_offset(coords);
                let square = (self.0 >> offset) & 0b1111;
                let square_with_inverted_allegiance = if square == 0b0_000 {
                    square
                } else {
                    square ^ 0b1_000
                };

                let inv_row = 3 - row;
                let inv_col = 2 - col;
                let inv_coords = (inv_row << 2) | inv_col;
                let inv_offset = coords_to_board_offset(inv_coords);

                out |= square_with_inverted_allegiance << inv_offset;
            }
        }

        Self(out)
    }
}

impl Display for Pretty<Action> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.0 .0 == 0 {
            return write!(f, "action_search_complete");
        }

        let actor = match self.0 .0 >> 4 {
            0b001 => "active_lion",
            0b010 => "chick0",
            0b011 => "chick1",
            0b100 => "elephant0",
            0b101 => "elephant1",
            0b110 => "giraffe0",
            0b111 => "giraffe1",

            _ => "bad_actor",
        };
        let row = (self.0 .0 >> 2) & 0b11;
        let col = self.0 .0 & 0b11;
        write!(f, "{actor} to r{row}c{col}")
    }
}

impl Debug for Pretty<Action> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self, f)
    }
}

impl SearchNode {
    pub fn children(mut self) -> SearchNodeSet {
        let mut out = vec![];
        loop {
            let Ok(action) = self.next_action() else {
                return SearchNodeSet(out);
            };

            let (new_self, child) = self.explore(action);
            self = new_self;

            if child.is_none() {
                continue;
            }

            out.push(child.unchecked_unwrap());
        }
    }
}

impl Display for Pretty<SearchNodeSet> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let divider = "=".repeat("|---|".len() * 2 + GAP.len());

        let len = self.0 .0.len();

        writeln!(f, "SearchNodeSet(len = {len}) [")?;

        for (i, node) in self.0 .0.iter().enumerate() {
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
