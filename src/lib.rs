const PLY_LIMIT: u8 = 200;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchNode(pub u64);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchQuasinode(pub u64);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Solution(pub u64);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SolutionMap {
    raw: Vec<Solution>,
}

impl SearchNode {
    fn initial() -> SearchNode {
        todo!()
    }

    fn record_terminal_child_outcome(&mut self, terminal_node: SearchQuasinode) {
        todo!()
    }

    fn record_nonterminal_child_outcome(&mut self, solved_node: SearchNode) {
        todo!()
    }

    fn explorer_index(self) -> usize {
        todo!()
    }

    fn explore(&mut self, explorer_index: usize) -> SearchQuasinode {
        EXPLORERS[explorer_index](self)
    }
}

impl SearchQuasinode {
    fn is_terminal(self) -> bool {
        todo!()
    }
}

impl From<SearchQuasinode> for SearchNode {
    fn from(quasinode: SearchQuasinode) -> Self {
        SearchNode(quasinode.0)
    }
}

pub fn calculate() -> SolutionMap {
    let mut stack: Vec<SearchNode> = Vec::with_capacity(PLY_LIMIT as usize);
    stack.push(SearchNode::initial());

    let mut solution_map = SolutionMap { raw: vec![] };

    loop {
        let last_node = stack.last().unwrap().clone();
        let explorer_index = last_node.clone().explorer_index();

        if explorer_index == 0 {
            stack.pop();

            if stack.is_empty() {
                break;
            }

            stack
                .last_mut()
                .unwrap()
                .record_nonterminal_child_outcome(last_node);

            continue;
        }

        let last_node = stack.last_mut().unwrap();
        let new_quasinode = last_node.explore(explorer_index);

        if new_quasinode.clone().is_terminal() {
            last_node.record_terminal_child_outcome(new_quasinode);
            continue;
        }

        stack.push(new_quasinode.into())
    }

    solution_map
}

pub const EXPLORERS: [fn(&mut SearchNode) -> SearchQuasinode; 128] = [todo_dummy; 128];

fn todo_dummy(node: &mut SearchNode) -> SearchQuasinode {
    todo!()
}
