use std::collections::HashMap;

use nalgebra::DVector;

use crate::logging::messages::*;

/// A node in the VAS state trie.
// #[derive(Default)]
// struct TrieNode {
    //     children: HashMap<u64, TrieNode>,
    //     id: usize,
    //     is_end: bool,
    // }
pub enum TrieNode {
    LeafNode(usize),
    Node(HashMap<u64, TrieNode>)
}
    
/// Trie for storing VAS states, where each state is a vector of u64.
/// WARNING: It is the user's responsibility to ensure that the 
/// ordering of the state vector is consistent.
impl TrieNode {
    pub fn new() -> Self {
        TrieNode::Node(HashMap::new())
    }

    pub fn insert_if_not_exists(&mut self, state: &DVector<i64>, id: usize) -> Option<usize> {
        if id == 0 {
            error(&format!("Error: ID 0 inserted for state {:?}", state));
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




// #[derive(Default)]
// pub struct VasTrie {
//     root: TrieNode,
// }




// impl VasTrie {
//     /// Creates a new, empty VasTrie.
//     pub fn new() -> Self {
//         Self {
//             ..Self::default()
//         }
//     }

//     /// Inserts a state into the trie.
//     pub fn insert(&mut self, state: &DVector<i64>, id: usize) {
//         let mut node = &mut self.root;
//         for &val in state {
//             node = node.children.entry(val.try_into().unwrap()).or_default();
//         }
//         node.is_end = true;
//         node.id = id;
//     }

//     /// Checks if a state exists in the trie.
//     pub fn contains(&self, state: &[i64]) -> bool {
//         let mut node = &self.root;
//         for &val in state {
//             match node.children.get(&(val.try_into().unwrap())) {
//                 Some(child) => node = child,
//                 None => return false,
//             }
//         }
//         node.is_end
//     }

//     /// Returns the first ID associated with a state if it exists, or inserts the state and returns None.
//     pub fn id_else_insert(&mut self, state: &DVector<i64>, id: usize) -> Option<usize> {
//         let mut node = &mut self.root;
//         for &val in state {
//             node = node.children.entry(val.try_into().unwrap()).or_default();
//         }
//         if node.is_end {
//             Some(node.id)
//         } else {
//             node.is_end = true;
//             node.id = id;
//             None
//         }
//     }
// }