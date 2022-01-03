use std::collections::VecDeque;

use crate::shared::*;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// This mark-compact algorithm uses the LISP-2 style sliding algorithm
/// Heap includes the graph data structure, and acts pretty much like an arena
pub struct StopAndCopyHeap {
    // // should be at the middle of the heap (len() / 2)
    // pub from_space: usize,
    // // should be at the bottom of the heap (0)
    // pub to_space: usize,
    // // where we allocate from
    pub free: usize,
    pub committed_memory: Vec<Node>,
    pub backup_memory: Vec<Node>,
}

impl StopAndCopyHeap {
    /// allocates twice as much as the size you give it
    pub fn init(size: usize) -> Self {
        let mut committed_memory: Vec<Node> = Vec::new();
        for _ in 0..size {
            committed_memory.push(Node::default());
        }
        let mut backup_memory: Vec<Node> = Vec::new();
        for _ in 0..size {
            backup_memory.push(Node::default());
        }
        Self {
            free: 0,
            committed_memory,
            backup_memory,
        }
    }
}

impl StopAndCopyHeap {
    // breadth-first traversal of node, printing out
    pub fn dump(&self, node: NodePointer) {
        if let Some(n) = self.committed_memory.get(node.idx) {
            if let Some(value) = n.value {
                print!("{} ", value);
            }
            for child in &n.children {
                self.dump(*child);
            }
        }
    }
}

impl MemoryManager for StopAndCopyHeap {
    // allocates a new node
    // we can just add a new node and return its id
    fn alloc(&mut self, stack: &mut Stack) -> Result<NodePointer> {
        todo!()
    }

    /// mark-compact algorithm
    fn collect(&mut self, stack: &mut Stack) -> Result<usize> {
        todo!()
    }

    fn committed_memory(&self) -> &[Node] {
        &self.committed_memory
    }

    fn committed_memory_mut(&mut self) -> &mut [Node] {
        &mut self.committed_memory
    }
}
