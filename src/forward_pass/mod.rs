use super::*;

use std::collections::VecDeque;

/// Returns a sorted vector of all states reachable from the provided initial state.
pub fn reachable_states(
    initial_state: SearchNode,
    mut on_node_processed: impl FnMut(SearchNode),
) -> Vec<SearchNode> {
    let mut reachable_states = StateSet::empty();
    reachable_states.add(initial_state);

    let mut queue = std::iter::once(initial_state).collect::<VecDeque<_>>();

    while let Some(node) = queue.pop_front() {
        node.visit_children(|new_child| {
            if !reachable_states.add(new_child).did_addend_already_exist {
                queue.push_back(new_child);
            }
        });

        on_node_processed(node);
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
