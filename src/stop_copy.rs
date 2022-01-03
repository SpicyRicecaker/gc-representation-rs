use std::collections::VecDeque;

use crate::shared::*;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// This mark-compact algorithm uses the LISP-2 style sliding algorithm
/// Heap includes the graph data structure, and acts pretty much like an arena
pub struct StopAndCopyHeap {
    // should be at the start of the heap
    pub from_space: usize,
    // should be at the middle of the heap
    pub to_space: usize,
    // // extent is always len() / 2
    // pub extent: usize,
    // where we allocate from
    pub free: usize,
    //
    pub committed_memory: Vec<Node>,
}

impl StopAndCopyHeap {
    pub fn init(size: usize) -> Self {
        let mut committed_memory: Vec<Node> = Vec::new();
        for _ in 0..size {
            committed_memory.push(Node::default());
        }
        Self {
            from_space: 0,
            to_space: size / 2,
            free: 0,
            committed_memory,
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

    /// copy function
    pub fn copy(&mut self, node_pointer: NodePointer) -> Result<NodePointer> {
        // if object has a forwarding address, it means that we've already moved it over to to space, so we can just give it its reference
        if let Some(forwarding_address) = api::forwarding_address(node_pointer, self)? {
            Ok(forwarding_address)
        } else {
            let new_node_pointer = NodePointer::new(self.free);
            // otherwise, the new nodepointer value of this object will be whatever free there is
            // now use .swap() to move nodepointer current location to its new location free
            self.committed_memory
                .swap(node_pointer.idx, new_node_pointer.idx);

            // and remember to set the forwarding address of the moved nodepointer to none
            api::set_forwarding_address(new_node_pointer, None, self)?;

            // now update the old forwarding address to include itself
            // keep in mind that this object in to space is complete garbage except for the forwarding address part
            api::set_forwarding_address(node_pointer, Some(new_node_pointer), self)?;

            // also remember to bump free
            self.free += 1;

            // finally we can return the new_node_pointer
            Ok(new_node_pointer)
        }
    }
}

impl MemoryManager for StopAndCopyHeap {
    // allocates a new node
    // we can just add a new node and return its id
    fn alloc(&mut self, stack: &mut Stack) -> Result<NodePointer> {
        // check if free is going over fromspace + tospace
        if self.free < self.from_space + self.to_space {
            let node = Node::default();
            // set the node id to where the top of the heap is
            let node_pointer = NodePointer::new(self.free);
            // add it to the heap
            self.committed_memory[node_pointer.idx] = node;
            // bump the free pointer
            self.free += 1;

            Ok(node_pointer)
        } else {
            // we need to run gc
            let freed = self.collect(stack)?;
            if freed != 0 {
                self.alloc(stack)
            } else {
                Err("gg collection didn't result in any amount of garbage collected".into())
            }
        }
    }

    /// mark-compact algorithm
    fn collect(&mut self, stack: &mut Stack) -> Result<usize> {
        // first we swap from space with tospace
        {
            // literally std::mem swap them. They're both locations, neither is size
            std::mem::swap(&mut self.to_space, &mut self.from_space);
            // set our free pointer to the new space
            self.free = self.to_space;
        }

        // copy the roots over
        // add the roots to worklist

        // for each node in worklist
        //
        //
        //    copy the references over,

        // and add the references to the worklist

        // move the roots over
        {}

        todo!()
    }

    // we provide a slice from from space to to space!
    fn committed_memory(&self) -> &[Node] {
        &self.committed_memory[self.from_space..self.from_space + self.to_space]
    }

    fn committed_memory_mut(&mut self) -> &mut [Node] {
        &mut self.committed_memory[self.from_space..self.from_space + self.to_space]
    }
}
