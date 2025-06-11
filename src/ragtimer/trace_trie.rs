use std::collections::HashMap;

/// The ID of a transition, aliased for readability
type Transition = usize;

#[derive(Debug, Default)]
pub struct TraceTrie {
    root: TraceTrieNode,
}

#[derive(Debug, Default)]
struct TraceTrieNode {
    children: HashMap<usize, TraceTrieNode>,
    is_trace_end: bool,
}

impl TraceTrie {
    pub fn new() -> Self {
        Self {
            root: TraceTrieNode::default(),
        }
    }

    /// Check if a trace exists in the trie, and if not, insert it.
    /// Returns true if the trace already existed, false if it was newly inserted.
    pub fn contains_or_insert(&mut self, trace: &[Transition]) -> bool {
        let mut node = &mut self.root;
        for t in trace {
            node = node.children.entry(*t).or_insert_with(TraceTrieNode::default);
        }
        if node.is_trace_end {
            true
        } else {
            node.is_trace_end = true;
            false
        }
    }

    /// Insert a trace (sequence of transitions) into the trie
    pub fn insert(&mut self, trace: &[Transition]) {
        let mut node = &mut self.root;
        for t in trace {
            node = node.children.entry(t.clone()).or_insert_with(TraceTrieNode::default);
        }
        node.is_trace_end = true;
    }

    /// Check if a trace exists in the trie
    pub fn contains(&self, trace: &[Transition]) -> bool {
        let mut node = &self.root;
        for t in trace {
            match node.children.get(t) {
                Some(child) => node = child,
                None => return false,
            }
        }
        node.is_trace_end
    }
}
