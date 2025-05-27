use std::collections::HashMap;

use nalgebra::DVector;

use super::vas_model::AbstractVas;

/// A node in the VAS state trie.
#[derive(Default)]
struct TrieNode {
    children: HashMap<u64, TrieNode>,
    ids: Vec<usize>,
    is_end: bool,
}

/// Trie for storing VAS states, where each state is a vector of u64.
/// WARNING: It is the user's responsibility to ensure that the 
/// ordering of the state vector is consistent.
#[derive(Default)]
pub struct VasTrie {
    root: TrieNode,
}

impl VasTrie {
    /// Creates a new, empty VasTrie.
    pub fn new() -> Self {
        Self {
            root: TrieNode::default(),
        }
    }

    /// Inserts a state into the trie.
    pub fn insert(&mut self, state: &DVector<i64>, id: usize) {
        let mut node = &mut self.root;
        for &val in state {
            node = node.children.entry(val.try_into().unwrap()).or_default();
        }
        node.is_end = true;
        node.ids.push(id);
    }

    /// Checks if a state exists in the trie.
    pub fn contains(&self, state: &[i64]) -> bool {
        let mut node = &self.root;
        for &val in state {
            match node.children.get(&(val.try_into().unwrap())) {
                Some(child) => node = child,
                None => return false,
            }
        }
        node.is_end
    }

    pub fn id_else_insert(&mut self, state: &DVector<i64>, id: usize) -> Option<usize> {
        let mut node = &mut self.root;
        for &val in state {
            node = node.children.entry(val.try_into().unwrap()).or_default();
        }
        if node.is_end {
            return Some(node.ids[0]); // State already exists (contains)
        }
        node.is_end = true;
        node.ids.push(id);
        None // State was inserted (not contains)
    }
}