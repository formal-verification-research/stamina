use std::collections::HashMap;

use crate::model::vas_model::{VasStateVector, VasValue};
use crate::*;

/// A node in the VAS state trie.
#[derive(Clone)]
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
					// debug_message!("Traversing trie for value: {}", val);
					match node {
						VasTrieNode::Node(children) => {
							// debug_message!("At node with children: {:?}", children.keys());
							// if !children.contains_key(&val) {
							// 	debug_message!("Inserting new child for value: {}", val);
							// }
							node = children.entry(val).or_insert_with(VasTrieNode::new);
						}
						VasTrieNode::LeafNode(_) => {
							// Should not happen in normal traversal, break early
							// debug_message!("Reached leaf node unexpectedly while inserting state {:?}", state);
							break;
						}
					}
				}
				match node {
					VasTrieNode::LeafNode(existing_id) => {
						// debug_message!("[TRIE] State {:?} already exists with ID {}", state, existing_id);
						Some(*existing_id)
					}
					VasTrieNode::Node(_) => {
						*node = VasTrieNode::LeafNode(id);
						// debug_message!("[TRIE] Inserted new state {:?} with ID {}", state, id);
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
	/// Prints the trie structure for debugging purposes.
	pub fn print(&self, depth: usize) {
		match self {
			VasTrieNode::LeafNode(id) => {
				println!("{:indent$}Leaf (ID: {})", "", id, indent = depth * 2);
			}
			VasTrieNode::Node(children) => {
				for (val, child) in children {
					println!("{:indent$}{}", "", val, indent = depth * 2);
					child.print(depth + 1);
				}
			}
		}
	}
}
