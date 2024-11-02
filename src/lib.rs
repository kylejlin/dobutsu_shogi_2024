#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Solution(pub u64);

const PLY_LIMIT: u8 = 200;

#[derive(Clone, Debug, PartialEq, Eq)]
struct SearchNode(pub u64);

#[derive(Clone, Debug, PartialEq, Eq)]
struct SearchQuasinode(pub u64);

struct SolutionCache {
    raw: [CacheNode<CacheNode<CacheNode<CacheNode<CacheNode<[u64; 16]>>>>>; 256 * 256],
}

type CacheNode<T> = [Option<Box<T>>; 16];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SolutionMap {
    raw: Vec<Solution>,
}

impl SearchNode {
    fn initial() -> SearchNode {
        todo!()
    }

    fn record_solution(&mut self, solution: Solution) {
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

impl From<SearchNode> for Solution {
    fn from(node: SearchNode) -> Self {
        Solution(node.0)
    }
}

impl From<SearchQuasinode> for Solution {
    fn from(quasinode: SearchQuasinode) -> Self {
        Solution(quasinode.0)
    }
}

impl From<SearchQuasinode> for SearchNode {
    fn from(quasinode: SearchQuasinode) -> Self {
        SearchNode(quasinode.0)
    }
}

impl SolutionCache {
    fn new() -> SolutionCache {
        let empty: CacheNode<CacheNode<CacheNode<CacheNode<CacheNode<[u64; 16]>>>>> =
            Default::default();
        let mut v = Vec::with_capacity(256 * 256);
        for _ in 0..256 * 256 {
            v.push(empty.clone());
        }
        SolutionCache {
            raw: v.try_into().unwrap(),
        }
    }

    fn get(&self, node: SearchNode) -> Option<Solution> {
        todo!()
    }
}

impl From<SolutionCache> for SolutionMap {
    fn from(cache: SolutionCache) -> Self {
        todo!()
    }
}

pub fn calculate() -> SolutionMap {
    let mut solution_cache = SolutionCache::new();

    let mut stack: Vec<SearchNode> = Vec::with_capacity(PLY_LIMIT as usize);
    stack.push(SearchNode::initial());

    loop {
        let last_node = stack.last().unwrap().clone();
        let explorer_index = last_node.clone().explorer_index();

        if explorer_index == 0 {
            stack.pop();

            if stack.is_empty() {
                break;
            }

            stack.last_mut().unwrap().record_solution(last_node.into());

            continue;
        }

        let last_node = stack.last_mut().unwrap();
        let new_quasinode = last_node.explore(explorer_index);

        if new_quasinode.clone().is_terminal() {
            last_node.record_solution(new_quasinode.into());
            continue;
        }

        let new_node: SearchNode = new_quasinode.into();

        if let Some(solution) = solution_cache.get(new_node.clone()) {
            last_node.record_solution(solution);
            continue;
        }

        stack.push(new_node);
    }

    solution_cache.into()
}

pub const EXPLORERS: [fn(&mut SearchNode) -> SearchQuasinode; 128] = [todo_dummy; 128];

fn todo_dummy(node: &mut SearchNode) -> SearchQuasinode {
    todo!()
}
