type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug)]
/// we'll have stack pointing to nodes on the heap
pub struct Stack {
    pub roots: Vec<Node>,
}

/// Heap includes the graph data structure, and acts pretty much like an arena
pub struct Heap {
    // the `top` of the memory != strip.len()
    // because we don't want to have to zero them out if we don't need to, and don't want to push / pop the vec
    // especially when we're compacting
    pub strip: Vec<Node>,
    // should be equal to strip.len()
    pub size: usize,
    // the size of the top, where the last piece of recognizable memory is. 1 less than strip.len()
    pub top: usize,
    pub max: usize,
}

impl Heap {
    // allocates a new node
    // we can just add a new node and return its id
    pub fn alloc(&mut self) -> Result<NodePointer> {
        // if we can fit
        if self.top != self.size {
            let node = Node::default();
            // set the node id to where the top of the heap is
            let node_pointer = self.top;
            let node_pointer = NodePointer::new(node_pointer);
            // add it to the heap
            self.strip.push(node);
            // bump the node_pointer
            self.top += 1;
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
    fn collect(&mut self) -> usize {
        todo!()
    }
}

pub mod api {
    type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
    use super::{Heap, NodePointer};

    pub fn add_child(
        parent_node_pointer: NodePointer,
        child_node_pointer: NodePointer,
        heap: &mut Heap,
    ) -> Result<()> {
        if let Some(child) = heap.strip.get_mut(child_node_pointer.idx) {
            child.parent = Some(parent_node_pointer);
        } else {
            return Err("child not found while trying to add child to parent".into());
        }

        if let Some(parent) = heap.strip.get_mut(parent_node_pointer.idx) {
            parent.children.push(child_node_pointer);
            Ok(())
        } else {
            Err("parent not found while adding child".into())
        }
    }

    pub fn children(
        parent_node_pointer: NodePointer,
        heap: &Heap,
    ) -> Result<Vec<NodePointer>> {
        if let Some(parent) = heap.strip.get(parent_node_pointer.idx) {
            Ok(parent.children.clone())
        } else {
            Err("parent not found while getting children".into())
        }
    }

    pub fn parent(child_node_pointer: NodePointer, heap: &Heap) -> Result<Option<NodePointer>> {
        if let Some(child) = heap.strip.get(child_node_pointer.idx) {
            Ok(child.parent)
        } else {
            Err("child not found while getting parent".into())
        }
    }

    pub fn value(node_pointer: NodePointer, heap: &Heap) -> Result<Option<u32>> {
        if let Some(node) = heap.strip.get(node_pointer.idx) {
            Ok(node.value)
        } else {
            Err("children not found".into())
        }
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
