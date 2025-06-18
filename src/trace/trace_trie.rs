use std::collections::HashMap;

// TODO: We may want to generalize this to store anything, states or transitions or whatever.

/// The ID of a transition, aliased for readability
type Transition = usize;

pub enum TraceTrieNode {
	LeafNode,
	Node(HashMap<Transition, TraceTrieNode>),
}

/// Trie for storing traces, where each node is a transition name.
impl TraceTrieNode {
	/// Creates a new empty TraceTrieNode.
	pub fn new() -> Self {
		TraceTrieNode::Node(HashMap::new())
	}
	/// Inserts a transition into the trie, or adds it if it doesn't exist yet.
	/// Returns
	pub fn insert_if_not_exists(&mut self, trace: &Vec<Transition>) {
		match self {
			TraceTrieNode::LeafNode => (),
			TraceTrieNode::Node(_) => {
				let mut node = self;
				for &transition in trace {
					match node {
						TraceTrieNode::Node(children) => {
							node = children
								.entry(transition)
								.or_insert_with(TraceTrieNode::new);
						}
						TraceTrieNode::LeafNode => {
							// Should not happen in normal traversal, break early
							break;
						}
					}
				}
				match node {
					TraceTrieNode::LeafNode => (),
					TraceTrieNode::Node(_) => {
						*node = TraceTrieNode::LeafNode;
						()
					}
				}
			}
		}
	}
}
