use std::collections::HashMap;

use nalgebra::DVector;

use crate::logging::messages::*;

/// A node in the VAS state trie.
pub enum TrieNode {
	LeafNode(usize),
	Node(HashMap<u64, TrieNode>),
}

/// Trie for storing VAS states, where each state is a vector of u64.
/// WARNING: It is the user's responsibility to ensure that the
/// ordering of the state vector is consistent.
impl TrieNode {
	/// Creates a new empty TrieNode.
	pub fn new() -> Self {
		TrieNode::Node(HashMap::new())
	}
	/// Inserts a state into the trie, or returns the first ID associated with the state if it exists.
	/// If the state is not found, it inserts the state with the given ID and returns None.
	pub fn insert_if_not_exists(&mut self, state: &DVector<i64>, id: usize) -> Option<usize> {
		if id == 0 {
			error!("Error: ID 0 inserted for state {:?}", state);
		}
		match self {
			TrieNode::LeafNode(existing_id) => Some(*existing_id),
			TrieNode::Node(_) => {
				let mut node = self;
				for &val in state {
					match node {
						TrieNode::Node(children) => {
							node = children.entry(val as u64).or_insert_with(TrieNode::new);
						}
						TrieNode::LeafNode(_) => {
							// Should not happen in normal traversal, break early
							break;
						}
					}
				}
				match node {
					TrieNode::LeafNode(existing_id) => Some(*existing_id),
					TrieNode::Node(_) => {
						*node = TrieNode::LeafNode(id);
						None
					}
				}
			}
		}
	}
}
