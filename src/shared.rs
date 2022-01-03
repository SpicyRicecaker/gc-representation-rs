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
    pub fn dump_all<T: MemoryManager>(&self, heap: &T) -> Result<()> {
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

pub trait MemoryManager {
    fn alloc(&mut self, stack: &mut Stack) -> Result<NodePointer>;
    fn collect(&mut self, stack: &mut Stack) -> Result<usize>;
    fn committed_memory(&self) -> &[Node];
    fn committed_memory_mut(&mut self) -> &mut [Node];
}

pub struct StopAndCopyHeap {
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

pub mod api {
    type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
    use crate::shared::{Node, NodePointer};

    use super::MemoryManager;

    pub fn add_child<T: MemoryManager>(
        parent_node_pointer: NodePointer,
        child_node_pointer: NodePointer,
        heap: &mut T,
    ) -> Result<()> {
        if let Some(child) = heap.committed_memory_mut().get_mut(child_node_pointer.idx) {
            child.parent = Some(parent_node_pointer);
        } else {
            return Err("child not found while trying to add child to parent".into());
        }

        if let Some(parent) = heap.committed_memory_mut().get_mut(parent_node_pointer.idx) {
            parent.children.push(child_node_pointer);
            Ok(())
        } else {
            Err("parent not found while adding child".into())
        }
    }

    pub fn children<T: MemoryManager>(
        parent_node_pointer: NodePointer,
        heap: &T,
    ) -> Result<Vec<NodePointer>> {
        if let Some(parent) = heap.committed_memory().get(parent_node_pointer.idx) {
            Ok(parent.children.clone())
        } else {
            Err("parent not found while getting children".into())
        }
    }

    pub fn parent<T: MemoryManager>(
        child_node_pointer: NodePointer,
        heap: &T,
    ) -> Result<Option<NodePointer>> {
        if let Some(child) = heap.committed_memory().get(child_node_pointer.idx) {
            Ok(child.parent)
        } else {
            Err("child not found while getting parent".into())
        }
    }

    pub fn value<T: MemoryManager>(node_pointer: NodePointer, heap: &T) -> Result<Option<u32>> {
        if let Some(node) = heap.committed_memory().get(node_pointer.idx) {
            Ok(node.value)
        } else {
            Err("node not found when trying to get value".into())
        }
    }
    pub fn set_value<T: MemoryManager>(
        node_pointer: NodePointer,
        value: Option<u32>,
        heap: &mut T,
    ) -> Result<()> {
        if let Some(node) = heap.committed_memory_mut().get_mut(node_pointer.idx) {
            node.value = value;
            Ok(())
        } else {
            Err("node not found when trying to set value".into())
        }
    }

    pub fn forwarding_address<T: MemoryManager>(
        node_pointer: NodePointer,
        heap: &T,
    ) -> Result<Option<NodePointer>> {
        if let Some(node) = heap.committed_memory().get(node_pointer.idx) {
            Ok(node.forwarding_address)
        } else {
            Err("node not found when trying to get value".into())
        }
    }
    pub fn set_forwarding_address<T: MemoryManager>(
        node_pointer: NodePointer,
        forwarding_address: Option<NodePointer>,
        heap: &mut T,
    ) -> Result<()> {
        if let Some(node) = heap.committed_memory_mut().get_mut(node_pointer.idx) {
            node.forwarding_address = forwarding_address;
            Ok(())
        } else {
            Err("node not found when trying to set value".into())
        }
    }

    pub fn get<T: MemoryManager>(node_pointer: NodePointer, heap: &T) -> Result<&Node> {
        heap.committed_memory()
            .get(node_pointer.idx)
            .ok_or_else(|| "node pointer not found".into())
    }

    pub fn get_mut<T: MemoryManager>(node_pointer: NodePointer, heap: &mut T) -> Result<&mut Node> {
        heap.committed_memory_mut()
            .get_mut(node_pointer.idx)
            .ok_or_else(|| "node pointer not found".into())
    }
    /// deletes some children given a parent node pointer and a mutable reference to heap
    /// returns a result of nothing
    ///
    /// keep in mind that we cannot delete a node directly given a node pointer
    /// because we don't know exactly how many nodes are pointing to it
    /// we would have to do a complete traversal of the tree just to delete a node (which defeats the point of having this data structure)
    /// so instead we only allow deletions from parent
    ///
    /// this also means that a tree data structure doesn't quite perfectly
    /// represent the memory of a program, since trees only have one parent reference anyway
    pub fn delete_some_children<T: MemoryManager>(
        parent_node_pointer: NodePointer,
        number_to_remove: usize,
        heap: &mut T,
    ) -> Result<()> {
        // go to parent
        if let Some(parent) = heap.committed_memory_mut().get_mut(parent_node_pointer.idx) {
            // delete x number of children
            // we can just delete 5 children for now
            for _ in 0..number_to_remove {
                parent.children.pop();
            }
        } else {
            return Err("(delete) node to delete children from does not exist".into());
        };
        Ok(())
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

/// A node represents some kind of object in memory
/// A node doesn't technically need a parent pointer, it's literally just there for eye candy
#[derive(Debug, Default)]
pub struct Node {
    pub forwarding_address: Option<NodePointer>,
    pub parent: Option<NodePointer>,
    pub children: Vec<NodePointer>,
    pub value: Option<u32>,
}
