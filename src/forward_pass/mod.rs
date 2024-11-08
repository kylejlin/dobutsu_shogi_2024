use super::*;

#[cfg(test)]
mod tests;

/// Returns a sorted vector of all states reachable from the provided initial state.
pub fn reachable_states(initial_state: SearchNode) -> Vec<SearchNode> {
    let mut reachable_states = StateSet::empty();
    reachable_states.add(initial_state);

    let mut stack = vec![initial_state];

    loop {
        let top_mut = stack.last_mut().unwrap();
        let (new_top, new_child) = top_mut.next_child();
        *top_mut = new_top;

        if new_child.is_some() {
            let new_child = new_child.unchecked_unwrap();
            if !reachable_states.add(new_child).did_addend_already_exist {
                stack.push(new_child);
            }
        } else {
            stack.pop();

            if stack.is_empty() {
                break;
            }
        }
    }

    reachable_states.into_sorted_vec()
}

#[derive(Clone, Debug)]
struct StateSet {
    raw: [Option<Box<StateSetNode<StateSetNode<StateSetNode<StateSetNode<[Bitset16; 16]>>>>>>;
        256 * 256],
}

type StateSetNode<T> = [Option<Box<T>>; 16];

#[derive(Clone, Copy, Debug, Default)]
struct Bitset16(u16);

#[derive(Clone, Copy, Debug)]
struct DidAddendAlreadyExist {
    did_addend_already_exist: bool,
}

impl StateSet {
    fn empty() -> Self {
        let empty: Option<
            Box<StateSetNode<StateSetNode<StateSetNode<StateSetNode<[Bitset16; 16]>>>>>,
        > = Default::default();

        let mut v = Vec::with_capacity(256 * 256);

        for _ in 0..256 * 256 {
            v.push(empty.clone());
        }

        Self {
            raw: v.try_into().unwrap(),
        }
    }

    fn add(&mut self, node: SearchNode) -> DidAddendAlreadyExist {
        let bin0 = self.raw[(node.0 >> (56 - 16)) as usize].get_or_insert_with(Default::default);
        let bin1 = bin0[((node.0 >> (56 - 16 - 4)) & 0b1111) as usize]
            .get_or_insert_with(Default::default);
        let bin2 = bin1[((node.0 >> (56 - 16 - 2 * 4)) & 0b1111) as usize]
            .get_or_insert_with(Default::default);
        let bin3 = bin2[((node.0 >> (56 - 16 - 3 * 4)) & 0b1111) as usize]
            .get_or_insert_with(Default::default);
        let bin4 = bin3[((node.0 >> (56 - 16 - 4 * 4)) & 0b1111) as usize]
            .get_or_insert_with(Default::default);
        let bin5 = &mut bin4[((node.0 >> (56 - 16 - 5 * 4)) & 0b1111) as usize];

        let i6 = (node.0 >> (56 - 16 - 6 * 4)) & 0b1111;
        let mask = 1 << i6;

        let did_addend_already_exist = bin5.0 & mask != 0;

        bin5.0 |= mask;

        DidAddendAlreadyExist {
            did_addend_already_exist,
        }
    }

    fn into_sorted_vec(self) -> Vec<SearchNode> {
        let mut raw = Vec::new();

        self.write(&mut raw);

        raw.sort_unstable();

        raw
    }

    fn write(&self, out: &mut Vec<SearchNode>) {
        for (i0, bin0) in self.raw.iter().enumerate() {
            let Some(bin0) = bin0 else {
                continue;
            };
            let prefix = (i0 as u64) << (56 - 16);
            self.write_bin0(prefix, bin0, out);
        }
    }

    fn write_bin0(
        &self,
        prefix: u64,
        bin0: &StateSetNode<StateSetNode<StateSetNode<StateSetNode<[Bitset16; 16]>>>>,
        out: &mut Vec<SearchNode>,
    ) {
        for (i1, bin1) in bin0.iter().enumerate() {
            let Some(bin1) = bin1 else {
                continue;
            };
            let prefix = prefix | ((i1 as u64) << (56 - 16 - 4));
            self.write_bin1(prefix, bin1, out);
        }
    }

    fn write_bin1(
        &self,
        prefix: u64,
        bin1: &StateSetNode<StateSetNode<StateSetNode<[Bitset16; 16]>>>,
        out: &mut Vec<SearchNode>,
    ) {
        for (i2, bin2) in bin1.iter().enumerate() {
            let Some(bin2) = bin2 else {
                continue;
            };
            let prefix = prefix | ((i2 as u64) << (56 - 16 - 2 * 4));
            self.write_bin2(prefix, bin2, out);
        }
    }

    fn write_bin2(
        &self,
        prefix: u64,
        bin2: &StateSetNode<StateSetNode<[Bitset16; 16]>>,
        out: &mut Vec<SearchNode>,
    ) {
        for (i3, bin3) in bin2.iter().enumerate() {
            let Some(bin3) = bin3 else {
                continue;
            };
            let prefix = prefix | ((i3 as u64) << (56 - 16 - 3 * 4));
            self.write_bin3(prefix, bin3, out);
        }
    }

    fn write_bin3(
        &self,
        prefix: u64,
        bin3: &StateSetNode<[Bitset16; 16]>,
        out: &mut Vec<SearchNode>,
    ) {
        for (i4, bin4) in bin3.iter().enumerate() {
            let Some(bin4) = bin4 else {
                continue;
            };
            let prefix = prefix | ((i4 as u64) << (56 - 16 - 4 * 4));
            self.write_bin4(prefix, bin4, out);
        }
    }

    fn write_bin4(&self, prefix: u64, bin4: &[Bitset16; 16], out: &mut Vec<SearchNode>) {
        for (i5, bin5) in bin4.iter().enumerate() {
            let prefix = prefix | ((i5 as u64) << (56 - 16 - 5 * 4));
            self.write_bin5(prefix, *bin5, out);
        }
    }

    fn write_bin5(&self, prefix: u64, bin5: Bitset16, out: &mut Vec<SearchNode>) {
        for i6 in 0..16 {
            if bin5.0 & (1 << i6) != 0 {
                let prefix = prefix | ((i6 as u64) << (56 - 16 - 6 * 4));
                out.push(SearchNode(prefix));
            }
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

        let next_action: u64 = 0b001_0000;

        Self(
            (chick0 << offsets::CHICK0)
                | (chick1 << offsets::CHICK1)
                | (elephant0 << offsets::ELEPHANT0)
                | (elephant1 << offsets::ELEPHANT1)
                | (giraffe0 << offsets::GIRAFFE0)
                | (giraffe1 << offsets::GIRAFFE1)
                | (active_lion << offsets::ACTIVE_LION)
                | (passive_lion << offsets::PASSIVE_LION)
                | (next_action << offsets::NEXT_ACTION),
        )
    }

    fn next_child(mut self) -> (Self, OptionalSearchNode) {
        loop {
            let raw = ((self.0 >> offsets::NEXT_ACTION) & 0b111_1111) as u8;
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
}
