use std::collections::HashMap;

use nalgebra::DVector;

use crate::model::vas_model::{VasStateVector, VasValue};
use crate::*;

/// A node in the VAS state trie.
pub enum VasTrieNode {
	LeafNode(usize),
	Node(HashMap<VasValue, VasTrieNode>),
}

/// Trie for storing VAS states, where each state is a vector of VasValue.
/// WARNING: It is the user's responsibility to ensure that the
/// ordering of the state vector is consistent.
impl VasTrieNode {
	/// Creates a new empty TrieNode.
	pub fn new() -> Self {
		VasTrieNode::Node(HashMap::new())
	}
	/// Inserts a state into the trie, or returns the first ID associated with the state if it exists.
	/// If the state is not found, it inserts the state with the given ID and returns None.
	pub fn insert_if_not_exists(&mut self, state: &VasStateVector, id: usize) -> Option<usize> {
		if id == 0 {
			error!("Error: ID 0 inserted for state {:?}", state);
		}
		match self {
			VasTrieNode::LeafNode(existing_id) => Some(*existing_id),
			VasTrieNode::Node(_) => {
				let mut node = self;
				for &val in state {
					match node {
						VasTrieNode::Node(children) => {
							node = children.entry(val).or_insert_with(VasTrieNode::new);
						}
						VasTrieNode::LeafNode(_) => {
							// Should not happen in normal traversal, break early
							break;
						}
					}
				}
				match node {
					VasTrieNode::LeafNode(existing_id) => Some(*existing_id),
					VasTrieNode::Node(_) => {
						*node = VasTrieNode::LeafNode(id);
						None
					}
				}
			}
		}
	}
	/// Gets the next available ID for a new state.
	pub fn next_available_id(&self) -> usize {
		fn max_id(node: &VasTrieNode) -> usize {
			match node {
				VasTrieNode::LeafNode(id) => *id,
				VasTrieNode::Node(children) => children.values().map(max_id).max().unwrap_or(0),
			}
		}
		max_id(self) + 1
	}
}
