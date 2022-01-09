use bitvec::prelude::*;
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
        log::debug!("exceeded heap size! now calling collect function for mark_compact");
        // we want to mark all nodes first

        // create marking bitmap using breadth-first traversal of the tree
        let mut marked_node_pointers: BitVec = bitvec![0; self.committed_memory.len()];
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
                // if the node isn't already marked
                if !marked_node_pointers[usize::from(node)] {
                    // we mark it because it means it's accessible
                    marked_node_pointers.set(usize::from(node), true);
                    // then add the rest of its children to the back of the queue
                    for child_node_pointer in &self.get(node).unwrap().children {
                        worklist.push_back(*child_node_pointer);
                    }
                }
            }
        }
        // now all our reachable objects should be marked, everything not in the list is garbo
        // we only care about the marked objects from now on
        // log::trace!(
        //     "marked_node_pointers is this long: {}",
        //     marked_node_pointers.len()
        // );
        let iterator = marked_node_pointers.iter().enumerate().filter(|(_, v)| **v);
        log::trace!(
            "we have this many marked nodes: {}",
            iterator.clone().count()
        );

        // free starts at 0, the beginning of the point which we wish to compact to
        let mut free = 0;
        // compact occurs next
        {
            // the first step is to calculate new locations of all objects

            // we iterate over all objects in the heap TODO vec of nodes seems really inefficient
            // if it is marked,
            iterator.clone().try_for_each(|(idx, _)| -> Result<()> {
                let mut marked_node = self.get_mut(NodePointer::from(idx)).unwrap();
                // set its forwarding address equal to free
                marked_node.forwarding_address = Some(NodePointer::from(free));
                // then bump free
                free += 1;
                if free > self.committed_memory.len() {
                    Err("not enough space on heap to allocate new object. Something went wrong with marking objects in `collect()`".into())
                } else {
                    Ok(())
                }
            })?;
        }

        {
            // now we update object references
            //
            //
            // for every marked parent
            iterator.clone().for_each(|(idx, _)| {
                //   for every child of the marked node
                for i in 0..self.get_mut(NodePointer::from(idx)).unwrap().children.len() {
                    let child_node_pointer = self.get(NodePointer::from(idx)).unwrap().children[i];
                    //  get the actual child_node's forwarding address
                    let forwarding_address = self
                        .get(child_node_pointer)
                        .unwrap()
                        .forwarding_address
                        .unwrap();

                    //  then set the child_node to child node's forwarding address
                    self.get_mut(NodePointer::from(idx)).unwrap().children[i] = forwarding_address;
                }
            });
        }
        // println!("cool");

        {
            // actually move the objects
            //   for every marked node
            iterator.for_each(|(idx, _)| {
                let forwarding_address = self
                    .get(NodePointer::from(idx))
                    .unwrap()
                    .forwarding_address
                    .unwrap();
                // swap node's current position with node's forwarding position,
                // as long as they're not already in the right palce
                if usize::from(forwarding_address) != idx {
                    self.committed_memory.swap(
                        usize::from(NodePointer::from(idx)),
                        usize::from(forwarding_address),
                    );
                }
            });
        }
        self.free = free;
        Ok(())
    }
    
    #[inline(always)]
    fn get(&self, node_pointer: NodePointer) -> Option<&Node> {
        self.committed_memory.get(usize::from(node_pointer))
    }

    #[inline(always)]
    fn get_mut(&mut self, node_pointer: NodePointer) -> Option<&mut Node> {
        self.committed_memory
            .get_mut(usize::from(node_pointer))
    }

    // breadh-first traversal of node, printing out
    fn dump(&self, node_pointer: NodePointer) -> Result<String> {
        let mut elements = Vec::new();

        let mut worklist: VecDeque<NodePointer> = VecDeque::new();
        worklist.push_back(node_pointer);

        while let Some(node_pointer) = worklist.pop_front() {
            let node = self.get(node_pointer).unwrap();
            if let Some(value) = node.value {
                elements.push(value.to_string());
            }
            for child in &node.children {
                worklist.push_back(*child);
            }
        }
        Ok(elements.join(", "))
    }

    fn free(&self) -> usize {
        self.free
    }
}
