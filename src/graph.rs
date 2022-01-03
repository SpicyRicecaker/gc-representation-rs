use std::collections::VecDeque;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug)]
/// we'll have "stack" pointing to nodes on the heap
/// it's really not necessary but it's useful in representing the layout of the stack
pub struct Stack {
    pub roots: Vec<Node>,
}

impl Stack {
    /// Provides a breadth-first ordered print of all the reachable values on the stack
    /// keep in mind the stack pooints into the heap
    pub fn dump_all(&self, heap: &Heap) -> Result<()> {
        // create a new queue. We use queues for breadth-first search, stack for depth-first search
        let mut queue = VecDeque::new();
        // add roots of stack first obviously
        for root in &self.roots {
            queue.push_back(root);
        }
        // we pop from front of queue
        while let Some(node) = queue.pop_front() {
            // print its value
            if let Some(value) = node.value {
                print!("{} ", value);
            }
            // then add the rest of its children to the back of the queue
            for child in &node.children {
                queue.push_back(api::get(*child, heap)?);
            }
        }
        Ok(())
    }
}

/// Heap includes the graph data structure, and acts pretty much like an arena
pub struct Heap {
    // the `top` of the memory != strip.len()
    // because we don't want to have to zero them out if we don't need to, and don't want to push / pop the vec
    // especially when we're compacting
    pub committed_memory: Vec<Node>,
    // when the length of vector len reaches the max
    pub max_size: usize,
    // // the size of the top, where the last piece of recognizable memory is. 1 less than strip.len()
    // pub top: usize,
    // pub max: usize,
}

impl Heap {
    // allocates a new node
    // we can just add a new node and return its id
    pub fn alloc(&mut self) -> Result<NodePointer> {
        // if we can fit
        if self.committed_memory.len() < self.max_size {
            let node = Node::default();
            // set the node id to where the top of the heap is
            let node_pointer = self.committed_memory.len();
            let node_pointer = NodePointer::new(node_pointer);
            // add it to the heap
            // the node_pointer is technically also bumped after we push to it
            self.committed_memory.push(node);
            Ok(node_pointer)
        } else {
            // we need to run gc
            let freed = self.collect();
            if freed != 0 {
                self.alloc()
            } else {
                Err("gg collection didn't result in any amount of garbage collected".into())
            }
        }
    }

    /// mark compact
    pub fn collect(&mut self) -> usize {
        todo!()
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

pub mod api {
    type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
    use super::{Heap, Node, NodePointer};

    pub fn add_child(
        parent_node_pointer: NodePointer,
        child_node_pointer: NodePointer,
        heap: &mut Heap,
    ) -> Result<()> {
        if let Some(child) = heap.committed_memory.get_mut(child_node_pointer.idx) {
            child.parent = Some(parent_node_pointer);
        } else {
            return Err("child not found while trying to add child to parent".into());
        }

        if let Some(parent) = heap.committed_memory.get_mut(parent_node_pointer.idx) {
            parent.children.push(child_node_pointer);
            Ok(())
        } else {
            Err("parent not found while adding child".into())
        }
    }

    pub fn children(parent_node_pointer: NodePointer, heap: &Heap) -> Result<Vec<NodePointer>> {
        if let Some(parent) = heap.committed_memory.get(parent_node_pointer.idx) {
            Ok(parent.children.clone())
        } else {
            Err("parent not found while getting children".into())
        }
    }

    pub fn parent(child_node_pointer: NodePointer, heap: &Heap) -> Result<Option<NodePointer>> {
        if let Some(child) = heap.committed_memory.get(child_node_pointer.idx) {
            Ok(child.parent)
        } else {
            Err("child not found while getting parent".into())
        }
    }

    pub fn value(node_pointer: NodePointer, heap: &Heap) -> Result<Option<u32>> {
        if let Some(node) = heap.committed_memory.get(node_pointer.idx) {
            Ok(node.value)
        } else {
            Err("node not found when trying to get value".into())
        }
    }
    pub fn set_value(node_pointer: NodePointer, value: Option<u32>, heap: &mut Heap) -> Result<()> {
        if let Some(node) = heap.committed_memory.get_mut(node_pointer.idx) {
            node.value = value;
            Ok(())
        } else {
            Err("node not found when trying to set value".into())
        }
    }
    pub fn get(node_pointer: NodePointer, heap: &Heap) -> Result<&Node> {
        heap.committed_memory
            .get(node_pointer.idx)
            .ok_or_else(|| "node pointer not found".into())
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct NodePointer {
    pub idx: usize,
}

impl NodePointer {
    pub fn new(idx: usize) -> Self {
        Self { idx }
    }
}

#[derive(Debug, Default)]
pub struct Node {
    pub parent: Option<NodePointer>,
    pub children: Vec<NodePointer>,
    pub value: Option<u32>,
}
