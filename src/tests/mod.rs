use super::{pretty::IntoPretty, *};

mod i9;
mod legal_moves;
mod parent_child_consistency;

#[test]
fn initial_search_node_is_correct() {
    insta::assert_snapshot!(SearchNode::initial().pretty());
}

#[test]
fn initial_search_node_allegiance_inversion_is_correct() {
    insta::assert_snapshot!(SearchNode::initial()
        .into_builder()
        .invert_active_player()
        .pretty());
}

#[test]
fn initial_search_node_partially_built_is_correct() {
    insta::assert_snapshot!(SearchNode::initial().into_builder().build().pretty());
}

#[test]
fn initial_search_node_allegiance_inverted_partially_built_is_correct() {
    insta::assert_snapshot!(SearchNode::initial()
        .into_builder()
        .invert_active_player()
        .build()
        .pretty());
}

#[test]
fn initial_search_node_children_are_correct() {
    insta::assert_snapshot!(SearchNode::initial().children().pretty());
}

#[test]
fn initial_search_node_child0_children_are_correct() {
    let child0 = SearchNode::initial().children()[0].pretty();
    let children = child0.0.children().pretty();
    insta::assert_snapshot!(format!("parent:\n{child0}\n\nchildren:\n{children}"));
}
