use super::*;

mod i9;

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
fn initial_search_node_horizontally_flipped_is_correct() {
    insta::assert_snapshot!(SearchNode::initial()
        .into_builder()
        .horizontally_flip()
        .pretty());
}

#[test]
fn initial_search_node_children_are_correct() {
    insta::assert_snapshot!(SearchNode::initial().children().pretty());
}
