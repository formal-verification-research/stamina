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
	/// Inserts a trace into the trie, or adds it if it doesn't exist yet.
	/// Returns true if the trace exists, false if it was inserted.
	pub fn exists_or_insert(&mut self, trace: &Vec<Transition>) -> bool {
		match self {
			TraceTrieNode::LeafNode => true,
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
					TraceTrieNode::LeafNode => true,
					TraceTrieNode::Node(_) => {
						*node = TraceTrieNode::LeafNode;
						false
					}
				}
			}
		}
	}
	/// Prints the trie structure for debugging purposes.
	pub fn print(&self, depth: usize) {
		match self {
			TraceTrieNode::LeafNode => {
				println!("{:indent$}Leaf", "", indent = depth * 2);
			}
			TraceTrieNode::Node(children) => {
				for (transition, child) in children {
					println!("{:indent$}{}", "", transition, indent = depth * 2);
					child.print(depth + 1);
				}
			}
		}
	}
}
