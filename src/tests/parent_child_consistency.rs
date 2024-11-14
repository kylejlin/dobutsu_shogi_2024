use super::*;

#[test]
fn every_child_lists_parent() {
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
                panic!("Child did not list parent.\n\nPARENT:\n\n{parent}\n\nCHILD:\n\n{child}\n\nCHILD.PARENTS:\n\n{child_parents}\n\nPARENT.CHILDREN:\n\n{parent_children}");
            }
        });
    });
}

#[test]
fn every_parent_lists_child() {
    fuzz(1_000_000, |child| {
        child.visit_parents(|parent| {
            let mut found_child = false;
            parent.visit_children(|parent_child| {
                found_child |= parent_child == child;
            });
            if !found_child {
                let child = child.pretty();
                let child_parents = child.0.parents().pretty();
                let parent = parent.pretty();
                let parent_children = parent.0.children().pretty();
                panic!("Parent did not list child.\n\nCHILD:\n\n{child}\n\nPARENT:\n\n{parent}\n\nPARENT.CHILDREN:\n\n{parent_children}\n\nCHILD.PARENTS:\n\n{child_parents}");
            }
        })
    });
}
