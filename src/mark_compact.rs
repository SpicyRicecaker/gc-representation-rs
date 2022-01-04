use std::collections::VecDeque;

use crate::shared::*;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// This mark-compact algorithm uses the LISP-2 style sliding algorithm Heap
/// includes the graph data structure, and acts pretty much like an arena
pub struct MarkCompactHeap {
    // the `top` of the memory != strip.len() because we don't want to have to
    // zero them out if we don't need to, and don't want to push / pop the vec
    // especially when we're compacting
    pub committed_memory: Vec<Node>,
    // // when the length of vector len reaches the max pub max_size: usize, //
    // the size of the top, where the last piece of recognizable memory is. 1
    // less than strip.len() pub top: usize, pub max: usize,
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
    pub fn dump(&self, node_pointer: NodePointer) {
        if let Some(node) = self.committed_memory.get(usize::from(node_pointer)) {
            if let Some(value) = node.value {
                print!("{} ", value);
            }
            for child in &node.children {
                self.dump(*child);
            }
        }
    }
}

impl MemoryManager for MarkCompactHeap {
    // allocates a new node
    // we can just add a new node and return its id
    fn alloc(&mut self, node: Node, stack: &mut Stack) -> Result<NodePointer> {
        // if our free pointer is over the committed memory length
        if self.free >= self.committed_memory.len() {
            // we need to run gc
            self.collect(stack)?;
        }
        if self.free >= self.committed_memory.len() {
            return Err("gg collection didn't result in any amount of garbage collected".into());
        }

        // set the node id to where the top of the heap is
        let node_pointer = NodePointer::from(self.free);
        // add it to the heap
        self.committed_memory[usize::from(node_pointer)] = node;
        // bump the free pointer
        self.free += 1;

        Ok(node_pointer)
    }

    /// mark-compact algorithm
    fn collect(&mut self, stack: &mut Stack) -> Result<()> {
        dbg!("exceeded heap size! now calling collect function for mark_compact");
        // we want to mark all nodes first

        // create marking bitmap using breadth-first traversal of the tree,
        // which isn't actually going to be a bitmap but rather a bunch of node
        // indices
        let mut marked_node_pointers: Vec<NodePointer> = Vec::new();
        {
            // first create a worklist, which is going to be a queue, since we're doing breadth-first traversal
            let mut worklist: VecDeque<NodePointer> = VecDeque::new();

            // populate the worklist with children from the reachable stack first
            for root in &stack.roots {
                for child in &root.children {
                    worklist.push_back(*child);
                }
            }

            // then we just keep on taking from the worklist until it's empty
            while let Some(node) = worklist.pop_front() {
                // we mark it because it means it's accessible
                marked_node_pointers.push(node);
                // then add the rest of its children to the back of the queue
                for child_node_pointer in &self.get(node).unwrap().children {
                    worklist.push_back(*child_node_pointer);
                }
            }
        }
        // now all our objects should be marked
        // we need to maintain order so let's sort marked nodes
        marked_node_pointers.sort();

        // free starts at 0, the beginning of the point which we wish to compact to
        let mut free = 0;
        // compact occurs next
        {
            // the first step is to calculate new locations of all objects

            // we iterate over all objects in the heap TODO vec of nodes seems really inefficient
            // if it is marked,
            for marked_node_pointer in &marked_node_pointers {
                let mut marked_node = self.get_mut(*marked_node_pointer).unwrap();
                // set its forwarding address equal to free
                marked_node.forwarding_address = Some(NodePointer::from(free));
                // then bump free
                free += 1;
                if free > self.committed_memory.len() {
                    return Err("not enough space on heap".into());
                }
            }
        }

        {
            // now we update object references
            //
            //
            // for every marked parent
            for marked_node_pointer in &marked_node_pointers {
                //   for every child of the marked node
                for i in 0..self.get_mut(*marked_node_pointer).unwrap().children.len() {
                    let child_node_pointer = self.get(*marked_node_pointer).unwrap().children[i];
                    //  get the actual child_node's forwarding address
                    let forwarding_address = self
                        .get(child_node_pointer)
                        .unwrap()
                        .forwarding_address
                        .unwrap();

                    //  then set the child_node to child node's forwarding address
                    self.get_mut(*marked_node_pointer).unwrap().children[i] = forwarding_address;
                }
            }
        }
        // println!("cool");

        {
            // actually move the objects
            //   for every marked node
            for marked_node_pointer in marked_node_pointers {
                let forwarding_address = self
                    .get(marked_node_pointer)
                    .unwrap()
                    .forwarding_address
                    .unwrap();
                // swap node's current position with node's forwarding position
                self.committed_memory.swap(
                    usize::from(marked_node_pointer),
                    usize::from(forwarding_address),
                );
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
    // pub fn add_child<T: MemoryManager>(
    //     parent_node_pointer: NodePointer,
    //     child_node_pointer: NodePointer,
    //     heap: &mut T,
    // ) -> Result<()> {
    //     if let Some(child) = heap.committed_memory_mut().get_mut(child_node_pointer.idx) {
    //         child.parent = Some(parent_node_pointer);
    //     } else {
    //         return Err("child not found while trying to add child to parent".into());
    //     }

    //     if let Some(parent) = heap.committed_memory_mut().get_mut(parent_node_pointer.idx) {
    //         parent.children.push(child_node_pointer);
    //         Ok(())
    //     } else {
    //         Err("parent not found while adding child".into())
    //     }
    // }

    // pub fn children<T: MemoryManager>(
    //     parent_node_pointer: NodePointer,
    //     heap: &T,
    // ) -> Result<Vec<NodePointer>> {
    //     if let Some(parent) = heap.committed_memory().get(parent_node_pointer.idx) {
    //         Ok(parent.children.clone())
    //     } else {
    //         Err("parent not found while getting children".into())
    //     }
    // }

    // pub fn parent<T: MemoryManager>(
    //     child_node_pointer: NodePointer,
    //     heap: &T,
    // ) -> Result<Option<NodePointer>> {
    //     if let Some(child) = heap.committed_memory().get(child_node_pointer.idx) {
    //         Ok(child.parent)
    //     } else {
    //         Err("child not found while getting parent".into())
    //     }
    // }

    // pub fn value<T: MemoryManager>(node_pointer: NodePointer, heap: &T) -> Result<Option<u32>> {
    //     if let Some(node) = heap.committed_memory().get(node_pointer.idx) {
    //         Ok(node.value)
    //     } else {
    //         Err("node not found when trying to get value".into())
    //     }
    // }

    // pub fn set_value<T: MemoryManager>(
    //     node_pointer: NodePointer,
    //     value: Option<u32>,
    //     heap: &mut T,
    // ) -> Result<()> {
    //     if let Some(node) = heap.committed_memory_mut().get_mut(node_pointer.idx) {
    //         node.value = value;
    //         Ok(())
    //     } else {
    //         Err("PUBLIC node not found when trying to set value".into())
    //     }
    // }

    // pub fn forwarding_address<T: MemoryManager>(
    //     node_pointer: NodePointer,
    //     heap: &T,
    // ) -> Result<Option<NodePointer>> {
    //     if let Some(node) = heap.committed_memory().get(node_pointer.idx) {
    //         Ok(node.forwarding_address)
    //     } else {
    //         Err("node not found when trying to get forwarding address".into())
    //     }
    // }

    // pub fn set_forwarding_address<T: MemoryManager>(
    //     node_pointer: NodePointer,
    //     forwarding_address: Option<NodePointer>,
    //     heap: &mut T,
    // ) -> Result<()> {
    //     if let Some(node) = heap.committed_memory_mut().get_mut(node_pointer.idx) {
    //         node.forwarding_address = forwarding_address;
    //         Ok(())
    //     } else {
    //         Err("node not found when trying to set forwarding address value".into())
    //     }
    // }

    #[inline(always)]
    fn get(&self, node_pointer: NodePointer) -> Option<&Node> {
        self.committed_memory().get(usize::from(node_pointer))
    }

    #[inline(always)]
    fn get_mut(&mut self, node_pointer: NodePointer) -> Option<&mut Node> {
        self.committed_memory_mut()
            .get_mut(usize::from(node_pointer))
    }
    // / deletes some children given a parent node pointer and a mutable reference to heap
    // / returns a result of nothing
    // /
    // / keep in mind that we cannot delete a node directly given a node pointer
    // / because we don't know exactly how many nodes are pointing to it
    // / we would have to do a complete traversal of the tree just to delete a node (which defeats the point of having this data structure)
    // / so instead we only allow deletions from parent
    // /
    // / this also means that a tree data structure doesn't quite perfectly
    // / represent the memory of a program, since trees only have one parent reference anyway
    // pub fn delete_some_children<T: MemoryManager>(
    //     parent_node_pointer: NodePointer,
    //     number_to_remove: usize,
    //     heap: &mut T,
    // ) -> Result<()> {
    //     // go to parent
    //     if let Some(parent) = heap.committed_memory_mut().get_mut(parent_node_pointer.idx) {
    //         // delete x number of children
    //         // we can just delete 5 children for now
    //         for _ in 0..number_to_remove {
    //             parent.children.pop();
    //         }
    //     } else {
    //         return Err("(delete) node to delete children from does not exist".into());
    //     };
    //     Ok(())
    // }
}
