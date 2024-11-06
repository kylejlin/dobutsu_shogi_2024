use super::*;

mod i9;

mod helpers;

use helpers::IntoPretty;

#[test]
fn initial_search_node_is_correct() {
    insta::assert_snapshot!(SearchNode::initial().pretty());
}

#[test]
fn initial_search_node_children_are_correct() {
    insta::assert_snapshot!(SearchNode::initial().children().pretty());
}
