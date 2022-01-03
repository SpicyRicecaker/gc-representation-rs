use std::collections::VecDeque;

use crate::shared::*;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// This mark-compact algorithm uses the LISP-2 style sliding algorithm
/// Heap includes the graph data structure, and acts pretty much like an arena
pub struct MarkCompactHeap {
    // the `top` of the memory != strip.len()
    // because we don't want to have to zero them out if we don't need to, and don't want to push / pop the vec
    // especially when we're compacting
    pub committed_memory: Vec<Node>,
    // // when the length of vector len reaches the max
    // pub max_size: usize,
    // // the size of the top, where the last piece of recognizable memory is. 1 less than strip.len()
    // pub top: usize,
    // pub max: usize,
    pub free: usize,
}

impl MarkCompactHeap {
    pub fn init(size: usize) -> Self {
        let mut committed_memory: Vec<Node> = Vec::new();
        for _ in 0..size {
            committed_memory.push(Node::default());
        }
        Self {
            committed_memory,
            free: 0,
        }
    }
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

impl MemoryManager for MarkCompactHeap {
    // allocates a new node
    // we can just add a new node and return its id
    fn alloc(&mut self, stack: &mut Stack) -> Result<NodePointer> {
        // if our free pointer is over the committed memory length
        if self.free >= self.committed_memory.len() {
            // we need to run gc
            self.collect(stack)?;
        }
        if self.free >= self.committed_memory.len() {
            return Err("gg collection didn't result in any amount of garbage collected".into());
        }

        let node = Node::default();
        // set the node id to where the top of the heap is
        let node_pointer = NodePointer::new(self.free);
        // add it to the heap
        self.committed_memory[node_pointer.idx] = node;
        // bump the free pointer
        self.free += 1;

        Ok(node_pointer)
    }

    /// mark-compact algorithm
    fn collect(&mut self, stack: &mut Stack) -> Result<()> {
        // dbg!("exceeded heap size!");
        // # mark first
        // create marking bitmap
        // which isn't actually going to be a bitmap but rather a stack of node indices
        let mut marked_nodes: Vec<NodePointer> = Vec::new();

        // create worklist, which is going to be a queue, since we're doing breadth-first traversal
        // now do a breadth-first traversal of the tree,
        let mut worklist: VecDeque<NodePointer> = VecDeque::new();
        // populate worklist with children from stack first obviously
        for root in &stack.roots {
            for child in &root.children {
                worklist.push_back(*child);
            }
        }
        // we pop from front of queue
        while let Some(node) = worklist.pop_front() {
            // we mark it because it means it's accessible
            marked_nodes.push(node);
            // then add the rest of its children to the back of the queue
            for child_node_pointer in &api::get(node, self)?.children {
                worklist.push_back(*child_node_pointer);
            }
        }
        // now all our objects should be marked

        let mut free = 0;
        // # compact next
        {
            // first step is to calculate new locations of all objects

            // we iterate over all objects in the heap
            //
            // free starts at 0
            // if it is marked,
            for marked in &marked_nodes {
                // set its forwarding address equal to free
                api::set_forwarding_address(*marked, Some(NodePointer::new(free)), self)?;
                // then bump free
                free += 1;
            }
        }

        {
            // now we update object references
            //
            //
            // for every marked parent
            for marked in &marked_nodes {
                //   for every child of the marked node
                for i in 0..api::get(*marked, self)?.children.len() {
                    //      get the actual child_node's forwarding address
                    let forwarding_address =
                        api::forwarding_address(api::get(*marked, self)?.children[i], self)?
                            .unwrap();
                    //      then set the child_node to child node's forwarding address
                    api::get_mut(api::get(*marked, self)?.children[i], self)?.forwarding_address =
                        Some(forwarding_address);
                }
            }
        }

        {
            // actually move the objects
            //   for every marked node
            for marked in marked_nodes {
                let forwarding_address = api::forwarding_address(marked, self)?.unwrap();
                // swap node's current position with node's forwarding position
                self.committed_memory
                    .swap(marked.idx, forwarding_address.idx);
            }
        }
        self.free = free;
        Ok(())
    }

    fn committed_memory(&self) -> &[Node] {
        &self.committed_memory
    }

    fn committed_memory_mut(&mut self) -> &mut [Node] {
        &mut self.committed_memory
    }
}
