use super::*;

#[ignore]
#[test]
fn every_child_lists_parent_as_parent() {
    fuzz(10_000, |state| {
        state.visit_children(|child| {
            let mut found_parent = false;
            child.visit_parents(|parent| {
                found_parent |= parent == state;
            });
            if !found_parent {
                let parent = state.pretty();
                let parent_children = parent.0.children().pretty();
                let child = child.pretty();
                let child_parents = child.0.parents().pretty();
                panic!("Child did not list parent as parent.\n\nPARENT:\n\n{parent}\n\nPARENT.CHILDREN:\n\n{parent_children}\n\nCHILD:\n\n{child}\n\nCHILD.PARENTS:\n\n{child_parents}");
            }
        })
    });
}
