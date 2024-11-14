use super::*;

#[test]
fn every_child_lists_parent_as_parent() {
    fuzz(1_000_000, |parent| {
        parent.visit_children(|child| {
            let mut found_parent = false;
            child.visit_parents(|child_parent| {
                found_parent |= child_parent == parent;
            });
            if !found_parent {
                let parent = parent.pretty();
                let parent_children = parent.0.children().pretty();
                let child = child.pretty();
                let child_parents = child.0.parents().pretty();
                panic!("Child did not list parent as parent.\n\nPARENT:\n\n{parent}\n\nCHILD:\n\n{child}\n\nCHILD.PARENTS:\n\n{child_parents}\n\nPARENT.CHILDREN:\n\n{parent_children}");
            }
        })
    });
}
