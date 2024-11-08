use super::*;

mod helpers;

use helpers::IntoPretty;

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
fn initial_search_node_with_initialized_next_action() {
    insta::assert_snapshot!(SearchNode::initial()
        .into_builder()
        .init_next_action()
        .pretty());
}

#[test]
fn initial_search_node_horizontally_flipped_is_correct() {
    insta::assert_snapshot!(SearchNode::initial()
        .into_builder()
        .horizontally_flip()
        .pretty());
}

#[test]
fn initial_search_node_partially_built_is_correct() {
    insta::assert_snapshot!(SearchNode::initial()
        .into_builder()
        .build_without_horizontal_normalization()
        .pretty());
}

#[test]
fn initial_search_node_allegiance_inverted_partially_built_is_correct() {
    insta::assert_snapshot!(SearchNode::initial()
        .into_builder()
        .invert_active_player()
        .build_without_horizontal_normalization()
        .pretty());
}

#[test]
fn initial_search_node_children_are_correct() {
    insta::assert_snapshot!(SearchNode::initial().children().pretty());
}

#[test]
fn initial_search_node_child0_children_are_correct() {
    let child0 = SearchNode::initial().children().0[0].pretty();
    let children = child0.0.children().pretty();
    insta::assert_snapshot!(format!("parent:\n{child0}\n\nchildren:\n{children}"));
}
